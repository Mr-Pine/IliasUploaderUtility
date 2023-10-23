use anyhow::Result;
use reqwest::blocking::Client;

use super::file_data::FileData;

pub trait UploadProvider {
    type UploadedFile;
    fn upload_files<I: Iterator<Item = FileData>>(&self, client: &Client, file_data_iter: I) -> Result<()>;
    fn get_conflicting_files(self: &Self, client: &Client) -> Vec<Self::UploadedFile>;
    fn delete_files<I: IntoIterator<Item = Self::UploadedFile>>(self: &Self, client: &Client, files: I) -> Result<()>;
}