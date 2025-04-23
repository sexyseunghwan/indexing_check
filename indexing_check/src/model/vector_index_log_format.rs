use crate::common::*;

use crate::utils_modules::traits::*;

use crate::model::vector_index_log::*;

#[derive(Serialize, Deserialize, Debug, Getters, new)]
#[getset(get = "pub")]
pub struct VectorIndexLogFormat {
    pub doc_id: String,
    pub vector_index_log: VectorIndexLog,
}

impl FromSearchHit<VectorIndexLog> for VectorIndexLogFormat {
    fn from_search_hit(doc_id: String, vector_index_log: VectorIndexLog) -> Self {
        VectorIndexLogFormat::new(doc_id, vector_index_log)
    }
}
