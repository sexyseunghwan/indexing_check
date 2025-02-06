
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
    let mut pool: MutexGuard<'_, VecDeque<EsRepositoryPub>> = ELASTICSEARCH_CONN_POOL
        .lock()
        .await;

    /* Elasticsearch Connection 이 부족한 경우를 대비하여 대기 시간을 걸어준다. */
    for try_cnt in 1..=10 {
        if let Some(es_repo) = pool.pop_front() {
            info!("[connection get()] Elasticsearch pool.len = {:?}", pool.len());
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
