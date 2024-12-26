use crate::common::*;

use crate::model::ElasticServerConfig::*;
use crate::model::SmtpConfig::*;
use crate::model::TelegramConfig::*;
use crate::model::SystemConfig::*;

use crate::utils_modules::io_utils::*;

static SERVER_CONFIG: once_lazy<Arc<Config>> = once_lazy::new(|| {
    Arc::new(initialize_server_config())
});


#[doc = "Function to initialize System configuration information instances"]
pub fn initialize_server_config() -> Config {

    info!("initialize_server_config() START!");

    let system_config = Config::new();
    system_config
}


#[doc = ""]
pub fn get_elasticsearch_config_info() -> Arc<ElasticServerConfig> {

    let elastic_config = &SERVER_CONFIG.elasticsearch;
    Arc::clone(elastic_config)
}

#[doc = ""]
pub fn get_smtp_config_info() -> Arc<SmtpConfig> {

    let smtp_config = &SERVER_CONFIG.smtp;
    Arc::clone(smtp_config)
}

#[doc = ""]
pub fn get_telegram_config_info() -> Arc<TelegramConfig> {

    let telegram_config = &SERVER_CONFIG.telegram;
    Arc::clone(telegram_config)
}

#[doc = ""]
pub fn get_system_config_info() -> Arc<SystemConfig> {

    let system_config = &SERVER_CONFIG.system;
    Arc::clone(system_config)
}


#[derive(Debug)]
pub struct Config {
    pub elasticsearch: Arc<ElasticServerConfig>,
    pub smtp: Arc<SmtpConfig>,
    pub telegram: Arc<TelegramConfig>,
    pub system: Arc<SystemConfig>
}


#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigNotSafe {
    pub elasticsearch: ElasticServerConfig,
    pub smtp: SmtpConfig,
    pub telegram: TelegramConfig,
    pub system: SystemConfig
}


impl Config {
    
    pub fn new() -> Self {

        let system_config = match read_toml_from_file::<ConfigNotSafe>("./config/system_config.toml") {
            Ok(system_config) => system_config,
            Err(e) => {
                error!("[Error][main()] Failed to retrieve information 'system_config'. : {:?}", e);
                panic!("[Error][main()] Failed to retrieve information 'system_config'. : {:?}", e);
            }
        }; 
        
        Config {
            elasticsearch: Arc::new(system_config.elasticsearch),
            smtp: Arc::new(system_config.smtp),
            telegram: Arc::new(system_config.telegram),
            system: Arc::new(system_config.system)
        }
    }
}