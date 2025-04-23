use crate::common::*;

use crate::repository::es_repository::*;

use crate::utils_modules::time_utils::*;
use crate::utils_modules::traits::*;

use crate::model::error_alarm_info::*;
use crate::model::error_alram_info_format::*;
use crate::model::vector_index_log::*;
use crate::model::vector_index_log_format::*;

#[async_trait]
pub trait QueryService {
    async fn get_query_result_vec<T, S>(
        &self,
        response_body: &Value,
    ) -> Result<Vec<T>, anyhow::Error>
    where
        S: DeserializeOwned,
        T: FromSearchHit<S>;
    async fn get_indexing_movement_log(
        &self,
        query_index: &str,
        index_name: &str,
        index_type: &str,
        start_dt: NaiveDateTime,
        end_dt: NaiveDateTime,
    ) -> Result<Vec<VectorIndexLogFormat>, anyhow::Error>;
    async fn post_indexing_error_info(
        &self,
        index_name: &str,
        error_alaram_info: &mut ErrorAlarmInfo,
    ) -> Result<(), anyhow::Error>;
    async fn get_error_alarm_infos(
        &self,
        index_name: &str,
    ) -> Result<Vec<ErrorAlarmInfoFormat>, anyhow::Error>;
    async fn delete_index_by_doc(
        &self,
        index_name: &str,
        doc_id: &str,
    ) -> Result<(), anyhow::Error>;
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
    async fn get_query_result_vec<T, S>(
        &self,
        response_body: &Value,
    ) -> Result<Vec<T>, anyhow::Error>
    where
        S: DeserializeOwned,
        T: FromSearchHit<S>,
    {
        let hits: &Value = response_body
            .get("hits")
            .and_then(|h| h.get("hits"))
            .ok_or_else(|| anyhow!("Missing 'hits.hits' field"))?;

        let arr: &Vec<Value> = hits
            .as_array()
            .ok_or_else(|| anyhow!("'hits.hits' is not an array"))?;

        /* ID + source 역직렬화 → T 로 변환 */
        let results: Vec<T> = arr
            .iter()
            .map(|hit| {
                /* 1) doc_id */
                let id: String = hit
                    .get("_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing or invalid '_id'"))?
                    .to_string();

                /* 2) source 역직렬화 */
                let src_val: &Value = hit
                    .get("_source")
                    .ok_or_else(|| anyhow!("Missing '_source'"))?;

                let source: S = serde_json::from_value(src_val.clone())
                    .map_err(|e| anyhow!("Failed to deserialize source: {}", e))?;

                /* 3) 트레이트 메서드로 T 생성 */
                Ok::<T, anyhow::Error>(T::from_search_hit(id, source))
            })
            .collect::<Result<_, _>>()?;

        // let results = hits.as_array()
        //     .ok_or_else(|| anyhow!("[Error][get_query_result_vec] 'hits.hits' is not an array"))?
        //     .iter()
        //     .map(|hit| {
        //         /* _id 필드(String) 추출 */
        //         let id_str: &str = hit.get("_id")
        //             .and_then(|v| v.as_str())
        //             .ok_or_else(|| anyhow!("[Error][get_query_result_vec] Missing or invalid '_id' field"))?;

        //         /* _source 필드(Value) 추출 */
        //         let source_val: &Value = hit.get("_source")
        //             .ok_or_else(|| anyhow!("[Error][get_query_result_vec] Missing '_source' field"))?;

        //         /* T로 역직렬화 */
        //         let source: T = serde_json::from_value(source_val.clone())
        //             .map_err(|e| anyhow!("[Error][get_query_result_vec] Failed to deserialize source: {}", e))?;

        //         Ok((id_str.to_string(), source))
        //     })
        //     .collect::<Result<Vec<T>, anyhow::Error>>()?;

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
    ) -> Result<Vec<VectorIndexLogFormat>, anyhow::Error> {
        let start_dt_str: String = get_str_from_naive_datetime(start_dt, "%Y-%m-%dT%H:%M:%SZ")?;
        let end_dt_str: String = get_str_from_naive_datetime(end_dt, "%Y-%m-%dT%H:%M:%SZ")?;

        let query: Value = json!({
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

        let es_client: ElasticConnGuard = get_elastic_guard_conn().await?;
        let response_body: Value = es_client.get_search_query(&query, query_index).await?;

        let result: Vec<VectorIndexLogFormat> = self
            .get_query_result_vec::<VectorIndexLogFormat, VectorIndexLog>(&response_body)
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
        let es_client: ElasticConnGuard = get_elastic_guard_conn().await?;

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
    ) -> Result<Vec<ErrorAlarmInfoFormat>, anyhow::Error> {
        let es_client: ElasticConnGuard = get_elastic_guard_conn().await?;

        let query: Value = json!({
            "query": {
                "match_all": {}
            },
            "size": 1000
        });

        let response_body: Value = es_client.get_search_query(&query, index_name).await?;
        let err_alram_infos: Vec<ErrorAlarmInfoFormat> = self
            .get_query_result_vec::<ErrorAlarmInfoFormat, ErrorAlarmInfo>(&response_body)
            .await?;

        Ok(err_alram_infos)
    }

    #[doc = "특정 인덱스의 특정 문서를 삭제해주는 함수"]
    /// # Arguments
    /// * `index_name` - 삭제 대상이 되는 인덱스 이름
    /// * `doc_id` - 삭제할 문서의 id
    ///
    /// # Returns
    /// * Result<Vec<ErrorAlarmInfo>, anyhow::Error>
    async fn delete_index_by_doc(
        &self,
        index_name: &str,
        doc_id: &str,
    ) -> Result<(), anyhow::Error> {
        let es_client: ElasticConnGuard = get_elastic_guard_conn().await?;
        es_client.delete_query(doc_id, index_name).await?;
        Ok(())
    }
}
