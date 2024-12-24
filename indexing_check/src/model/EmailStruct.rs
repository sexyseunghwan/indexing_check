use crate::common::*;

use crate::utils_modules::time_utils::*;


#[derive(Serialize, Deserialize, Debug)]
pub struct EmailStruct {
    pub index_name: String,
    pub indexing_date: String,
    pub status: bool,
    pub html_form: String
}


impl EmailStruct {
        
    pub fn new(index_name: &str, deletion_date_naive: NaiveDateTime, status: bool) -> Result<Self, anyhow::Error> {  
        
        let indexing_date = 
            get_str_from_naivedatetime(deletion_date_naive, "%Y-%m-%dT%H:%M:%SZ")?;
        
        let status_description = if status { "Successful" } else { "Failed" };
        let color = if status { "green" } else { "red" };

        let html_form = format!(
            "<tr>
                <td style='border: 1px solid #ddd; padding: 8px; text-align: left;'>{}</td>
                <td style='border: 1px solid #ddd; padding: 8px; text-align: left;'>{}</td>
                <td style='border: 1px solid #ddd; padding: 8px; text-align: left; color: {};'>{}</td>
            </tr>",
            index_name, deletion_date_naive, color, status_description
        );
        
        Ok(
            Self { index_name: index_name.to_string(), 
                    indexing_date, 
                    status, 
                    html_form
                }
        )
    }

}