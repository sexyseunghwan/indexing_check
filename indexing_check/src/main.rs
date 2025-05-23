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
*/
mod common;
use common::*;

mod utils_modules;

use tokio::time;
use utils_modules::io_utils::*;
use utils_modules::logger_utils::*;

mod model;
use model::index_schedules_config::*;

mod handler;
use handler::main_handler::*;

mod repository;

mod service;
use service::query_service::*;
use service::smtp_service::*;
use service::telegram_service::*;

mod env_configuration;
use env_configuration::env_config::*;

#[tokio::main]
async fn main() {
    /* 전역 로거설정 */
    set_global_logger();

    info!("Program start!");

    let smtp_service: SmtpServicePub = SmtpServicePub::new();
    let query_service: QueryServicePub = QueryServicePub::new();
    let telegram_service: TelegramServicePub = TelegramServicePub::new();

    let handler_arc: Arc<MainHandler<SmtpServicePub, QueryServicePub, TelegramServicePub>> =
        Arc::new(MainHandler::new(
            smtp_service,
            query_service,
            telegram_service,
        ));

    let alarm_handler: Arc<MainHandler<SmtpServicePub, QueryServicePub, TelegramServicePub>> =
        Arc::clone(&handler_arc);
    
    /* 알람 테스크 */
    tokio::spawn(async move {
        let mut other_interval: Interval = time::interval(Duration::from_secs(60));

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
                error!("{:?}", e);
                panic!("{:?}", e);
            }
        };

    /*
        각 인덱스 별로 모니터링을 비동기적으로 실시해준다.
        스케쥴링 대기 작업 진행
    */
    for index in index_schdules.index {
        let index_clone: IndexSchedules = index.clone();

        let handler_arc_clone: Arc<
            MainHandler<SmtpServicePub, QueryServicePub, TelegramServicePub>,
        > = Arc::clone(&handler_arc);

        tokio::spawn(async move {
            if let Err(e) = handler_arc_clone.main_schedule_task(index_clone).await {
                error!("[Error][main_schedule_task] {:?}", e);
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

//#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
// fn main() {
//     let runtime: tokio::runtime::Runtime = tokio::runtime::Builder::new_multi_thread()
//         .worker_threads(4) // 4개의 스레드 사용
//         .thread_name_fn(|| {
//             static ATOMIC_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
//             let id = ATOMIC_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
//             format!("custom-worker-{}", id) // 스레드 이름을 고유하게 설정
//         })
//         .enable_all()
//         .build()
//         .unwrap();

//     runtime.block_on(async_main()); // 런타임 실행
// }
