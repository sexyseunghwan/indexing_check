
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
use utils_modules::logger_utils::*;

mod model;

mod controller;
use controller::main_controller::*;

mod repository;

mod service;
use service::query_service::*;
use service::smtp_service::*;

#[tokio::main]
async fn main() {
    
    /* 전역 로거설정 */
    set_global_logger();
    
    loop {

        let query_service = QueryServicePub::new();
        let smtp_service = match initialize_smtp_clients("./config/smtp_config.toml","./config/email_receiver_info.toml") {
            Ok(smtp_service) => smtp_service,
            Err(e) => {
                error!("[Error][main()] {:?}", e);
                panic!("[Error][main()] {:?}", e);
            }
        };        
        
        let main_controller = MainController::new(smtp_service, query_service);
        
        match main_controller.main_task().await {
            Ok(_) => (),
            Err(e) => {
                error!("[Error][main()] {:?}", e);
            }
        }        

        break;
    }
    
}
