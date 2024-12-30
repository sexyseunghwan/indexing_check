use crate::common::*;

use crate::repository::telegram_repository::*;


#[async_trait]
pub trait TelegramService {
    async fn send_indexing_total_failed_msg(&self, index_name: &str) -> Result<(), anyhow::Error>;
    async fn send_indexing_cnt_failed_msg(&self, index_name: &str, index_cnt: usize, declare_index_cnt: usize) -> Result<(), anyhow::Error>;
}

#[derive(Debug, new)]
pub struct TelegramServicePub {}


#[async_trait]
impl TelegramService for TelegramServicePub {
    
    #[doc = "색인에 실패한 경우에 실패한 인덱스 이름과 이유를 Telegram 메시지로 보내주는 함수"]
    /// # Arguments
    /// * `index_name` - 색인에 실패한 인덱스 이름
    /// 
    /// # Returns
    /// * Result<(), anyhow::Error>
    async fn send_indexing_total_failed_msg(&self, index_name: &str) -> Result<(), anyhow::Error> {
        
        let tele_repo = get_telegram_repo();
        
        let mut send_msg = String::from("[Elasticsearch Indexing Error!]\n");
        send_msg.push_str("Index failed.\n\n");
        send_msg.push_str(&format!(" - index name: {}\n", index_name));
        
        tele_repo.bot_send(&send_msg).await?;

        Ok(())
    }
    
    #[doc = ""]
    /// # Arguments
    /// * `index_name` - 색인에 실패한 인덱스 이름
    /// * `index_cnt` - 색인된 문서의 수
    /// * `declare_index_cnt` - 색인 예상 문서 수
    /// 
    /// # Returns
    /// * Result<(), anyhow::Error>
    async fn send_indexing_cnt_failed_msg(&self, index_name: &str, index_cnt: usize, declare_index_cnt: usize) -> Result<(), anyhow::Error> {
        
        let tele_repo = get_telegram_repo();

        let mut send_msg = String::from("[Elasticsearch Indexing Error!]\n");
        send_msg.push_str("Indexing is successful, but there are insufficient number of indexed documents.\n\n");
        send_msg.push_str(&format!(" - index name: {}\n", index_name));
        send_msg.push_str(&format!(" - index cnt (declare cnt): {} ({})\n", 
            index_cnt.to_formatted_string(&Locale::en), 
            declare_index_cnt.to_formatted_string(&Locale::en)));

        tele_repo.bot_send(&send_msg).await?;

        Ok(())
    }
    
}