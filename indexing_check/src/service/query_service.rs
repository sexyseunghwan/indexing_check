use crate::common::*;

use crate::repository::es_repository::*;

use crate::utils_modules::time_utils::*;

use crate::model::VectorIndexLog::*;

#[async_trait]
pub trait QueryService {

    async fn get_indexing_movement_log(&self, query_index: &str, index_name: &str, index_type: &str, start_dt: NaiveDateTime, end_dt: NaiveDateTime) 
        -> Result<Vec<VectorIndexLog>, anyhow::Error>;

}

#[derive(Debug, new)]
pub struct QueryServicePub {}


#[async_trait]
impl QueryService for QueryServicePub {

    #[doc = ""]
    /// # Arguments
    /// * `query_index` - 쿼리의 대상이 되는 Elasticsearch 인덱스 이름
    /// * `index_name`  - 색인될 인덱스의 이름
    /// * `index_type`  - 정적색인인지 동적색인인지 구분하는 타입
    /// * `start_dt`    - 색인 시작 시각
    /// * `end_dt`      - 색인 종료 시각
    /// 
    /// # Returns
    /// * Result<Vec<VectorIndexLog>, anyhow::Error>
    async fn get_indexing_movement_log(&self, query_index: &str, index_name: &str, index_type: &str, start_dt: NaiveDateTime, end_dt: NaiveDateTime) 
        -> Result<Vec<VectorIndexLog>, anyhow::Error> {
        
        let start_dt_str = get_str_from_naive_datetime(start_dt, "%Y-%m-%dT%H:%M:%SZ")?;
        let end_dt_str = get_str_from_naive_datetime(end_dt, "%Y-%m-%dT%H:%M:%SZ")?;
        //let start_dt_str = "2024-12-24T00:00:00Z"; // for test
        //let end_dt_str = "2024-12-24T23:59:59Z"; // for test
        
        let query = json!({
            "query": {
                "bool": {
                    "must": [
                        {
                            "term": {
                                "index_name.keyword": index_name
                            }
                        },
                        {
                            "term": {
                                "state.keyword": index_type
                            }
                        },
                        {
                            "range": {
                                "timestamp": {
                                "gte": start_dt_str,
                                "lte": end_dt_str
                                }
                            }
                        }
                    ]
                }
            }
        });
        
        info!("query: {:?}", query);
        
        let es_client = get_elastic_conn()?;
        let response_body = es_client.get_search_query(&query, query_index).await?;
        let hits = &response_body["hits"]["hits"];
        
        let results: Vec<VectorIndexLog> = hits.as_array()
            .ok_or_else(|| anyhow!("[Error][total_cost_detail_specific_period()] error"))?
            .iter()
            .map(|hit| {
                hit.get("_source") 
                    .ok_or_else(|| anyhow!("[Error][total_cost_detail_specific_period()] Missing '_source' field"))
                    .and_then(|source| serde_json::from_value(source.clone()).map_err(Into::into))
            })
            .collect::<Result<Vec<_>, _>>()?; 
        
        Ok(results)
    }
}