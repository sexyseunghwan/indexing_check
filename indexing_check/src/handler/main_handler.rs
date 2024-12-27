use rand::seq::index;

use crate::common::*;

use crate::service::smtp_service::*;
use crate::service::query_service::*;
use crate::service::index_storage_service::*;

use crate::model::IndexSchedulesConfig::*;
use crate::model::Config::*;
use crate::model::VectorIndexLog::*;

use crate::utils_modules::io_utils::*;
use crate::utils_modules::time_utils::*;


pub struct MainHandler<S: SmtpService, Q: QueryService, I: IndexStorageService> {
    smtp_service: S,
    query_service: Q,
    index_storage_service: I
}


impl<S: SmtpService, Q: QueryService, I: IndexStorageService> MainHandler<S, Q, I> {
    
    pub fn new(smtp_service: S, query_service: Q, index_storage_service: I) -> Self {
        Self {
            smtp_service,
            query_service,
            index_storage_service
        }
    }
    
    
    #[doc = "인덱스 정적 색인 작업 확인 함수"]
    pub async fn main_task(&self) -> Result<(), anyhow::Error> {
        
        // 탐색할 인덱스 이름을 가져온다 -> UTC 시간 기준으로 이름이 맵핑된다.
        let curr_date_utc = get_current_utc_naivedate_str("%Y-%m-%d")?;
        //let search_index_name = format!("vector-index-logs-{}", curr_date_utc);
        let search_index_name = String::from("vector-indexing-logs-2024-12-24"); // for test
        
        // 인덱스 스케쥴 정보
        let index_schedule_info = self.index_storage_service.get_index_schedule_info();
        
        // 정보 검색
        let curr_time_utc: NaiveDateTime = get_currnet_utc_naivedatetime();
        let time_minutes_ago: NaiveDateTime = curr_time_utc - chrono::Duration::seconds(index_schedule_info.duration);
        
        // 정보 확인
        let vector_index_logs: Vec<VectorIndexLog> = self.query_service.get_indexing_movement_log(
            &search_index_name, 
            index_schedule_info.index_name(), 
            index_schedule_info.indexing_type(), 
            time_minutes_ago, 
            curr_time_utc).await?;
        
        // 벡터로그 인덱스 검수 -> 색인이 잘 되었는지 확인.
        let mut index_succ_flag = false;
        let mut cnt_succ_flag = false;
        // [00:06:02] es_voice_fishing_deduplicate_vector_index static index start
        // [00:06:03] es_voice_fishing_deduplicate_vector_index static index data collecting start
        // [00:06:05] es_voice_fishing_deduplicate_vector_index static index worked (3,475) - elapsed time : 00:00:01

        for vector_index_log in vector_index_logs {
            let log_message = vector_index_log.message();
            

            if log_message.contains("index worked") {
                succ_flag = true;
            }
            println!("{}", log_message);

        }
        
        println!("flag: {}", succ_flag);
        // 문제 있는경우 없는경우 나눠서 알람 보내주기 & 로깅
        
        //println!("{:?}", index_schedule);

        /* 확인이 필요한 색인될 인덱스 정보들 */
        // let index_schdules: IndexSchedulesConfig = read_toml_from_file::<IndexSchedulesConfig>("./config/index_list.toml")?;
        
        //println!("{:?}", self.index_storage_service.get_index_schedule_vec());
        //let index_name = self.index_storage_service.get_index_name();
        //info!("{}", index_name);
        
        Ok(())
    }
    
}