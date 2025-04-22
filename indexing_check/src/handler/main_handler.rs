use crate::common::*;

use crate::model::error_alarm_info::*;
use crate::model::vector_index_log_format::VectorIndexLogFormat;
use crate::service::query_service::*;
use crate::service::smtp_service::*;
use crate::service::telegram_service::*;

use crate::model::code_config::*;
use crate::model::index_schedules_config::*;
use crate::model::system_config::*;
use crate::model::total_config::*;
use crate::model::vector_index_log::*;
use crate::model::error_alram_info_format::*;

use crate::utils_modules::time_utils::*;

pub struct MainHandler<S: SmtpService, Q: QueryService, T: TelegramService> {
    smtp_service: S,
    query_service: Q,
    telegram_service: T,
}

impl<S: SmtpService, Q: QueryService, T: TelegramService> MainHandler<S, Q, T> {
    pub fn new(smtp_service: S, query_service: Q, telegram_service: T) -> Self {
        Self {
            smtp_service,
            query_service,
            telegram_service,
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
        let schedule: Schedule =
            Schedule::from_str(&index_schedule.time).expect("Failed to parse CRON expression");
        let schedule_term: Arc<SystemConfig> = get_system_config_info();
        let mut interval: Interval = tokio::time::interval(tokio::time::Duration::from_millis(
            schedule_term.schedule_term,
        ));

        /* 한국 표준시 GMT + 9 */
        let kst_offset: FixedOffset = match FixedOffset::east_opt(9 * 3600) {
            Some(kst_offset) => kst_offset,
            None => {
                error!(
                    "[Error][main_schedule_task()] There was a problem initializing 'kst_offset'."
                );
                panic!(
                    "[Error][main_schedule_task()] There was a problem initializing 'kst_offset'."
                );
            }
        };

        loop {
            interval.tick().await;

            let now: DateTime<Utc> = Utc::now();
            let kst_now: DateTime<FixedOffset> = now.with_timezone(&kst_offset); /* Converting UTC Current Time to KST */

            if let Some(next) = schedule.upcoming(kst_offset).take(1).next() {
                if (next - kst_now).num_seconds() < 1 {
                    match self.main_task(index_schedule.clone()).await {
                        Ok(_) => (),
                        Err(e) => {
                            error!("[Error][main_schedule_task() -> main_task()] {:?}", e);
                        }
                    }
                }
            }
        }
    }

    #[doc = "인덱스 정적 색인 작업 확인 함수"]
    /// # Arguments
    /// * `index_schedule` - 인덱스 스케쥴 객체
    ///
    /// # Returns
    /// * Result<(), anyhow::Error>
    pub async fn main_task(&self, index_schedule: IndexSchedules) -> Result<(), anyhow::Error> {
        info!("main task start: {}", index_schedule.index_name());
        /* 탐색할 인덱스 이름을 가져온다 -> UTC 시간 기준으로 이름이 맵핑된다. */
        let code_config: Arc<CodeConfig> = get_code_config_info();

        let search_index_name: String = if code_config.code_type() == "prod" {
            let curr_date_utc: String = get_current_utc_naivedate_str("%Y-%m-%d")?;
            let search_index: Arc<SystemConfig> = get_system_config_info();
            format!("{}-{}", search_index.log_index_name(), curr_date_utc)
        } else {
            String::from("vector-indexing-logs-2025-01-08")
        };

        /* 현재시간 - utc 시간 */
        let curr_time_utc: NaiveDateTime = get_currnet_utc_naivedatetime();
        /* 색인 동작시간 */
        let time_minutes_ago: NaiveDateTime =
            curr_time_utc - chrono::Duration::seconds(index_schedule.duration);

        /* 색인 로그 확인 -> ES 쿼리 */
        let vector_index_logs: Vec<VectorIndexLogFormat> = self
            .query_service
            .get_indexing_movement_log(
                &search_index_name,
                index_schedule.index_name(),
                index_schedule.indexing_type(),
                time_minutes_ago,
                curr_time_utc,
            )
            .await?;

        let mut index_succ_flag: bool = false; /* 색인 완전 실패 유무 */
        let mut cnt_succ_flag: bool = false; /* 색인 부분 실패 유무 -> 건수가 지정값 미만인 경우 */
        let mut indexing_cnt_num: usize = 0; /* 색인된 문서 수 -> 기본적으로는 0개 */

        for vector_index_log in vector_index_logs {
            let log_message: &String = vector_index_log.vector_index_log.message();

            /* 정상적으로 색인이 되었을 경우에는 `index worked` 라는 문자열이 포함되어 있다. */
            if log_message.contains("index worked") {
                index_succ_flag = true;
            }

            let regex: Regex = Regex::new(r"\((.*?)\)")?;

            if let Some(caps) = regex.captures(log_message) {
                let indexing_cnt = match caps.get(0) {
                    Some(indexing_cnt) => indexing_cnt.as_str(),
                    None => "(0)",
                };

                let caps_trim: &str = &indexing_cnt[1..indexing_cnt.len() - 1];

                let indexing_cnt_replace = caps_trim.replace(",", "");
                indexing_cnt_num = indexing_cnt_replace.parse()?;

                /* 지정한 색인 문서 수 이상인 경우는 색인에 문제없다고 판단함. */
                if indexing_cnt_num >= index_schedule.size {
                    cnt_succ_flag = true;
                }
            }
        }

        let system_config: Arc<SystemConfig> = get_system_config_info();
        let err_monitor_index: String = system_config.err_monitor_index().to_string();

        if !index_succ_flag {
            /* 색인 자체가 실패가 난 경우. */
            let mut error_alarm_info: ErrorAlarmInfo = ErrorAlarmInfo::new(
                String::from(""),
                String::from("Full Error"),
                index_schedule.index_name().to_string(),
                index_schedule.indexing_type().to_string(),
                indexing_cnt_num,
                *index_schedule.size(),
            );

            /* Elasticsearch 로그 인덱스로 실패건 전송 */
            self.query_service
                .post_indexing_error_info(&err_monitor_index, &mut error_alarm_info)
                .await?;
        } else if !cnt_succ_flag {
            /* 색인은 성공했지만, 색인 개수가 올바르지 않은 경우. */
            let mut error_alarm_info: ErrorAlarmInfo = ErrorAlarmInfo::new(
                String::from(""),
                String::from("Partial Error"),
                index_schedule.index_name().to_string(),
                index_schedule.indexing_type().to_string(),
                indexing_cnt_num,
                *index_schedule.size(),
            );

            /* Elasticsearch 로그 인덱스로 실패건 전송 */
            self.query_service
                .post_indexing_error_info(&err_monitor_index, &mut error_alarm_info)
                .await?;
        } else {
            /* 색인에 문제가 없는 경우 */
            info!(
                "{} index completed.: {:?}",
                index_schedule.index_name(),
                index_schedule
            );
        }

        Ok(())
    }   
    
    #[doc = "알람관련 로직을 실행하는 함수 -> Telegram 메시지 발송 및 이메일 발송"]
    pub async fn alarm_task(&self) -> Result<(), anyhow::Error> {
        info!("alarm task start");

        let system_config: Arc<SystemConfig> = get_system_config_info();
        let err_monitor_index: String = system_config.err_monitor_index().to_string();

        /* Error 관련 인덱스를 조회한다. */
        let error_alaram_infos: Vec<ErrorAlarmInfoFormat> = self
            .query_service
            .get_error_alarm_infos(&err_monitor_index)
            .await?;

        if !error_alaram_infos.is_empty() {

            /* Telegram && 이메일 알람 전송*/
            /* 1. Telegram 전송 */
            self.telegram_service
                .send_indexing_failed_msg(&error_alaram_infos)
                .await?;

            // /* 2. Email 전송 */
            self.smtp_service
                .send_message_to_receivers(&error_alaram_infos)
                .await?;

            for alarm in error_alaram_infos {
                let error_alram_info: &ErrorAlarmInfo = alarm.error_alram_info();
                let index_type: &str = error_alram_info.index_type();

                /* 증분색인인 경우에는 한번 알람 주고 제거 */
                if index_type == "dynamic index" {
                    
                }
            }
        }

        Ok(())
    }
}
