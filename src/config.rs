use serde::Deserialize;

use crate::{preselect_delete_setting::PreselectDeleteSetting, util::UploadType};

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Config {
    pub username: Option<String>,
    pub ilias_id: Option<String>,
    pub preselect_delete: Option<PreselectDeleteSetting>,
    pub transform_regex: Option<String>,
    pub transform_format: Option<String>,
    pub upload_type: Option<UploadType> 
}