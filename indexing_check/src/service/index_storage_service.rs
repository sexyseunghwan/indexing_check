use crate::common::*;

use crate::model::IndexSchedulesConfig::*;


#[async_trait]
pub trait IndexStorageService {

    fn get_index_schedule_vec(&self) -> Vec<IndexSchedules>;

}

#[derive(Debug, Getters, new)]
#[getset(get = "pub")]
pub struct IndexStorageServicePub {
    pub index_schedule_vec: Vec<IndexSchedules>
}


#[async_trait]
impl IndexStorageService for IndexStorageServicePub {
    
    #[doc = ""]
    fn get_index_schedule_vec(&self) -> Vec<IndexSchedules> {
        self.index_schedule_vec.clone()
    }

}