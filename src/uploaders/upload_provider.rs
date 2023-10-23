use anyhow::Result;
use reqwest::blocking::Client;

use crate::preselect_delete_setting::PreselectDeleteSetting;

use super::file_data::FileData;

pub trait UploadProvider {
    type UploadedFile;
    fn upload_files<I: Iterator<Item = FileData>>(&self, client: &Client, file_data_iter: I) -> Result<()>;
    fn get_conflicting_files(self: &Self, client: &Client) -> Vec<Self::UploadedFile>;
    fn delete_files<I: IntoIterator<Item = Self::UploadedFile>>(self: &Self, client: &Client, files: I) -> Result<()>;
    fn select_files_to_delete<'a, I: Iterator<Item = FileData>>(
        self: &'a Self,
        preselect_setting: PreselectDeleteSetting,
        file_data: &I,
        conflicting_files: &'a [Self::UploadedFile],
    ) -> Result<Box<dyn Iterator<Item = Self::UploadedFile> + '_>>
    where
        I: Clone;
}