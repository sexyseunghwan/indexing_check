use crate::common::*;

use crate::repository::es_repository::*;

use crate::utils_modules::time_utils::*;

use crate::model::error_alarm_info::*;
use crate::model::vectorIndex_log::*;

#[async_trait]
pub trait QueryService {
    async fn get_query_result_vec<T: DeserializeOwned>(
        &self,
        response_body: &Value,
    ) -> Result<Vec<T>, anyhow::Error>;
    async fn get_indexing_movement_log(
        &self,
        query_index: &str,
        index_name: &str,
        index_type: &str,
        start_dt: NaiveDateTime,
        end_dt: NaiveDateTime,
    ) -> Result<Vec<VectorIndexLog>, anyhow::Error>;
    async fn post_indexing_error_info(
        &self,
        index_name: &str,
        error_alaram_info: &mut ErrorAlarmInfo,
    ) -> Result<(), anyhow::Error>;
    async fn get_error_alarm_infos(
        &self,
        index_name: &str,
    ) -> Result<Vec<ErrorAlarmInfo>, anyhow::Error>;
}

#[derive(Debug, new)]
pub struct QueryServicePub {}

#[async_trait]
impl QueryService for QueryServicePub {
    #[doc = "Functions that return queried results as vectors"]
    /// # Arguments
    /// * `response_body` - Querying Results
    ///
    /// # Returns
    /// * Result<Vec<T>, anyhow::Error>
    async fn get_query_result_vec<T: DeserializeOwned>(
        &self,
        response_body: &Value,
    ) -> Result<Vec<T>, anyhow::Error> {
        let hits: &Value = &response_body["hits"]["hits"];

        let results: Vec<T> = hits
            .as_array()
            .ok_or_else(|| anyhow!("[Error][get_query_result_vec()] 'hits' field is not an array"))?
            .iter()
            .map(|hit| {
                let source: &Value = hit.get("_source").ok_or_else(|| {
                    anyhow!("[Error][get_query_result_vec()] Missing '_source' field")
                })?;

                let source: T = serde_json::from_value(source.clone()).map_err(|e| {
                    anyhow!(
                        "[Error][get_query_result_vec()] Failed to deserialize source: {:?}",
                        e
                    )
                })?;

                Ok::<T, anyhow::Error>(source)
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(results)
    }

    #[doc = "색인 동작 로그를 가져오는 함수"]
    /// # Arguments
    /// * `query_index` - 쿼리의 대상이 되는 Elasticsearch 인덱스 이름
    /// * `index_name`  - 색인될 인덱스의 이름
    /// * `index_type`  - 정적색인인지 동적색인인지 구분하는 타입
    /// * `start_dt`    - 색인 시작 시각
    /// * `end_dt`      - 색인 종료 시각
    ///
    /// # Returns
    /// * Result<Vec<VectorIndexLog>, anyhow::Error>
    async fn get_indexing_movement_log(
        &self,
        query_index: &str,
        index_name: &str,
        index_type: &str,
        start_dt: NaiveDateTime,
        end_dt: NaiveDateTime,
    ) -> Result<Vec<VectorIndexLog>, anyhow::Error> {
        let start_dt_str: String = get_str_from_naive_datetime(start_dt, "%Y-%m-%dT%H:%M:%SZ")?;
        let end_dt_str: String = get_str_from_naive_datetime(end_dt, "%Y-%m-%dT%H:%M:%SZ")?;
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

        let es_client: EsRepositoryPub = get_elastic_conn()?;
        let response_body: Value = es_client.get_search_query(&query, query_index).await?;

        let result: Vec<VectorIndexLog> = self
            .get_query_result_vec::<VectorIndexLog>(&response_body)
            .await?;

        Ok(result)
    }

    #[doc = "색인 실패 정보를 모니터링 Elasitcsearch 인덱스에 색인해주는 함수"]
    /// # Arguments
    /// * `index_name`  - 에러메시지 정보가 들어있는 인덱스 이름
    ///
    /// # Returns
    /// * Result<(), anyhow::Error>
    async fn post_indexing_error_info(
        &self,
        index_name: &str,
        error_alaram_info: &mut ErrorAlarmInfo,
    ) -> Result<(), anyhow::Error> {
        let es_client: EsRepositoryPub = get_elastic_conn()?;

        let cur_kor_time: NaiveDateTime = get_current_kor_naive_datetime();
        let cur_kor_time_str: String =
            get_str_from_naive_datetime(cur_kor_time, "%Y-%m-%dT%H:%M:%SZ")?;

        error_alaram_info.set_timestamp(cur_kor_time_str);

        es_client
            .post_query_struct(&error_alaram_info, index_name)
            .await?;

        Ok(())
    }

    #[doc = "색인 에러 정보들을 반환해주는 함수"]
    /// # Arguments
    /// * `index_name`  - 에러메시지 정보가 들어있는 인덱스 이름
    ///
    /// # Returns
    /// * Result<Vec<ErrorAlarmInfo>, anyhow::Error>
    async fn get_error_alarm_infos(
        &self,
        index_name: &str,
    ) -> Result<Vec<ErrorAlarmInfo>, anyhow::Error> {
        let es_client: EsRepositoryPub = get_elastic_conn()?;

        let query: Value = json!({
            "query": {
                "match_all": {},
                "size": 1000
            }
        });

        let response_body: Value = es_client.get_search_query(&query, index_name).await?;
        let err_alram_infos: Vec<ErrorAlarmInfo> = self
            .get_query_result_vec::<ErrorAlarmInfo>(&response_body)
            .await?;

        Ok(err_alram_infos)
    }
}
