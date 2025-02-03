use crate::common::*;

#[derive(Serialize, Deserialize, Debug, Getters)]
#[getset(get = "pub")]
pub struct VectorIndexLog {
    pub file: String,
    pub host: String,
    pub index_name: String,
    pub message: String,
    pub source_type: String,
    pub state: String,
    pub timestamp: String,
}
