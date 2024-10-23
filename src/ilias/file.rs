use chrono::{DateTime, Local};

#[derive(Debug, Clone)]
pub struct File {
    pub name: String,
    pub description: String,
    pub date: Option<DateTime<Local>>,
    pub download_querypath: Option<String>,
    pub id: Option<String>
}
