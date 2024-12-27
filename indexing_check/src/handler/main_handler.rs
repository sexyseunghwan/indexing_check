use rand::seq::index;

use crate::common::*;

use crate::service::smtp_service::*;
use crate::service::query_service::*;
use crate::service::index_storage_service::*;

use crate::model::IndexSchedulesConfig::*;
use crate::model::Config::*;

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
        let search_index_name = format!("vector-index-logs-{}", curr_date_utc);
        
        // 인덱스 스케쥴 정보
        let index_schedule_info = self.index_storage_service.get_index_schedule_info();
        
        // 정보 검색
        let curr_time_utc = get_currnet_utc_naivedatetime();
        let time_minutes_ago = curr_time_utc - chrono::Duration::seconds(index_schedule_info.duration);
        
        // 정보 확인
        
        

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