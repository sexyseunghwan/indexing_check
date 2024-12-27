use crate::common::*;

use crate::repository::es_repository::*;

use crate::utils_modules::time_utils::*;

#[async_trait]
pub trait QueryService {

    async fn get_indexing_movement_log(&self, index_name: &str, index_type: &str, start_dt: NaiveDateTime, end_dt: NaiveDateTime) -> Result<(), anyhow::Error>;

}

#[derive(Debug, new)]
pub struct QueryServicePub {}


#[async_trait]
impl QueryService for QueryServicePub {

    async fn get_indexing_movement_log(&self, index_name: &str, index_type: &str, start_dt: NaiveDateTime, end_dt: NaiveDateTime) -> Result<(), anyhow::Error> {

        let es_conn = get_elastic_conn()?;

        let start_dt_str = get_str_from_naive_datetime(start_dt, "%Y-%m-%dT%H:%M:%SZ")?;
        let end_dt_str = get_str_from_naive_datetime(end_dt, "%Y-%m-%dT%H:%M:%SZ")?;

        let query = json!({
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
        });

        
        
        
        Ok(())
    }

}