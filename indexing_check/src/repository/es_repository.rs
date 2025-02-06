use crate::common::*;

use crate::utils_modules::io_utils::*;

use crate::model::elastic_server_config::*;
use crate::model::total_config::*;


#[doc = "Elasticsearch connection object to be used in a single tone"]
static ELASTICSEARCH_CONN_POOL: once_lazy<Arc<Mutex<VecDeque<EsRepositoryPub>>>> =
    once_lazy::new(|| Arc::new(Mutex::new(initialize_elastic_clients())));

#[doc = "Function to initialize Elasticsearch connection instances"]
pub fn initialize_elastic_clients() -> VecDeque<EsRepositoryPub> {
    info!("initialize_elastic_clients() START!");

    let config: Arc<ElasticServerConfig> = get_elasticsearch_config_info();

    /* Number of Elasticsearch connection pool */
    let pool_cnt: i32 = *config.elastic_pool_cnt();

    let es_host: &Vec<String> = config.elastic_host();
    let es_id: String = config.elastic_id().clone().unwrap_or(String::from(""));
    let es_pw: String = config.elastic_pw().clone().unwrap_or(String::from(""));

    let mut es_pool_vec: VecDeque<EsRepositoryPub> = VecDeque::new();

    for _conn_id in 0..pool_cnt {
        /* Elasticsearch connection */
        let es_connection: EsRepositoryPub = match EsRepositoryPub::new(
            es_host.clone(),
            &es_id,
            &es_pw,
        ) {
            Ok(es_client) => es_client,
            Err(err) => {
                error!("[DB Connection Error][initialize_db_clients()] Failed to create Elasticsearch client : {:?}", err);
                panic!("[DB Connection Error][initialize_db_clients()] Failed to create Elasticsearch client : {:?}", err);
            }
        };

        es_pool_vec.push_back(es_connection);
    }

    es_pool_vec
}

#[doc = "Function to get elasticsearch connection"]
async fn get_elastic_conn() -> Result<EsRepositoryPub, anyhow::Error> {
    
    /* Elasticsearch Connection 이 부족한 경우를 대비하여 대기 시간을 걸어준다. */
    for try_cnt in 1..=10 {
        
        let es_repo: Option<EsRepositoryPub> = {
            let mut pool: MutexGuard<'_, VecDeque<EsRepositoryPub>> = ELASTICSEARCH_CONN_POOL.lock().await;
            
            /* 여기서 pool.pop_front()가 실행된 후, pool은 스코프를 벗어나면서 자동 해제 */
            let inner_pool: Option<EsRepositoryPub> = pool.pop_front();
            info!("[connection get()] Elasticsearch pool.len = {:?}", pool.len());

            inner_pool
        };

        if let Some(es_repo) = es_repo {
            return Ok(es_repo);
        }
        
        warn!(
            "[Attempt {}] The Elasticsearch connection pool does not have an idle connection.",
            try_cnt
        );
        
        tokio::time::sleep(Duration::from_secs(7)).await;
    }
    
    return Err(anyhow!(
        "[Error][get_elastic_conn()] Cannot Find Elasticsearch Connection"
    ));
}

#[doc = "Function to return Elasticsearch connection objects"]
pub async fn release_elastic_conn(es_repo: EsRepositoryPub) {

    let mut pool: MutexGuard<'_, VecDeque<EsRepositoryPub>> = ELASTICSEARCH_CONN_POOL
        .lock()
        .await;
    
    pool.push_back(es_repo);
    info!("[connection return] Elasticsearch pool.len = {:?}", pool.len());
}

#[doc = "Functions that return Elasticsearch guard connections"]
pub async fn get_elastic_guard_conn() -> Result<ElasticConnGuard, anyhow::Error> {
    let es_guard: ElasticConnGuard = ElasticConnGuard::new().await?;

    Ok(es_guard)
}


#[doc = "RAII Pattern: Guard to automatically return connections"]
pub struct ElasticConnGuard {
    es_repo: Option<EsRepositoryPub>,
}

impl ElasticConnGuard {
    pub async fn new() -> Result<Self, anyhow::Error> {
        let es_repo: EsRepositoryPub = get_elastic_conn().await?;
        Ok(Self { es_repo: Some(es_repo) })
    }
}

impl Deref for ElasticConnGuard {
    type Target = EsRepositoryPub;
    fn deref(&self) -> &Self::Target {
        self.es_repo.as_ref().expect("[Error] Attempted to dereference an empty ElasticConnGuard")
    }
}

impl Drop for ElasticConnGuard {
    fn drop(&mut self) {
        if let Some(es_repo) = self.es_repo.take() {
            let _ = tokio::spawn(async move {
                release_elastic_conn(es_repo).await;
            });
        }
    }
}

#[async_trait]
pub trait EsRepository {
    async fn get_search_query(
        &self,
        es_query: &Value,
        index_name: &str,
    ) -> Result<Value, anyhow::Error>;
    async fn post_query(&self, document: &Value, index_name: &str) -> Result<(), anyhow::Error>;
    //async fn delete_query(&self, doc_id: &str, index_name: &str) -> Result<(), anyhow::Error>;

    async fn post_query_struct<T: Serialize + Sync>(
        &self,
        param_struct: &T,
        index_name: &str,
    ) -> Result<(), anyhow::Error>;
}

#[derive(Debug, Getters, Clone)]
pub struct EsRepositoryPub {
    es_clients: Vec<EsClient>,
}

#[derive(Debug, Getters, Clone, new)]
pub(crate) struct EsClient {
    host: String,
    es_conn: Elasticsearch,
}

impl EsRepositoryPub {
    pub fn new(es_url_vec: Vec<String>, es_id: &str, es_pw: &str) -> Result<Self, anyhow::Error> {
        let mut es_clients: Vec<EsClient> = Vec::new();
        
        for url in es_url_vec {
            let parse_url: String = format!("http://{}:{}@{}", es_id, es_pw, url);

            let es_url: Url = Url::parse(&parse_url)?;
            let conn_pool: SingleNodeConnectionPool = SingleNodeConnectionPool::new(es_url);
            let transport: EsTransport = TransportBuilder::new(conn_pool)
                .timeout(Duration::new(5, 0))
                .build()?;

            let elastic_conn: Elasticsearch = Elasticsearch::new(transport);
            let es_client: EsClient = EsClient::new(url, elastic_conn);

            es_clients.push(es_client);
        }

        Ok(EsRepositoryPub { es_clients })
    }

    #[doc = "Common logic: common node failure handling and node selection"]
    async fn execute_on_any_node<F, Fut>(&self, operation: F) -> Result<Response, anyhow::Error>
    where
        F: Fn(EsClient) -> Fut + Send + Sync,
        Fut: Future<Output = Result<Response, anyhow::Error>> + Send,
    {
        let mut last_error: Option<anyhow::Error> = None;

        let mut rng: StdRng = StdRng::from_entropy();
        let mut shuffled_clients = self.es_clients.clone();
        shuffled_clients.shuffle(&mut rng);

        for es_client in shuffled_clients {
            match operation(es_client).await {
                Ok(response) => return Ok(response),
                Err(err) => {
                    last_error = Some(err);
                }
            }
        }

        Err(anyhow::anyhow!(
            "All Elasticsearch nodes failed. Last error: {:?}",
            last_error
        ))
    }
}

// /* RAII pattern */
// impl Drop for EsRepositoryPub {
//     fn drop(&mut self) {
//         match ELASTICSEARCH_CONN_POOL.try_lock() {
//             Ok(mut pool) => {
//                 pool.push_back(self.clone());
//             }
//             Err(e) => {
//                 error!("[Error][EsRepositoryPub -> drop()] {:?}", e);
//             }
//         }
//     }
// }

#[async_trait]
impl EsRepository for EsRepositoryPub {
    #[doc = "Function that EXECUTES elasticsearch queries - search"]
    async fn get_search_query(
        &self,
        es_query: &Value,
        index_name: &str,
    ) -> Result<Value, anyhow::Error> {
        let response = self
            .execute_on_any_node(|es_client| async move {
                let response = es_client
                    .es_conn
                    .search(SearchParts::Index(&[index_name]))
                    .body(es_query)
                    .send()
                    .await?;

                Ok(response)
            })
            .await?;

        if response.status_code().is_success() {
            let response_body: Value = response.json::<Value>().await?;
            Ok(response_body)
        } else {
            let error_body: String = response.text().await?;
            Err(anyhow!(
                "[Elasticsearch Error][node_search_query()] response status is failed: {:?}",
                error_body
            ))
        }
    }

    #[doc = "Function that EXECUTES elasticsearch queries - indexing struct"]
    async fn post_query_struct<T: Serialize + Sync>(
        &self,
        param_struct: &T,
        index_name: &str,
    ) -> Result<(), anyhow::Error> {
        let struct_json: Value = convert_json_from_struct(param_struct)?;
        self.post_query(&struct_json, index_name).await?;

        Ok(())
    }

    #[doc = "Function that EXECUTES elasticsearch queries - indexing"]
    async fn post_query(&self, document: &Value, index_name: &str) -> Result<(), anyhow::Error> {
        let response = self
            .execute_on_any_node(|es_client| async move {
                let response = es_client
                    .es_conn
                    .index(IndexParts::Index(index_name))
                    .body(document)
                    .send()
                    .await?;

                Ok(response)
            })
            .await?;

        if response.status_code().is_success() {
            Ok(())
        } else {
            let error_message = format!("[Elasticsearch Error][node_post_query()] Failed to index document: Status Code: {}", response.status_code());
            Err(anyhow!(error_message))
        }
    }

    // #[doc = "Function that EXECUTES elasticsearch queries - delete"]
    // async fn delete_query(&self, doc_id: &str, index_name: &str) -> Result<(), anyhow::Error> {
    //     let response = self
    //         .execute_on_any_node(|es_client| async move {
    //             // let body = serde_json::json!({
    //             //     "query": {
    //             //         "ids": {
    //             //             "values": [doc_id]
    //             //         }
    //             //     }
    //             // });

    //             let response = es_client
    //                 .es_conn
    //                 //.delete_by_query(DeleteByQueryParts::Index(&[index_name]))
    //                 //.body(body)
    //                 .delete(DeleteParts::IndexId(index_name, doc_id))
    //                 .send()
    //                 .await?;

    //             println!("{:?}", response);

    //             Ok(response)
    //         })
    //         .await?;

    //     if response.status_code().is_success() {
    //         Ok(())
    //     } else {
    //         let error_message = format!("[Elasticsearch Error][node_delete_query()] Failed to delete document: Status Code: {}, Document ID: {}", response.status_code(), doc_id);
    //         Err(anyhow!(error_message))
    //     }
    // }
}
