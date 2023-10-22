use anyhow::Result;
use reqwest::blocking::Client;

use crate::preselect_delete_setting::PreselectDeleteSetting;

use super::{file_data::FileData, excercise::existing_file::ExistingFile};

pub trait DeleteSelection {
    fn select_files_to_delete<I: Iterator<Item = FileData>>(self: &Self, client: &Client, preselect_setting: PreselectDeleteSetting, file_data: &I) -> Result<Box<dyn Iterator<Item = ExistingFile>>> where I: Clone;
}