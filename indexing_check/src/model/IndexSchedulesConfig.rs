use crate::common::*;


#[derive(Debug, Deserialize, Serialize, Getters, Clone)]
#[getset(get = "pub")]
pub struct IndexSchedules {
    pub index_name: String,
    pub time: String
}

#[derive(Debug, Deserialize, Serialize, Getters, Clone)]
#[getset(get = "pub")]
pub struct IndexSchedulesConfig {
    pub index: Vec<IndexSchedules>
}