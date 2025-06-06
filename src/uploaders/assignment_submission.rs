use crate::preselect_delete_setting::PreselectDeleteSetting;
use ilias::{
    client::IliasClient, exercise::assignment::AssignmentSubmission, file::File,
    local_file::NamedLocalFile,
};
use snafu::{ResultExt, Whatever};

use super::upload_provider::UploadProvider;

impl UploadProvider for AssignmentSubmission {
    type UploadedFile = File;

    fn upload_files(&self, ilias_client: &IliasClient, file_data: &[NamedLocalFile]) -> Result<(), Whatever> {
        self.upload_files(ilias_client, file_data).whatever_context("ilias-rs file upload failed")
    }

    fn delete_files(
        &self,
        ilias_client: &IliasClient,
        files: &[&Self::UploadedFile],
    ) -> Result<(), Whatever> {
        self.delete_files(ilias_client, files).whatever_context("ilias-rs file delete failed")
    }

    fn get_existing_files(&self) -> Vec<&File> {
        self.submissions.iter().collect()
    }

    fn preselect_files<'a>(
        &self,
        preselect_setting: PreselectDeleteSetting,
        upload_files: &[NamedLocalFile],
        existing_files: Vec<&'a Self::UploadedFile>,
    ) -> Vec<(&'a Self::UploadedFile, bool)> {
        existing_files
            .into_iter()
            .map(|existing_file| {
                (
                    existing_file,
                    match preselect_setting {
                        PreselectDeleteSetting::All => true,
                        PreselectDeleteSetting::None => false,
                        PreselectDeleteSetting::Smart => {
                            let filename = &existing_file.name;
                            upload_files.iter().any(|file| file.name == *filename)
                        }
                    },
                )
            })
            .collect()
    }
}
