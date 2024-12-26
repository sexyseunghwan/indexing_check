
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


use std::str::FromStr;

async fn async_task(task_name: &str) {
    println!("Executing async task: {}", task_name);
    // 여기에 실제 비동기 로직을 구현하거나 외부 API 호출 등을 포함할 수 있습니다.
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await; // 시뮬레이션을 위한 1초 대기
}

async fn schedule_task(cron_expression: &str, task_name: &'static str) {
    let schedule = Schedule::from_str(cron_expression).expect("Failed to parse CRON expression");
    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(500)); // 0.5초 간격으로 검사

    loop {
        interval.tick().await;
        let now = Utc::now();
        if let Some(next) = schedule.upcoming(Utc).take(1).next() {
            if (next - now).num_seconds() < 1 {
                println!("Running task: {} at {:?}", task_name, next);
                async_task(task_name).await; // 비동기 작업 실행
            }
        }
    }
}



#[tokio::main]
async fn main() {
    
    /* 전역 로거설정 */
    set_global_logger();
    
    let cron_jobs = vec![
        ("* 10,26,30 6-23,0-4 * * * *", "Task 1"),
        ("* 59 * * * * *", "Task 2"),
    ];

    let tasks: Vec<_> = cron_jobs
        .iter()
        .map(|(expression, name)| {
            let task = schedule_task(expression, name);
            tokio::spawn(task)
        })
        .collect();

    for task in tasks {
        let _ = task.await;
    }


    
    // loop {
                
    //     /* 확인이 필요한 색인될 인덱스 정보들 */
    //     let index_schdules: IndexSchedulesConfig = match read_toml_from_file::<IndexSchedulesConfig>("./config/index_list.toml") {
    //         Ok(index_schdules) => index_schdules,
    //         Err(e) => {
    //             error!("{:?}", e);
    //             panic!("{:?}", e);
    //         }
    //     };
        
    //     let system_config = get_system_config_info();
    //     let chunk_size = system_config.chunk_size; /* 한 번에 처리할 요소의 수 */ 

    //     let mut quotient = index_schdules.index().len() / chunk_size;
    //     let remainder = index_schdules.index().len() % chunk_size;
    //     if remainder != 0 {
    //         quotient += 1;
    //     }
        
    //     /* Handler 의존주입 */
    //     let mut handlers: Vec<MainHandler<SmtpServicePub, QueryServicePub, IndexStorageServicePub>> = Vec::new();

    //     for idx in 0..quotient {

    //         let index_schedule: Vec<IndexSchedules> = index_schdules.index[(idx*chunk_size)..(idx*chunk_size) + chunk_size.min(index_schdules.index.len() - idx*chunk_size)].to_vec();
            
    //         let query_service: QueryServicePub = QueryServicePub::new();
    //         let smtp_service: SmtpServicePub = SmtpServicePub::new();
    //         let index_storage_service: IndexStorageServicePub =  IndexStorageServicePub::new(index_schedule);
            
    //         let main_handler: MainHandler<SmtpServicePub, QueryServicePub, IndexStorageServicePub> = MainHandler::new(smtp_service, query_service, index_storage_service);
            
    //         handlers.push(main_handler);
    //     }
        
    //     let futures = handlers.iter().map(|handler| {
    //         async move {
    //             handler.main_task().await /* 실제 Task */
    //         }
    //     });
        
    //     let results = join_all(futures).await;
        
    //     for result in results {
    //         match result {
    //             Ok(_) => {
    //                 info!("Program processed successfully");
    //             }
    //             Err(e) => {
    //                 error!("[Error][main()] Error processing template: {:?}", e);
    //             }
    //         }
    //     }  
        
    //     break;
    // }
    
}
