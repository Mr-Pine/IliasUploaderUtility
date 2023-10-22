use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Config {
    pub username: Option<String>,
    pub ilias_id: Option<String>,
    pub transform_regex: Option<String>,
    pub transform_format: Option<String>
}