use crate::common::*;

use crate::model::error_alarm_info::*;
use crate::utils_modules::traits::*;

#[derive(Serialize, Deserialize, Debug, Setters, Getters, new)]
#[getset(get = "pub", set = "pub")]
pub struct ErrorAlarmInfoFormat {
    pub doc_id: String,
    pub error_alram_info: ErrorAlarmInfo,
}

impl FromSearchHit<ErrorAlarmInfo> for ErrorAlarmInfoFormat {
    fn from_search_hit(doc_id: String, error_alram_info: ErrorAlarmInfo) -> Self {
        ErrorAlarmInfoFormat::new(doc_id, error_alram_info)
    }
}
