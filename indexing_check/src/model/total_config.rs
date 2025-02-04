use crate::common::*;

use crate::model::code_config::*;
use crate::model::elastic_server_config::*;
use crate::model::smtp_config::*;
use crate::model::system_config::*;
use crate::model::telegram_config::*;

use crate::utils_modules::io_utils::*;

use crate::env_configuration::env_config::*;

static SERVER_CONFIG: once_lazy<Arc<Config>> =
    once_lazy::new(|| Arc::new(initialize_server_config()));

#[doc = "Function to initialize System configuration information instances"]
pub fn initialize_server_config() -> Config {
    info!("initialize_server_config() START!");

    let system_config: Config = Config::new();
    system_config
}

#[doc = "Elasticsearch config 정보"]
pub fn get_elasticsearch_config_info() -> Arc<ElasticServerConfig> {
    let elastic_config: &Arc<ElasticServerConfig> = &SERVER_CONFIG.elasticsearch;
    Arc::clone(elastic_config)
}

#[doc = "SMTP config 정보"]
pub fn get_smtp_config_info() -> Arc<SmtpConfig> {
    let smtp_config: &Arc<SmtpConfig> = &SERVER_CONFIG.smtp;
    Arc::clone(smtp_config)
}

#[doc = "Telegram config 정보"]
pub fn get_telegram_config_info() -> Arc<TelegramConfig> {
    let telegram_config: &Arc<TelegramConfig> = &SERVER_CONFIG.telegram;
    Arc::clone(telegram_config)
}

#[doc = "System config 정보"]
pub fn get_system_config_info() -> Arc<SystemConfig> {
    let system_config: &Arc<SystemConfig> = &SERVER_CONFIG.system;
    Arc::clone(system_config)
}

#[doc = "code 타입 config 정보"]
pub fn get_code_config_info() -> Arc<CodeConfig> {
    let code_config: &Arc<CodeConfig> = &SERVER_CONFIG.code_type;
    Arc::clone(code_config)
}

#[derive(Debug)]
pub struct Config {
    pub elasticsearch: Arc<ElasticServerConfig>,
    pub smtp: Arc<SmtpConfig>,
    pub telegram: Arc<TelegramConfig>,
    pub system: Arc<SystemConfig>,
    pub code_type: Arc<CodeConfig>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigNotSafe {
    pub elasticsearch: ElasticServerConfig,
    pub smtp: SmtpConfig,
    pub telegram: TelegramConfig,
    pub system: SystemConfig,
    pub code_type: CodeConfig,
}

impl Config {
    pub fn new() -> Self {
        let system_config = match read_toml_from_file::<ConfigNotSafe>(&SYSTEM_CONFIG_PATH) {
            Ok(system_config) => system_config,
            Err(e) => {
                error!(
                    "[Error][main()] Failed to retrieve information 'system_config'. : {:?}",
                    e
                );
                panic!(
                    "[Error][main()] Failed to retrieve information 'system_config'. : {:?}",
                    e
                );
            }
        };

        Config {
            elasticsearch: Arc::new(system_config.elasticsearch),
            smtp: Arc::new(system_config.smtp),
            telegram: Arc::new(system_config.telegram),
            system: Arc::new(system_config.system),
            code_type: Arc::new(system_config.code_type),
        }
    }
}
