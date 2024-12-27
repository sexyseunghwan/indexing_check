use crate::common::*;

use crate::utils_modules::io_utils::*;

use crate::model::ElasticServerConfig::*;
use crate::model::Config::*;


#[doc = "Elasticsearch connection object to be used in a single tone"]
static ELASTICSEARCH_CONN_POOL: once_lazy<Arc<Mutex<VecDeque<EsRepositoryPub>>>> = once_lazy::new(|| {
    Arc::new(Mutex::new(initialize_elastic_clients()))
});


#[doc = "Function to initialize Elasticsearch connection instances"]
pub fn initialize_elastic_clients() -> VecDeque<EsRepositoryPub> {
    
    info!("initialize_elastic_clients() START!");
    
    // let config: ElasticServerConfig = match read_toml_from_file::<ElasticServerConfig>("./config/elastic_config.toml") {
    //     Ok(config) => config,
    //     Err(e) => {
    //         error!("[Error][initialize_elastic_clients()] The config file could not be found. : {:?}", e);
    //         panic!("{:?}", e)
    //     }
    // };
    let config = get_elasticsearch_config_info();
    
    /* Number of Elasticsearch connection pool */
    let pool_cnt = *config.elastic_pool_cnt();
    
    let es_host = config.elastic_host();
    let es_id = config.elastic_id().clone().unwrap_or(String::from(""));
    let es_pw = config.elastic_pw().clone().unwrap_or(String::from(""));

    let mut es_pool_vec: VecDeque<EsRepositoryPub> = VecDeque::new();
    
    for _conn_id in 0..pool_cnt {
        
        /* Elasticsearch connection */ 
        let es_connection: EsRepositoryPub = match EsRepositoryPub::new(es_host.clone(), &es_id, &es_pw) {
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
pub fn get_elastic_conn() -> Result<EsRepositoryPub, anyhow::Error> {
    
    let mut pool = match ELASTICSEARCH_CONN_POOL.lock() {
        Ok(pool) => pool,
        Err(e) => {
            return Err(anyhow!("[Error][get_elastic_conn()] {:?}", e));
        }
    };
    
    let es_repo = pool.pop_front()
        .ok_or_else(|| anyhow!("[Error][get_elastic_conn()] Cannot Find Elasticsearch Connection"))?; 

    info!("pool.len = {:?}",pool.len());

    Ok(es_repo)  
}


#[async_trait]
pub trait EsRepository {
    async fn get_search_query(&self, es_query: &Value, index_name: &str) -> Result<Value, anyhow::Error>;
    async fn post_query(&self, document: &Value, index_name: &str) -> Result<(), anyhow::Error>;
    async fn delete_query(&self, doc_id: &str, index_name: &str) -> Result<(), anyhow::Error>;

    async fn post_query_struct<T: Serialize + Sync>(&self, param_struct: &T, index_name: &str) -> Result<(), anyhow::Error>;
}


#[derive(Debug, Getters, Clone)]
pub struct EsRepositoryPub {
    es_clients: Vec<EsClient>,
}

#[derive(Debug, Getters, Clone, new)]
pub(crate) struct EsClient {
    host: String,
    es_conn: Elasticsearch
}

impl EsRepositoryPub {
    
    pub fn new(es_url_vec: Vec<String>, es_id: &str, es_pw: &str) -> Result<Self, anyhow::Error> {
        
        let mut es_clients: Vec<EsClient> = Vec::new();

        for url in es_url_vec {
            
            let parse_url: String = format!("http://{}:{}@{}", es_id, es_pw, url);
            
            let es_url: Url = Url::parse(&parse_url)?;
            let conn_pool: SingleNodeConnectionPool = SingleNodeConnectionPool::new(es_url);
            let transport: EsTransport = TransportBuilder::new(conn_pool)
                .timeout(Duration::new(5,0))
                .build()?;
            
            let elastic_conn: Elasticsearch = Elasticsearch::new(transport);
            let es_client: EsClient = EsClient::new(url, elastic_conn);
            
            es_clients.push(es_client);
        }
        
        Ok(EsRepositoryPub{es_clients})
    }


    #[doc = "Common logic: common node failure handling and node selection"]
    async fn execute_on_any_node<F, Fut>(&self, operation: F) -> Result<Response, anyhow::Error>
    where
        F: Fn(EsClient) -> Fut + Send + Sync,
        Fut: Future<Output = Result<Response, anyhow::Error>> + Send,
    {
        let mut last_error = None;
        

        let mut rng = StdRng::from_entropy();
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


/* RAII pattern */
impl Drop for EsRepositoryPub {

    fn drop(&mut self) {
        
        match ELASTICSEARCH_CONN_POOL.lock() {
            Ok(mut pool) => {
                pool.push_back(self.clone());
            },
            Err(e) => {
                error!("[Error][EsRepositoryPub -> drop()] {:?}", e);
            }
        }
    }
}



#[async_trait]
impl EsRepository for EsRepositoryPub {
    
    #[doc = "Function that EXECUTES elasticsearch queries - search"]
    async fn get_search_query(&self, es_query: &Value, index_name: &str) -> Result<Value, anyhow::Error> {
        
        let response = self.execute_on_any_node(|es_client| async move {
            
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
            let response_body = response.json::<Value>().await?;
            Ok(response_body)
        } else {
            let error_body = response.text().await?;
            Err(anyhow!("[Elasticsearch Error][node_search_query()] response status is failed: {:?}", error_body))
        }
    }
    
    
    #[doc = "Function that EXECUTES elasticsearch queries - indexing struct"]
    async fn post_query_struct<T: Serialize + Sync>(&self, param_struct: &T, index_name: &str) -> Result<(), anyhow::Error> {
        
        let struct_json = convert_json_from_struct(param_struct)?;
        self.post_query(&struct_json, index_name).await?;
        
        Ok(())
    }   

    
    #[doc = "Function that EXECUTES elasticsearch queries - indexing"]
    async fn post_query(&self, document: &Value, index_name: &str) -> Result<(), anyhow::Error> {
        
        let response = self.execute_on_any_node(|es_client| async move {
        
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
    
        
    #[doc = "Function that EXECUTES elasticsearch queries - delete"]
    async fn delete_query(&self, doc_id: &str, index_name: &str) -> Result<(), anyhow::Error> {

        let response = self.execute_on_any_node(|es_client| async move {
            
            // let body = serde_json::json!({
            //     "query": {
            //         "ids": {
            //             "values": [doc_id]
            //         }
            //     }
            // });
            
            let response = es_client
                .es_conn
                //.delete_by_query(DeleteByQueryParts::Index(&[index_name]))
                //.body(body)
                .delete(DeleteParts::IndexId(index_name, doc_id))
                .send()
                .await?;

            println!("{:?}", response);

            Ok(response)
        })
        .await?;

        if response.status_code().is_success() {
            Ok(())
        } else {
            let error_message = format!("[Elasticsearch Error][node_delete_query()] Failed to delete document: Status Code: {}, Document ID: {}", response.status_code(), doc_id);
            Err(anyhow!(error_message))
        }
        
    }
}