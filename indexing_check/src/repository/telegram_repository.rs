use crate::common::*;

use crate::model::total_config::*;

use crate::traits::repository_traits::telegram_repository_trait::*;

#[doc = "전역 Telebot 인스턴스를 선언"]
static TELEGRAM_REPO: once_lazy<Arc<TelebotRepositoryPub>> =
    once_lazy::new(initialize_tele_bot_client);

#[doc = "Telebot 을 전역적으로 초기화 함."]
pub fn initialize_tele_bot_client() -> Arc<TelebotRepositoryPub> {
    info!("initialize_tele_bot_client() START!");

    let telegram_config: Arc<crate::model::telegram_config::TelegramConfig> =
        get_telegram_config_info();
    let bot_token: &String = telegram_config.bot_token();
    let chat_room_id: &String = telegram_config.chat_room_id();

    let tele_repo: TelebotRepositoryPub =
        TelebotRepositoryPub::new(bot_token.clone(), chat_room_id.clone());

    Arc::new(tele_repo)
}

#[doc = "TelebotService 를 Thread-safe 하게 이용하는 함수."]
pub fn get_telegram_repo() -> Arc<TelebotRepositoryPub> {
    Arc::clone(&TELEGRAM_REPO)
}

/* TelebotService는 비즈니스 로직을 담당하는 서비스 레이어로 분리 */
#[derive(Clone, Debug, Deserialize, Serialize, new)]
pub struct TelebotRepositoryPub {
    pub bot_token: String,
    pub chat_room_id: String,
}

#[async_trait]
impl TelebotRepository for TelebotRepositoryPub {
    #[doc = "Telegram bot 이 메시지를 보내주는 기능 -> 3번 실패 시 에러발생"]
    /// # Arguments
    /// * `send_msg` - Telegram 을 통해서 보내줄 메시지
    ///
    /// # Returns
    /// * Result<(), anyhow::Error>
    async fn bot_send(&self, send_msg: &str) -> Result<(), anyhow::Error> {
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.bot_token);

        let body = serde_json::json!({
            "chat_id": self.chat_room_id,
            "text": send_msg
        });

        let client: Client = Client::new();

        /* 최대 3번의 시도를 수행 */
        for try_cnt in 0..3 {
            match self.try_send(&client, &url, &body).await {
                Ok(_) => {
                    info!("Successfully sent Telegram message");
                    return Ok(());
                }
                Err(err) => {
                    error!(
                       "[Timeout Error][bot_send()] Attempt {} failed: {}. Retrying in 40 seconds.",
                       try_cnt + 1,
                       err
                   );
                    sleep(Duration::from_secs(40)).await;
                }
            }
        }

        Err(anyhow!("[Timeout Error][bot_send()] Failed to send message after 3 attempts to the Telegram bot."))
    }

    #[doc = "메시지를 직접 보내주는 함수"]
    /// # Arguments
    /// * `client` - Telegram 메시지 통신을 위한 클라이언트
    /// * `url` - Telegram bot 을 통해서 메시지를 보내줄 url 정보
    /// * `body` - Telegram Bot 에 대한 상세정보: chat_id, 메시지
    ///
    /// # Returns
    /// * Result<(), anyhow::Error>
    async fn try_send(
        &self,
        client: &reqwest::Client,
        url: &str,
        body: &Value,
    ) -> Result<(), anyhow::Error> {
        let res = client
            .post(url)
            .header("Content-Type", "application/json")
            .body(body.to_string())
            .send()
            .await?;

        if res.status().is_success() {
            Ok(())
        } else {
            let err_text = res
                .text()
                .await
                .unwrap_or_else(|_| "Failed to retrieve error message".to_string());
            Err(anyhow!("HTTP request failed with status: {:?}", err_text))
        }
    }
}
