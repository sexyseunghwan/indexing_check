
/*
Author      : Seunghwan Shin 
Create date : 2024-12-30 
Description : 색인이 완벽하게 끝났는지 확인해주는 프로그램.

History     : 2024-12-30 Seunghwan Shin       # [v.1.0.0] first create
*/
mod common;
use core::panic;

use common::*;

mod utils_modules;
use utils_modules::logger_utils::*;
use utils_modules::io_utils::*;

mod model;
use model::IndexSchedulesConfig::*;
use model::Config::*;

mod handler;
use handler::main_handler::*;

mod repository;

mod service;
use service::query_service::*;
use service::smtp_service::*;
use service::index_storage_service::*;
use service::telegram_service::*;


#[doc = "메인 스케쥴러 함수"]
/// # Arguments
/// * `index_schedule` - 인덱스 스케쥴 객체
/// 
/// # Returns
/// * Result<(), anyhow::Error>
async fn main_schedule_task(index_schedule: IndexSchedules) -> Result<(), anyhow::Error> {
    
    let schedule = Schedule::from_str(&index_schedule.time).expect("Failed to parse CRON expression");
    let schedule_term = get_system_config_info();
    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(schedule_term.schedule_term));
     
    let query_service: QueryServicePub = QueryServicePub::new();
    let smtp_service: SmtpServicePub = SmtpServicePub::new();
    let index_storage_service: IndexStorageServicePub =  IndexStorageServicePub::new(index_schedule);
    let telegram_service: TelegramServicePub = TelegramServicePub::new();

    let main_handler: MainHandler<SmtpServicePub, QueryServicePub, IndexStorageServicePub, TelegramServicePub> = MainHandler::new(smtp_service, query_service, index_storage_service, telegram_service);
    
    /* 한국 표준시 GMT + 9 */ 
    let kst_offset = match FixedOffset::east_opt(9 * 3600) {
        Some(kst_offset) => kst_offset,
        None => {
            error!("[Error][main_schedule_task()] There was a problem initializing 'kst_offset'.");
            panic!("[Error][main_schedule_task()] There was a problem initializing 'kst_offset'.");
        }
    };
    
    loop {
        
        interval.tick().await;
        
        let now: DateTime<Utc> = Utc::now();
        let kst_now = now.with_timezone(&kst_offset); /* Converting UTC Current Time to KST */ 
        
        if let Some(next) = schedule.upcoming(kst_offset).take(1).next() {
            if (next - kst_now).num_seconds() < 1 { 
                match main_handler.main_task().await {
                    Ok(_) => (),
                    Err(e) => {
                        error!("[Error][main_schedule_task()] {:?}", e);
                    }
                }
            }
        }
    }
}


#[tokio::main]
async fn main() {
    
    /* 전역 로거설정 */
    set_global_logger();

    info!("Program start!");
    
    /* 모니터링 대상이 되는 색인될 인덱스 정보들 */
    let index_schdules: IndexSchedulesConfig = match read_toml_from_file::<IndexSchedulesConfig>("./config/index_list.toml") {
        Ok(index_schdules) => index_schdules,
        Err(e) => {
            error!("{:?}", e);
            panic!("{:?}", e);
        }
    };
    
    /* 각 인덱스 별로 모니터링을 비동기적으로 실시해준다. */
    let tasks: Vec<_> = 
        index_schdules.index
            .iter()
            .map(|index: &IndexSchedules| {
                let task = main_schedule_task(index.clone()); /* 실제 동작함수 */
                tokio::spawn(task)
            })
            .collect();
    
    for task in tasks {
        match task.await {
            Ok(result) => match result {
                Ok(_) => (),
                Err(e) => error!("[Error][main()] {:?}", e),
            },
            Err(e) => println!("Task panicked: {:?}", e),
        }
    }
}
