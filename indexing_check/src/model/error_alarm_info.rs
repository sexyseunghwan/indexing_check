use crate::common::*;

#[derive(Serialize, Deserialize, Debug, Setters, Getters, new)]
#[getset(get = "pub", set = "pub")]
pub struct ErrorAlarmInfo {
    #[serde(rename = "@timestamp")]
    pub timestamp: String,
    pub error_type: String,
    pub index_name: String,
    pub index_type: String,
    pub indexing_cnt_num: usize,
    pub declare_index_size: usize,
}

impl ErrorAlarmInfo {
    #[doc = "인덱스 에러 정보를 이메일 구조로 변환해주는 함수"]
    pub fn convert_email_struct(&self) -> Result<String, anyhow::Error> {
        let color: &str = if self.error_type == "Full Error" {
            "red"
        } else {
            "yellow"
        };

        let html_form: String = format!(
            "<tr>
                <td style='border: 1px solid #ddd; padding: 8px; text-align: left;'>{}</td>
                <td style='border: 1px solid #ddd; padding: 8px; text-align: left;'><span style='color: red;'>{}</span> ({})</td>
                <td style='border: 1px solid #ddd; padding: 8px; text-align: left;'>{}</td>
                <td style='border: 1px solid #ddd; padding: 8px; text-align: left; color: {};'>{}</td>
            </tr>",
            self.index_name, 
            self.indexing_cnt_num.to_formatted_string(&Locale::en), self.declare_index_size.to_formatted_string(&Locale::en),
            self.index_type(),
            color, self.error_type
        );
        
        Ok(html_form)
    }
}