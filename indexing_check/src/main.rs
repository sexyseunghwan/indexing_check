
/*
Author      : Seunghwan Shin 
Create date : 2024-00-00 
Description : 
    
History     : 2024-00-00 Seunghwan Shin       # [v.1.0.0] first create
*/
mod common;
use core::panic;

use common::*;

mod utils_modules;
use handler::main_handler;
use service::smtp_service;
use utils_modules::logger_utils::*;
use utils_modules::io_utils::*;

mod model;
use model::Config::*;
use model::IndexSchedulesConfig::*;
use model::ReceiverEmailConfig::*;

mod handler;
use handler::main_handler::*;

mod repository;

mod service;
use service::query_service::*;
use service::smtp_service::*;
use service::index_storage_service::*;

#[tokio::main]
async fn main() {
    
    /* 전역 로거설정 */
    set_global_logger();
    
    loop {
        
        // let receiver_email_list: ReceiverEmailConfig = match read_toml_from_file::<ReceiverEmailConfig>("./config/system_config.toml") {
        //     Ok(receiver_email_list) => receiver_email_list,
        //     Err(e) => {
        //         error!("[Error][main()] Failed to retrieve information 'receiver_email_list'. : {:?}", e);
        //         panic!("[Error][main()] Failed to retrieve information 'receiver_email_list'. : {:?}", e);
        //     }
        // };        
        
        /* 확인이 필요한 색인될 인덱스 정보들 */
        let index_schdules: IndexSchedulesConfig = match read_toml_from_file::<IndexSchedulesConfig>("./config/index_list.toml") {
            Ok(index_schdules) => index_schdules,
            Err(e) => {
                error!("{:?}", e);
                panic!("{:?}", e);
            }
        };
        
        let system_config = get_system_config_info();
        let chunk_size = system_config.chunk_size; /* 한 번에 처리할 요소의 수 */ 

        let mut quotient = index_schdules.index().len() / chunk_size;
        let remainder = index_schdules.index().len() % chunk_size;
        if remainder != 0 {
            quotient += 1;
        }
        
        /* Handler 의존주입 */
        let mut handlers: Vec<MainHandler<SmtpServicePub, QueryServicePub, IndexStorageServicePub>> = Vec::new();

        for idx in 0..quotient {
            
            // println!("start: {}", idx*chunk_size);
            // println!("end: {}", idx*chunk_size + chunk_size.min(index_schdules.index.len() - idx*chunk_size) - 1);

            let index_schedule: Vec<IndexSchedules> = index_schdules.index[(idx*chunk_size)..(idx*chunk_size) + chunk_size.min(index_schdules.index.len() - idx*chunk_size)].to_vec();
            
            let query_service: QueryServicePub = QueryServicePub::new();
            let smtp_service: SmtpServicePub = SmtpServicePub::new();
            let index_storage_service: IndexStorageServicePub =  IndexStorageServicePub::new(index_schedule);
            
            let main_handler: MainHandler<SmtpServicePub, QueryServicePub, IndexStorageServicePub> = MainHandler::new(smtp_service, query_service, index_storage_service);
            
            handlers.push(main_handler);
        }
        
        let futures = handlers.iter().map(|handler| {
            async move {
                handler.main_task().await /* 실제 Task */
            }
        });
        
        let results = join_all(futures).await;
        
        for result in results {
            match result {
                Ok(_) => {
                    info!("Program processed successfully");
                }
                Err(e) => {
                    error!("[Error][main()] Error processing template: {:?}", e);
                }
            }
        }  
        






        //let mut handles: Vec<task::JoinHandle<()>> = Vec::new();

        // for i in (0..arc_index_infos.len()).step_by(chunk_size) {

        //     let index_data_clone = Arc::clone(&arc_index_infos);
            
        //     let handle = task::spawn(async move {
        //         let chunk = index_data_clone[i..i + chunk_size.min(index_data_clone.len() - i)].to_vec();
        //         /* 예를 들어 각 청크의 요소를 처리하는 로직 */ 
        //         //println!("chunk: {:?}", chunk);
        //         main_handler.main_task().await
                
        //     });
            
        //     handles.push(handle);
        // }
        
        // for handle in handles {
        //     handle.await.unwrap();
        // }


        // match main_handler.main_task().await {
        //     Ok(_) => (),
        //     Err(e) => {
        //         error!("[Error][main() -> main_handler] {:?}", e);
        //     }
        // };
        
        // let index_schdules: IndexSchedulesConfig = match read_toml_from_file::<IndexSchedulesConfig>("./config/index_list.toml") {
        //     Ok(index_list) => index_list,
        //     Err(e) => {
        //         error!("[Error][main()] Failed to retrieve information 'index_list'. : {:?}", e);
        //         panic!("[Error][main()] Failed to retrieve information 'index_list'. : {:?}", e);
        //     }
        // };  
        
        // let chunk_size = *system_config.system.chunk_size(); /* 한 번에 처리할 요소의 수 */
        // let arc_index_infos: Arc<Vec<IndexSchedules>> = Arc::new(index_schdules.index);
        
        // let mut handles: Vec<task::JoinHandle<()>> = Vec::new();
        
        
        // for i in (0..arc_index_infos.len()).step_by(chunk_size) {

        //     let index_data_clone = Arc::clone(&arc_index_infos);
            
        //     let handle = task::spawn(async move {
        //         let chunk = index_data_clone[i..i + chunk_size.min(index_data_clone.len() - i)].to_vec();
        //         /* 예를 들어 각 청크의 요소를 처리하는 로직 */ 
        //         println!("chunk: {:?}", chunk);
        //     });
            
        //     handles.push(handle);
        // }

        // for handle in handles {
        //     handle.await.unwrap();
        // }


        // // for schedule in index_schdules.index {
            
        // // }

        // let query_service = QueryServicePub::new();
        // let smtp_service = match initialize_smtp_clients("./config/smtp_config.toml","./config/email_receiver_info.toml") {
        //     Ok(smtp_service) => smtp_service,
        //     Err(e) => {
        //         error!("[Error][main()] {:?}", e);
        //         panic!("[Error][main()] {:?}", e);
        //     }
        // };        
        

        // let main_controller = MainHandler::new(smtp_service, query_service);
        
        // match main_controller.main_task().await {
        //     Ok(_) => (),
        //     Err(e) => {
        //         error!("[Error][main()] {:?}", e);
        //     }
        // }        
        
        break;
    }
    
}
