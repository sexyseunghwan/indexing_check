use crate::common::*;

use crate::model::error_alarm_info_format::*;

#[async_trait]
pub trait NotificationService {
    async fn send_message_to_receivers(
        &self,
        error_alarm_infos: &[ErrorAlarmInfoFormat],
    ) -> Result<(), anyhow::Error>;
}
