use crate::common::*;

use crate::model::index_schedules_config::*;

#[async_trait]
pub trait IndexStorageService {
    fn get_index_schedule_info(&self) -> IndexSchedules;
}

#[derive(Debug, Getters, new)]
#[getset(get = "pub")]
pub struct IndexStorageServicePub {
    pub index_schedule: IndexSchedules,
}

#[async_trait]
impl IndexStorageService for IndexStorageServicePub {
    #[doc = "index_schedule 의 정보를 반환해주는 함수"]
    fn get_index_schedule_info(&self) -> IndexSchedules {
        self.index_schedule.clone()
    }
}
