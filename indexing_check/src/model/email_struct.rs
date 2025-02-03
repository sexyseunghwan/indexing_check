use crate::common::*;


#[derive(Serialize, Deserialize, Debug)]
pub struct EmailStruct {
    pub index_name: String,
    pub html_form: String,
}

impl EmailStruct {
    pub fn new(
        index_name: &str,
        index_cnt: usize,
        declare_index_cnt: usize,
    ) -> Result<Self, anyhow::Error> {
        let html_form: String = format!(
            "<tr>
                <td style='border: 1px solid #ddd; padding: 8px; text-align: left;'>{}</td>
                <td style='border: 1px solid #ddd; padding: 8px; text-align: left;'><span style='color: red;'>{}</span> ({})</td>
                <td style='border: 1px solid #ddd; padding: 8px; text-align: left; color: red;'>Failed</td>
            </tr>",
            index_name, 
            index_cnt.to_formatted_string(&Locale::en), declare_index_cnt.to_formatted_string(&Locale::en)
        );

        Ok(Self {
            index_name: index_name.to_string(),
            html_form,
        })
    }
}
