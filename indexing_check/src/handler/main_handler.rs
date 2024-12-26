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

        println!("============================");
        
        let index_schedule = self.index_storage_service.get_index_schedule_vec();
        println!("{:?}", index_schedule);
        //println!("{:?}", index_schedule);

        /* 확인이 필요한 색인될 인덱스 정보들 */
        // let index_schdules: IndexSchedulesConfig = read_toml_from_file::<IndexSchedulesConfig>("./config/index_list.toml")?;
        
        // let system_config = get_system_config_info();
        // let chunk_size = system_config.chunk_size; /* 한 번에 처리할 요소의 수 */
        
        // let arc_index_infos: Arc<Vec<IndexSchedules>> = Arc::new(index_schdules.index);

        // let mut handles: Vec<task::JoinHandle<()>> = Vec::new();

        // for i in (0..arc_index_infos.len()).step_by(chunk_size) {

        //     let index_data_clone = Arc::clone(&arc_index_infos);
            
        //     let handle = task::spawn(async move {
        //         let chunk = index_data_clone[i..i + chunk_size.min(index_data_clone.len() - i)].to_vec();
        //         /* 예를 들어 각 청크의 요소를 처리하는 로직 */ 
        //         //println!("chunk: {:?}", chunk);
        //         self.parallel_task().await;
        //     });
            
        //     handles.push(handle);
        // }
        
        // for handle in handles {
        //     handle.await.unwrap();
        // }


        // println!("index_schdules= {:?}", index_schdules);
        
        
        Ok(())
    }
    

    // #[doc = ""]
    // async fn parallel_task(&self) -> Result<(), anyhow::Error>{

    //     println!("test");

    //     Ok(())
    // }


}