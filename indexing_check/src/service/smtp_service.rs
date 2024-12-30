use crate::common::*;

use crate::model::SmtpConfig::*;
use crate::model::Config::*;
use crate::model::ReceiverEmailConfig::*;
use crate::model::EmailStruct::*;

use crate::utils_modules::io_utils::*;



#[async_trait]
pub trait SmtpService {
    async fn send_message_to_receiver_html(&self, email_id: &str, subject: &str, html_content: &str) -> Result<(), anyhow::Error>;
    async fn send_message_to_receivers(&self, email_subject: &str, send_email_form: EmailStruct, cluster_name: &str) -> Result<(), anyhow::Error>;
} 


#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct SmtpServicePub {
    receiver_email_list: ReceiverEmailConfig
}


impl SmtpServicePub {

    #[doc = "SmtpServicePub 구조체의 생성자"]
    pub fn new() -> Self {
    
        let receiver_email_list: ReceiverEmailConfig = match read_toml_from_file::<ReceiverEmailConfig>("./config/email_receiver_info.toml") {
            Ok(receiver_email_list) => receiver_email_list,
            Err(e) => {
                error!("[Error][main()] Failed to retrieve information 'receiver_email_list'. : {:?}", e);
                panic!("[Error][main()] Failed to retrieve information 'receiver_email_list'. : {:?}", e);
            }
        };   
        
        SmtpServicePub {
            receiver_email_list
        }
    }
}


#[async_trait]
impl SmtpService for SmtpServicePub {

    
    #[doc = "수신자에게 이메일을 보내주는 함수"]
    /// # Arguments
    /// * `email_id`        - 수신자 이메일 주소
    /// * `subject`         - 이메일 제목
    /// * `html_content`    - 이메일 양식 (HTML 양식)
    /// 
    /// # Returns
    /// * Result<(), anyhow::Error>
    async fn send_message_to_receiver_html(&self, email_id: &str, subject: &str, html_content: &str) -> Result<(), anyhow::Error> {

        let smtp_config_info = get_smtp_config_info();

        let email = Message::builder()
            .from(smtp_config_info.credential_id.parse()?)
            .to(email_id.parse().unwrap())
            .subject(subject)
            .multipart(
                MultiPart::alternative() 
                    .singlepart(
                        SinglePart::html(html_content.to_string())
                    )
            )?;
        
        let creds = Credentials::new(
            smtp_config_info.credential_id().to_string(), 
            smtp_config_info.credential_pw().to_string()
        );
                
        let mailer = 
            AsyncSmtpTransport::<lettre::Tokio1Executor>::relay(smtp_config_info.smtp_name().as_str())?
                .credentials(creds)
                .build();
        
        match mailer.send(email).await {
            Ok(_) => Ok(()),
            Err(e) => {
                Err(anyhow!("{:?} : Failed to send email to {} ", e, email_id))
            }
        }
    }  
    
    

    #[doc = "지정된 수신자 모두에게 이메일을 보내주는 함수"]
    /// # Arguments
    /// * `email_subject`   - 이메일 제목
    /// * `send_email_form` - 이메일 양식 (HTML 양식)
    /// * `cluster_name`    - Elasticsearch Cluster 이름
    /// 
    /// # Returns
    /// * Result<(), anyhow::Error>
    async fn send_message_to_receivers(&self, email_subject: &str, send_email_form: EmailStruct, cluster_name: &str) -> Result<(), anyhow::Error> {
        
        let receiver_email_list = &self.receiver_email_list().emails;
        
        let html_template = std::fs::read_to_string("./html/view.html")?;        
        
        let html_content = html_template
            .replace("{cluster_name}", cluster_name)
            .replace("{index_list}", &send_email_form.html_form);
        
        /* Not Async */                
        // for receiver in receiver_email_list {
        //     let email_id = receiver.email_id();
        //     self.send_message_to_receiver_html(email_id.as_str(), "[Elasticsearch] Index removed list", &html_content).await?;
        // }
        
        /* ASYNC TASK */
        let tasks = receiver_email_list
            .iter()
            .map(|receiver| {
                let email_id = receiver.email_id();
                self.send_message_to_receiver_html(email_id.as_str(), email_subject, &html_content)
            });

        let results = join_all(tasks).await;

        for result in results {
            match result {
                Ok(_) => info!("Email sent successfully"),
                Err(e) => error!("[Error][send_message_to_receivers()] Failed to send email: {}", e),
            }
        }
        
        Ok(())
    } 
}