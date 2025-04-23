use crate::common::*;

use crate::model::elastic_server_config::*;
use crate::model::error_alram_info_format::ErrorAlarmInfoFormat;
use crate::model::receiver_emai_config::*;
use crate::model::smtp_config::*;
use crate::model::total_config::*;

use crate::utils_modules::io_utils::*;

use crate::env_configuration::env_config::*;

#[async_trait]
pub trait SmtpService {
    async fn send_message_to_receiver_html(
        &self,
        email_id: &str,
        subject: &str,
        html_content: &str,
    ) -> Result<String, anyhow::Error>;
    async fn send_message_to_receivers(
        &self,
        error_alarm_infos: &Vec<ErrorAlarmInfoFormat>,
    ) -> Result<(), anyhow::Error>;
}

#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct SmtpServicePub {
    receiver_email_list: ReceiverEmailConfig,
}

impl SmtpServicePub {
    #[doc = "SmtpServicePub 구조체의 생성자"]
    pub fn new() -> Self {
        let receiver_email_list: ReceiverEmailConfig =
            match read_toml_from_file::<ReceiverEmailConfig>(&EMAIL_RECEIVER_PATH) {
                Ok(receiver_email_list) => receiver_email_list,
                Err(e) => {
                    error!(
                    "[Error][main()] Failed to retrieve information 'receiver_email_list'. : {:?}",
                    e
                );
                    panic!(
                    "[Error][main()] Failed to retrieve information 'receiver_email_list'. : {:?}",
                    e
                );
                }
            };

        SmtpServicePub {
            receiver_email_list,
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
    async fn send_message_to_receiver_html(
        &self,
        email_id: &str,
        subject: &str,
        html_content: &str,
    ) -> Result<String, anyhow::Error> {
        let smtp_config_info: Arc<SmtpConfig> = get_smtp_config_info();

        let email: Message = Message::builder()
            .from(smtp_config_info.credential_id.parse()?)
            .to(email_id.parse().unwrap())
            .subject(subject)
            .multipart(
                MultiPart::alternative().singlepart(SinglePart::html(html_content.to_string())),
            )?;

        let creds = Credentials::new(
            smtp_config_info.credential_id().to_string(),
            smtp_config_info.credential_pw().to_string(),
        );

        let mailer = AsyncSmtpTransport::<lettre::Tokio1Executor>::relay(
            smtp_config_info.smtp_name().as_str(),
        )?
        .credentials(creds)
        .build();

        match mailer.send(email).await {
            Ok(_) => Ok(email_id.to_string()),
            Err(e) => Err(anyhow!("{:?} : Failed to send email to {} ", e, email_id)),
        }
    }

    #[doc = "지정된 수신자 모두에게 이메일을 보내주는 함수"]
    /// # Arguments
    /// * `error_alarm_infos` - Index error informations
    ///
    /// # Returns
    /// * Result<(), anyhow::Error>
    async fn send_message_to_receivers(
        &self,
        error_alarm_infos: &Vec<ErrorAlarmInfoFormat>,
    ) -> Result<(), anyhow::Error> {
        /* configs */
        let elastic_config: Arc<ElasticServerConfig> = get_elasticsearch_config_info();
        let smtp_config: Arc<SmtpConfig> = get_smtp_config_info();

        /* receiver email list */
        let receiver_email_list: &Vec<ReceiverEmail> = &self.receiver_email_list().emails;

        let email_subject: String = String::from("[Elasticsearch] Indexing ERROR Alarm");
        let mut inner_template: String = String::from("");
        let html_template: String = fs::read_to_string(Path::new(HTML_TEMPLATE_PATH.as_str()))?;

        for err_info in error_alarm_infos {
            let err_info_tag: String = err_info.error_alram_info().convert_email_struct()?;
            inner_template.push_str(&err_info_tag);
        }

        let html_content: String = html_template
            .replace("{cluster_name}", elastic_config.elastic_cluster_name())
            .replace("{index_list}", &inner_template);

        if smtp_config.async_process_yn {
            /* ASYNC TASK */
            let tasks = receiver_email_list.iter().map(|receiver| {
                let email_id: &String = receiver.email_id();
                self.send_message_to_receiver_html(email_id.as_str(), &email_subject, &html_content)
            });

            let results: Vec<Result<String, anyhow::Error>> = join_all(tasks).await;

            for result in results {
                match result {
                    Ok(succ_email_id) => info!("Email sent successfully: {}", succ_email_id),
                    Err(e) => error!(
                        "[Error][send_message_to_receivers()] Failed to send email: {:?}",
                        e
                    ),
                }
            }
        } else {
            /* Not Async */
            for receiver in receiver_email_list {
                let email_id: &String = receiver.email_id();
                self.send_message_to_receiver_html(
                    email_id.as_str(),
                    "[Elasticsearch] Index removed list",
                    &html_content,
                )
                .await?;
            }
        }

        Ok(())
    }
}
