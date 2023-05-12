use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub username: Option<String>,
    pub ilias_id: Option<String>
}