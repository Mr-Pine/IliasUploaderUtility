use clap::ValueEnum;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, ValueEnum, PartialEq)]
#[clap(rename_all = "kebab_case")]
pub enum UploadType {
    Exercise,
    Folder,
}

impl UploadType {
    pub fn get_delete_message(&self) -> &str {
        match self {
            UploadType::Exercise => {
                "This excercise already has uploaded files. Do you want to delete any of them?"
            }
            UploadType::Folder => {
                "There are already files in this folder. Do you want to delete any of them?"
            }
        }
    }
}
