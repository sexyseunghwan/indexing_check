use crate::common::*;

#[doc = "Function to globally initialize the 'INDEX_LIST_PATH' variable"]
pub static INDEX_LIST_PATH: once_lazy<String> = once_lazy::new(|| {
    dotenv().ok();
    env::var("INDEX_LIST_PATH").expect("[ENV file read Error] 'INDEX_LIST_PATH' must be set")
});

#[doc = "Function to globally initialize the 'EMAIL_RECEIVER_PATH' variable"]
pub static EMAIL_RECEIVER_PATH: once_lazy<String> = once_lazy::new(|| {
    dotenv().ok();
    env::var("EMAIL_RECEIVER_PATH").expect("[ENV file read Error] 'EMAIL_RECEIVER_PATH' must be set")
});

#[doc = "Function to globally initialize the 'SYSTEM_CONFIG_PATH' variable"]
pub static SYSTEM_CONFIG_PATH: once_lazy<String> = once_lazy::new(|| {
    dotenv().ok();
    env::var("SYSTEM_CONFIG_PATH").expect("[ENV file read Error] 'SYSTEM_CONFIG_PATH' must be set")
});