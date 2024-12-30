use crate::common::*;

use crate::model::EmailStruct::*;
use crate::service::smtp_service::*;
use crate::service::query_service::*;
use crate::service::index_storage_service::*;
use crate::service::telegram_service::*;

use crate::model::IndexSchedulesConfig::*;
use crate::model::VectorIndexLog::*;
use crate::model::Config::*;

use crate::utils_modules::time_utils::*;


pub struct MainHandler<S: SmtpService, Q: QueryService, I: IndexStorageService, T: TelegramService> {
    smtp_service: S,
    query_service: Q,
    index_storage_service: I,
    telegram_service: T
}


impl<S: SmtpService, Q: QueryService, I: IndexStorageService, T: TelegramService> MainHandler<S, Q, I, T> {
    
    pub fn new(smtp_service: S, query_service: Q, index_storage_service: I, telegram_service: T) -> Self {
        Self {
            smtp_service,
            query_service,
            index_storage_service,
            telegram_service
        }
    }
    

    #[doc = "인덱스 정적 색인 작업 확인 함수"]
    pub async fn main_task(&self) -> Result<(), anyhow::Error> {
        
        // 탐색할 인덱스 이름을 가져온다 -> UTC 시간 기준으로 이름이 맵핑된다.
        // ==== Prod ====
        let curr_date_utc = get_current_utc_naivedate_str("%Y-%m-%d")?;
        let search_index = get_system_config_info();
        let search_index_name = format!("{}-{}", search_index.log_index_name(), curr_date_utc);
        // ==== Prod ====
        
        // ==== DEV ====
        //let search_index_name = String::from("vector-indexing-logs-2024-12-24"); // for test
        // ==== DEV ====
        
        /* 인덱스 스케쥴 정보 */ 
        let index_schedule_info: IndexSchedules = self.index_storage_service.get_index_schedule_info();
        
        /* 현재시간 */ 
        let curr_time_utc: NaiveDateTime = get_currnet_utc_naivedatetime();
        /* 색인 동작시간 */
        let time_minutes_ago: NaiveDateTime = curr_time_utc - chrono::Duration::seconds(index_schedule_info.duration); 
        
        /* 색인 로그 확인 -> ES 쿼리 */ 
        let vector_index_logs: Vec<VectorIndexLog> = self.query_service.get_indexing_movement_log(
            &search_index_name, 
            index_schedule_info.index_name(), 
            index_schedule_info.indexing_type(), 
            time_minutes_ago, 
            curr_time_utc).await?;
        
        
        let mut index_succ_flag = false;/* 색인 완전 실패 유무 */
        let mut cnt_succ_flag = false;  /* 색인 부분 실패 유무 -> 건수가 지정값 미만인 경우 */   
        let mut indexing_cnt_num: usize = 0;  /* 색인된 문서 수 */  
        
        for vector_index_log in vector_index_logs {
            let log_message = vector_index_log.message();
            
            /* 정상적으로 색인이 되었을 경우에는 `index worked` 라는 문자열이 포함되어 있다. */
            if log_message.contains("index worked") {
                index_succ_flag = true;
            }
            
            let regex = Regex::new(r"\((.*?)\)")?;
            
            if let Some(caps) = regex.captures(log_message) {       
                
                let indexing_cnt = match caps.get(0) {
                    Some(indexing_cnt) => indexing_cnt.as_str(),
                    None => "(0)"
                };
                
                let caps_trim = &indexing_cnt[1..indexing_cnt.len() - 1];
                
                let indexing_cnt_replace = caps_trim.replace(",", "");
                indexing_cnt_num = indexing_cnt_replace.parse()?;
                
                /* 지정한 색인 문서 수 이상인 경우는 색인에 문제없다고 판단함. */
                if indexing_cnt_num >= index_schedule_info.size {
                    cnt_succ_flag = true;
                }
            }
        }
        
        if !index_succ_flag {

            /* 색인 자체가 실패가 난 경우. */ 
            /* 1. Telegram 으로 실패건 전송 */ 
            self.telegram_service.send_indexing_total_failed_msg(index_schedule_info.index_name()).await?;
            
            /* 2. email 로 실패건 전송 */ 
            let send_email_form: EmailStruct = EmailStruct::new(
                index_schedule_info.index_name(),
                indexing_cnt_num, 
                index_schedule_info.size)?;
            
            self.smtp_service.send_message_to_receivers("[Elasticsearch] Indexing ERROR Alarm", 
                send_email_form, "alba-elastic").await?;
            
            info!("Indexing Error: {:?}", index_schedule_info);
            
        } else if !cnt_succ_flag {
            
            /* 색인은 성공했지만, 색인 개수가 올바르지 않은 경우. */ 
            /* 1. Telegram 으로 실패건 전송 */ 
            self.telegram_service.send_indexing_cnt_failed_msg(
                index_schedule_info.index_name(),
                indexing_cnt_num,
                index_schedule_info.size
            ).await?;
            
            
            /* 2. email 로 실패건 전송 */ 
            let send_email_form: EmailStruct = EmailStruct::new(
                index_schedule_info.index_name(), 
                indexing_cnt_num, 
                index_schedule_info.size)?;
            
            self.smtp_service.send_message_to_receivers("[Elasticsearch] Indexing ERROR Alarm", 
                send_email_form, "alba-elastic").await?;

            info!("Indexing Error: {:?}", index_schedule_info);
        } else {

            /* 색인에 문제가 없는 경우 */
            info!("{} index completed.: {:?}", index_schedule_info.index_name(), index_schedule_info);
        }
        
        Ok(())
    }
}