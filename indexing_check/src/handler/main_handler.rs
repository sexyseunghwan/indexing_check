use rand::seq::index;

use crate::common::*;

use crate::service::smtp_service::*;
use crate::service::query_service::*;
use crate::service::index_storage_service::*;

use crate::model::IndexSchedulesConfig::*;
use crate::model::Config::*;

use crate::utils_modules::io_utils::*;


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
        
        //println!("{:?}", index_schedule);

        /* 확인이 필요한 색인될 인덱스 정보들 */
        // let index_schdules: IndexSchedulesConfig = read_toml_from_file::<IndexSchedulesConfig>("./config/index_list.toml")?;
        
        println!("{:?}", self.index_storage_service.get_index_schedule_vec());
        
        
        Ok(())
    }
    
}