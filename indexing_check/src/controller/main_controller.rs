use crate::common::*;

use crate::service::smtp_service::*;
use crate::service::query_service::*;

use crate::model::IndexSchedulesConfig::*;

use crate::utils_modules::io_utils::*;


pub struct MainController<S: SmtpService, Q: QueryService> {
    smtp_service: S,
    query_service: Q
}


impl<S: SmtpService, Q: QueryService> MainController<S,Q> {
    
    pub fn new(smtp_service: S, query_service: Q) -> Self {
        Self {
            smtp_service,
            query_service
        }
    }
    

    #[doc = "인덱스 정적 색인 작업 확인 함수"]
    pub async fn main_task(&self) -> Result<(), anyhow::Error> {

        /* 확인이 필요한 색인될 인덱스 정보들 */
        let index_schdules: IndexSchedulesConfig = read_toml_from_file::<IndexSchedulesConfig>("./config/index_list.toml")?;

        
        Ok(())
    }
    
    
    #[doc = ""]
    pub async fn parallel_task() -> Result<(), anyhow::Error> {

        

        Ok(())
    }


}