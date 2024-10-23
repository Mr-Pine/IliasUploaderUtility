use crate::{
    ilias::{client::IliasClient, exercise::assignment::AssignmentSubmission, file::File},
    preselect_delete_setting::PreselectDeleteSetting,
};
use anyhow::Result;

use super::{file_data::FileData, upload_provider::UploadProvider};

impl UploadProvider for AssignmentSubmission {
    type UploadedFile = File;

    fn upload_files(
        &self,
        ilias_client: &IliasClient,
        file_data: &[FileData],
    ) -> Result<()> {
        self.upload_files(ilias_client, file_data)
    }

    fn delete_files(
        self: &Self,
        ilias_client: &IliasClient,
        files: &[&Self::UploadedFile],
    ) -> Result<()> {
        self.delete_files(ilias_client, files)
    }

    fn get_existing_files(
        self: &Self,
    ) -> Vec<&File>
    {
        self.submissions.iter().collect()
    }

    fn preselect_files<'a>(&self, preselect_setting: PreselectDeleteSetting, upload_files: &[FileData], existing_files: Vec<&'a Self::UploadedFile>) -> Vec<(&'a Self::UploadedFile, bool)> {
        existing_files.into_iter().map(|existing_file| {
            (existing_file,
             match preselect_setting {
                 PreselectDeleteSetting::ALL => true,
                 PreselectDeleteSetting::NONE => false,
                 PreselectDeleteSetting::SMART => {
                     let filename = &existing_file.name;
                     upload_files.iter().any(|file| file.name == *filename)
                 }
             }
        )}).collect()
    }
}
