/*
Author      : Seunghwan Shin
Create date : 2024-12-30
Description : 색인이 완벽하게 끝났는지 확인해주는 프로그램.

History     : 2024-12-30 Seunghwan Shin       # [v.1.0.0] first create
              2025-02-07 Seunghwan Shin       # [v.1.1.0]
                                                1) 색인 실패가 발생하였을 때, 알람을 계속 울릴 수 있도록 로직 변경.
                                                2) .env 파일사용으로 경로변경을 쉽게 할 수 있도록 변경
                                                3) 전체적인 알람구조 형식 변경
              2025-04-22 Seunghwan Shin       # [v.1.2.0] 증분색인 실패시에는 한번만 알람 보내주는 로직으로 변경
              2025-05-16 Seunghwan Shin       # [v.1.3.0] Elasticsearch connection Pool을 Mutex -> Semaphore 로 변경
              2025-08-05 Seunghwan Shin       # [v.2.0.0]
                                                1) 코드 구조 변경 및 smtp -> imailer 로 변환
                                                2) Elasticsearch 비밀번호 url 인코딩 처리 추가
              2025-09-11 Seunghwan Shin       # [v.2.1.0] 코드 리팩토링
*/
mod prelude;
mod external_deps;
mod common;
use common::*;

mod utils_modules;

use utils_modules::io_utils::*;
use utils_modules::logger_utils::*;

mod model;
use model::index_schedules_config::*;

mod handler;
use handler::main_handler::*;

mod repository;

mod service;
use service::{notification_service::*, query_service::*};

mod env_configuration;
use env_configuration::env_config::*;

mod traits;

#[tokio::main]
async fn main() {
    /* 전역 로거설정 */
    dotenv().ok();
    set_global_logger();

    info!("Program start!");

    let query_service: QueryServicePub = QueryServicePub::new();
    let notification_service: NotificationServicePub = NotificationServicePub::new();

    let handler_arc: Arc<MainHandler<NotificationServicePub, QueryServicePub>> =
        Arc::new(MainHandler::new(notification_service, query_service));

    let alarm_handler: Arc<MainHandler<NotificationServicePub, QueryServicePub>> =
        Arc::clone(&handler_arc);
    
    /* 알람 테스크 */
    tokio::spawn(async move {
        let mut other_interval: Interval = tokio::time::interval(Duration::from_secs(60));

        loop {
            other_interval.tick().await;

            match alarm_handler.alarm_task().await {
                Ok(_) => (),
                Err(e) => {
                    error!("[Error][main() -> alarm_task()] {:?}", e);
                }
            }
        }
    });

    /* 모니터링 대상이 되는 색인될 인덱스 정보들 */
    let index_schdules: IndexSchedulesConfig =
        match read_toml_from_file::<IndexSchedulesConfig>(&INDEX_LIST_PATH) {
            Ok(index_schdules) => index_schdules,
            Err(e) => {
                error!("[Error][Failed to load index schedules config] {:?}", e);
                panic!("[Fatal] Cannot continue without valid configuration: {:?}", e);
            }
        };
    
    /*
        각 인덱스 별로 모니터링을 비동기적으로 실시해준다.
        스케쥴링 대기 작업 진행
    */
    for index in index_schdules.index {
        let index_clone: IndexSchedules = index.clone();

        let handler_arc_clone: Arc<MainHandler<NotificationServicePub, QueryServicePub>> =
            Arc::clone(&handler_arc);

        tokio::spawn(async move {
            if let Err(e) = handler_arc_clone.main_schedule_task(index_clone.clone()).await {
                error!("[Error][main_schedule_task][{}] {:?}", index_clone.index_name, e);
            }
        });
    }

    /* 모두 서브테스크로 실행되므로 아래와 같이 메인 태스크를 계속 유지시켜줘야 한다. */
    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("Received Ctrl+C, shutting down...");
        }
    }
}