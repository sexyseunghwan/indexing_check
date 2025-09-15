use crate::common::*;

use crate::model::{
    code_config::*, error_alarm_info::*, error_alarm_info_format::*, index_schedules_config::*,
    system_config::*, total_config::*, vector_index_log_format::*,
};

use crate::traits::service_traits::{notification_service_trait::*, query_service_trait::*};

use crate::utils_modules::time_utils::*;

static IDX_CNT_RE: once_lazy<Regex> = once_lazy::new(|| {
    Regex::new(r"(?i)worked\s*\((?P<num>[\d,]+)\)").unwrap_or_else(|e| {
        error!(
            "[IDX_CNT_RE] Failed to initialize the `IDX_CNT_RE` regular expression.: {:?}",
            e
        );
        panic!("[IDX_CNT_RE] Failed to initialize the `IDX_CNT_RE` regular expression.")
    })
});

pub struct MainHandler<N: NotificationService, Q: QueryService> {
    notification_service: N,
    query_service: Q,
}

impl<N: NotificationService, Q: QueryService> MainHandler<N, Q> {
    pub fn new(notification_service: N, query_service: Q) -> Self {
        Self {
            notification_service,
            query_service,
        }
    }

    #[doc = "메인 스케쥴러 함수"]
    /// # Arguments
    /// * `index_schedule` - 인덱스 스케쥴 객체
    ///
    /// # Returns
    /// * Result<(), anyhow::Error>
    pub async fn main_schedule_task(
        &self,
        index_schedule: IndexSchedules,
    ) -> Result<(), anyhow::Error> {
        let schedule: Schedule = Schedule::from_str(&index_schedule.time).unwrap_or_else(|e| {
            let err_msg: String = format!(
                "[main_schedule_task] Failed to parse CRON expression: {:?}",
                e
            );
            error!("{}", err_msg);
            panic!("{}", err_msg)
        });

        let schedule_term: Arc<SystemConfig> = get_system_config_info();
        let mut interval: Interval = tokio::time::interval(tokio::time::Duration::from_millis(
            schedule_term.schedule_term,
        ));

        /* 한국 표준시 GMT + 9 */
        let kst_offset: FixedOffset = match FixedOffset::east_opt(9 * 3600) {
            Some(kst_offset) => kst_offset,
            None => {
                error!("[main_schedule_task()] There was a problem initializing 'kst_offset'.");
                panic!("[main_schedule_task()] There was a problem initializing 'kst_offset'.");
            }
        };

        loop {
            /* 설정한 시간대로 tick check -> schedule_term */
            interval.tick().await;

            let now: DateTime<Utc> = Utc::now();
            let kst_now: DateTime<FixedOffset> = now.with_timezone(&kst_offset); /* Converting UTC Current Time to KST */

            /*
                schedule.upcoming(kst_offset).take(1).next() -> 앞으로 실행될 일정 중 **맨 앞(다음 실행 시간)**만 하나 꺼낸다.
            */
            if let Some(next) = schedule.upcoming(kst_offset).take(1).next() {
                /*  현재 시각(kst_now)과 **다음 실행 예정 시각(next)**의 차이를 초 단위로 계산. 그 차이가 1초 미만이면 “지금 실행할 시각에 도달했다”고 판단. */
                if (next - kst_now).num_seconds() < 1 {
                    self.main_task(&index_schedule).await.unwrap_or_else(|e| {
                        error!("[main_schedule_task() -> main_task()] {:?}", e);
                    })
                }
            }
        }
    }

    #[doc = "인덱스 색인 작업 확인 함수"]
    /// # Arguments
    /// * `index_schedule` - 인덱스 스케쥴 객체
    ///
    /// # Returns
    /// * Result<(), anyhow::Error>
    pub async fn main_task(&self, index_schedule: &IndexSchedules) -> Result<(), anyhow::Error> {
        info!("main task start: {}", index_schedule.index_name());

        /* 탐색할 인덱스 이름을 가져온다. */
        let search_index_name: String = self.build_search_index_name()?;

        /* 현재시간, 색인 동작시간 */
        let (curr_time_utc, time_minutes_ago) = calc_time_window(index_schedule.duration);

        let system_config: Arc<SystemConfig> = get_system_config_info();

        /* 색인 로그 확인 -> ES 쿼리 */
        let vector_index_logs: Option<VectorIndexLogFormat> = self
            .query_service
            .get_indexing_movement_log(
                &search_index_name,
                index_schedule.index_name(),
                index_schedule.indexing_type(),
                time_minutes_ago,
                curr_time_utc,
            )
            .await
            .ok();

        if let Some(log) = vector_index_logs {
            /* 색인자체는 성공한 경우 */
            self.handle_indexing_success(&log, &system_config, index_schedule)
                .await?;
        } else {
            /* 색인 자체가 실패가 난 경우. */
            self.handle_indexing_failure(&system_config, index_schedule)
                .await?;
        }

        Ok(())
    }

    #[doc = "알람관련 로직을 실행하는 함수 -> Telegram 메시지 발송 및 이메일 발송"]
    pub async fn alarm_task(&self) -> Result<(), anyhow::Error> {
        info!("alarm task start");

        let system_config: Arc<SystemConfig> = get_system_config_info();
        let err_monitor_index: String = system_config.err_monitor_index().to_string();

        /* Error 관련 인덱스를 조회한다. */
        let error_alarm_infos: Vec<ErrorAlarmInfoFormat> = self
            .query_service
            .get_error_alarm_infos(&err_monitor_index)
            .await?;
        
        if error_alarm_infos.is_empty() {
            info!("No indexing failures");
        } else {

            let send_fut = self.send_error_notifications(&error_alarm_infos);
            let cleanup_fut = self.cleanup_dynamic_index_docs(&err_monitor_index, &error_alarm_infos);

            tokio::try_join!(send_fut, cleanup_fut)?;
        }
        //if !error_alaram_infos.is_empty() {
            /* 알람 내역이 있을 경우 -> 알람 보내주기 */
            

            /* 동적색인인 경우에는 기록 지워주기 */

            // self.notification_service
            //     .send_message_to_receivers(&error_alaram_infos)
            //     .await
            //     .unwrap_or_else(|e| error!("[MainHandler->alarm_task] {:?}", e));
            
            // self.notification_service -> 여기서 실행이 문제가 생겨서 그런거 같은데...? 병렬 처리 해야 될거같은데..?
            
            // for alarm in error_alaram_infos {
            //     let error_alarm_info: &ErrorAlarmInfo = alarm.error_alarm_info();
            //     let index_type: &str = error_alarm_info.index_type();

            //     /* 증분색인인 경우에는 한번 알람 주고 제거 */
            //     if index_type == "dynamic index" {
            //         match self.query_service
            //             .delete_index_by_doc(&err_monitor_index, alarm.doc_id())
            //             .await {
            //                 Ok(_) => {
            //                     info!("The document with ID {} has been successfully deleted", alarm.doc_id());
            //                 },
            //                 Err(e) => {
            //                     error!("[MainHandler->alarm_task] {:?}", e);
            //                 }
            //             }
            //     }
            // }
        //}

        Ok(())
    }

    #[doc = "prod / test 여부에 따라 검색 인덱스명 구성"]
    fn build_search_index_name(&self) -> Result<String, anyhow::Error> {
        /* 현재 프로그램실행 type -> prod type 인지 아닌지 체크 */
        let code_config: Arc<CodeConfig> = get_code_config_info();
        
        if code_config.code_type().as_str() == "prod" {
            let curr_date_utc: String =
                get_current_utc_naivedate_str("%Y-%m-%d").unwrap_or_else(|e| {
                    error!(
                        "[MainHandler->build_search_index_name] curr_date_utc error: {:?}",
                        e
                    );
                    panic!(
                        "[MainHandler->build_search_index_name] curr_date_utc error: {:?}",
                        e
                    );
                });
            let search_index: Arc<SystemConfig> = get_system_config_info();
            Ok(format!(
                "{}-{}",
                search_index.log_index_name(),
                curr_date_utc
            ))
        } else {
            Ok("vector-indexing-logs-2025-01-08".to_string())
        }
    }

    #[doc = "색인 로그가 존재하는 경우 처리 (Partial Error or Success)"]
    async fn handle_indexing_success(
        &self,
        log: &VectorIndexLogFormat,
        system_config: &SystemConfig,
        index_schedule: &IndexSchedules,
    ) -> Result<(), anyhow::Error> {
        let log_detail: &str = log.vector_index_log.message().as_str();
        let expected_size: usize = index_schedule.size;

        if let Some(caps) = IDX_CNT_RE.captures(log_detail) {
            if let Some(num) = caps.name("num") {
                let n: usize = num.as_str().replace(',', "").parse::<usize>()?; /* 실제 색인된 문서의 개수 */

                /* 실제 색인된 문서의 개수가 설정한 문서의 개수보다 작은 경우 */
                if n < expected_size {
                    let err_monitor_index: String = system_config.err_monitor_index().to_string();
                    let cur_time_kor_str: String = get_current_kor_naive_datetime_str()?; /* 현재 시각을 문자열로 표시함 */

                    /* 색인은 성공했지만, 색인 개수가 올바르지 않은 경우. */
                    let error_alarm_info: ErrorAlarmInfo = ErrorAlarmInfo::new(
                        cur_time_kor_str,
                        String::from("Partial Error"),
                        index_schedule.index_name().to_string(),
                        index_schedule.indexing_type().to_string(),
                        n,
                        *index_schedule.size(),
                    );

                    self.query_service
                        .post_indexing_error_info(&err_monitor_index, error_alarm_info)
                        .await?;
                } else {
                    /* 색인이 문제없이 잘 된 경우 */
                    info!(
                        "Indexing of `{}({})` completed successfully.",
                        log.vector_index_log.index_name(),
                        log.vector_index_log.state()
                    );
                }
            }
        }

        Ok(())
    }

    #[doc = "색인 로그가 없는 경우 처리(Full Error)"]
    async fn handle_indexing_failure(
        &self,
        system_config: &SystemConfig,
        index_schedule: &IndexSchedules,
    ) -> Result<(), anyhow::Error> {
        /* 색인 자체가 실패가 난 경우. */
        let err_monitor_index: String = system_config.err_monitor_index().to_string();
        let cur_time_kor_str: String = get_current_kor_naive_datetime_str()?; /* 현재 시각을 문자열로 표시함 */

        let error_alarm_info: ErrorAlarmInfo = ErrorAlarmInfo::new(
            cur_time_kor_str,
            String::from("Full Error"),
            index_schedule.index_name().to_string(),
            index_schedule.indexing_type().to_string(),
            0,
            *index_schedule.size(),
        );

        /* Elasticsearch 로그 인덱스로 실패건 전송 */
        self.query_service
            .post_indexing_error_info(&err_monitor_index, error_alarm_info)
            .await?;

        Ok(())
    }   

    #[doc = "알람 내역이 있을 경우 -> 알림을 발송 (실패해도 전체가 멈추지 않게 내부에서 로깅)"]
    async fn send_error_notifications(&self, error_alaram_infos: &[ErrorAlarmInfoFormat]) -> Result<(), anyhow::Error> {

        self.notification_service
                .send_message_to_receivers(error_alaram_infos)
                .await
                .unwrap_or_else(|e| error!("[MainHandler->alarm_task] {:?}", e));

        Ok(())
    } 

    #[doc = "증분색인(dynamic index) 문서 정리: 병렬 삭제(동시 N개), 실패한 건만 로깅"]
    async fn cleanup_dynamic_index_docs(
        &self,
        err_monitor_index: &str,
        infos: &[ErrorAlarmInfoFormat],
    ) -> Result<(), anyhow::Error> {

        let concurrency: usize = 8;

         stream::iter(infos.iter().filter(|a| {
            a.error_alarm_info().index_type() == "dynamic index"
        }))
        .for_each_concurrent(concurrency, |alarm| async move {

            match self.query_service
                .delete_index_by_doc(err_monitor_index, alarm.doc_id())
                .await
            {
                Ok(_) => info!("Deleted doc_id={}", alarm.doc_id()),
                Err(e) => error!("[cleanup_dynamic_index_docs] doc_id={} err={:?}", alarm.doc_id(), e),
            }

        }).await;

        // for alarm in infos {
        //         let error_alarm_info: &ErrorAlarmInfo = alarm.error_alarm_info();
        //         let index_type: &str = error_alarm_info.index_type();

        //         /* 증분색인인 경우에는 한번 알람 주고 제거 */
        //         if index_type == "dynamic index" {
        //             match self.query_service
        //                 .delete_index_by_doc(&err_monitor_index, alarm.doc_id())
        //                 .await {
        //                     Ok(_) => {
        //                         info!("The document with ID {} has been successfully deleted", alarm.doc_id());
        //                     },
        //                     Err(e) => {
        //                         error!("[MainHandler->alarm_task] {:?}", e);
        //                     }
        //                 }
        //         }
        //     }

        Ok(())
    }
}
