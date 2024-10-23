use anyhow::Result;

use crate::{ilias::client::IliasClient, preselect_delete_setting::PreselectDeleteSetting};

use super::file_data::FileData;

pub trait UploadProvider {
    type UploadedFile: ToString;
    fn upload_files(&self, ilias_client: &IliasClient, file_data: &[FileData]) -> Result<()>;
    fn get_existing_files(self: &Self) -> Vec<&Self::UploadedFile>;
    fn delete_files(self: &Self, ilias_client: &IliasClient, files: &[&Self::UploadedFile]) -> Result<()>;
    fn preselect_files<'a>(&self, preselect_setting: PreselectDeleteSetting, upload_files: &[FileData], existing_files: Vec<&'a Self::UploadedFile>) -> Vec<(&'a Self::UploadedFile, bool)>;
    /*fn select_files_to_delete<'a, I: Iterator<Item = FileData>>(
        self: &'a Self,
        preselect_setting: PreselectDeleteSetting,
        file_data: &I,
        conflicting_files: &'a [Self::UploadedFile],
    ) -> Result<Box<dyn Iterator<Item = Self::UploadedFile> + '_>>
    where
        I: Clone;*/
}
