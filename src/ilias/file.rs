use std::fmt::Display;

use chrono::{DateTime, Local};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct File {
    pub name: String,
    pub description: String,
    pub date: Option<DateTime<Local>>,
    pub download_querypath: Option<String>,
    pub id: Option<String>,
}

impl Display for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
