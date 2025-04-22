use crate::common::*;

use crate::repository::telegram_repository::*;

use crate::model::error_alarm_info::*;
use crate::model::error_alram_info_format::*;
use crate::model::system_config::*;
use crate::model::total_config::*;

#[async_trait]
pub trait TelegramService {
    async fn send_indexing_failed_msg(
        &self,
        error_alaram_infos: &Vec<ErrorAlarmInfoFormat>,
    ) -> Result<(), anyhow::Error>;
    fn get_error_clasification(
        &self,
        error_alaram_infos: &ErrorAlarmInfo,
        err_alram_map: &mut HashMap<String, Vec<String>>,
    ) -> Result<(), anyhow::Error>;
}

#[derive(Debug, new)]
pub struct TelegramServicePub {}

#[async_trait]
impl TelegramService for TelegramServicePub {
    #[doc = "색인이 실패했을 때, Telegram bot 을 통해서 알람을 보내주는 함수"]
    /// # Arguments
    /// * `error_alaram_infos` - 실패한 색인 정보들
    ///
    /// # Returns
    /// * Result<(), anyhow::Error>
    async fn send_indexing_failed_msg(
        &self,
        error_alaram_infos: &Vec<ErrorAlarmInfoFormat>,
    ) -> Result<(), anyhow::Error> {
        let tele_repo: Arc<TelebotRepositoryPub> = get_telegram_repo();

        let system_config: Arc<SystemConfig> = get_system_config_info();
        let msg_chunk_size: usize = *system_config.message_chunk_size();

        let mut err_alram_map: HashMap<String, Vec<String>> = HashMap::new();

        for (_i, chunk) in error_alaram_infos.chunks(msg_chunk_size).enumerate() {
            for item in chunk {
                self.get_error_clasification(item.error_alram_info(), &mut err_alram_map)?;
            }

            let mut msg_format: String = String::from("[Elasticsearch Indexing Error!]\n");

            for (key, value) in err_alram_map {
                let error_type: String = key;
                let error_map: Vec<String> = value;

                msg_format.push_str(format!("[{}]\n", error_type).as_str());

                for err_msg in error_map {
                    msg_format.push_str(format!("{}\n", err_msg).as_str());
                }
            }

            /* Send Message */
            tele_repo.bot_send(&msg_format).await?;

            err_alram_map = HashMap::new(); /* Clear HashMap */
        }

        Ok(())
    }

    #[doc = "색인 실패별 로그들을 완전실패/부분실패로 나눠주는 함수"]
    /// # Arguments
    /// * `error_alaram_infos` - 실패한 색인 정보
    ///
    /// # Returns
    /// * Result<(), anyhow::Error>
    fn get_error_clasification(
        &self,
        error_alaram_info: &ErrorAlarmInfo,
        err_alram_map: &mut HashMap<String, Vec<String>>,
    ) -> Result<(), anyhow::Error> {
        let mut send_msg: String = String::from("");
        send_msg.push_str(&format!(
            " index name: {}\n",
            error_alaram_info.index_name()
        ));

        send_msg.push_str(&format!(
            "   - indexing type: {}\n",
            error_alaram_info.index_type()
        ));

        let key_name: String = if error_alaram_info.error_type() == "Full Error" {
            String::from("Full Error")
        } else {
            send_msg.push_str(&format!(
                "   - index cnt (declare cnt): {} ({})\n",
                error_alaram_info
                    .indexing_cnt_num
                    .to_formatted_string(&Locale::en),
                error_alaram_info
                    .declare_index_size
                    .to_formatted_string(&Locale::en)
            ));
            String::from("Partial Error")
        };

        err_alram_map
            .entry(key_name.clone())
            .or_insert_with(Vec::new)
            .push(send_msg);

        Ok(())
    }
}
