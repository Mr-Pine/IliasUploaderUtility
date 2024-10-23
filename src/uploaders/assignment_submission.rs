use crate::{
    ilias::{client::IliasClient, exercise::assignment::AssignmentSubmission, file::File},
    preselect_delete_setting::PreselectDeleteSetting,
};
use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, MultiSelect};

use super::{file_data::FileData, upload_provider::UploadProvider};

impl UploadProvider for AssignmentSubmission {
    type UploadedFile = File;

    fn upload_files<I: IntoIterator<Item = FileData>>(
        &self,
        ilias_client: &IliasClient,
        file_data_iter: I,
    ) -> Result<()> {
        self.upload_files(ilias_client, file_data_iter)
    }

    fn delete_files<I: IntoIterator<Item = Self::UploadedFile>>(
        self: &Self,
        ilias_client: &IliasClient,
        files: I,
    ) -> Result<()> {
        self.delete_files(ilias_client, files)
    }

    fn get_conflicting_files<I: IntoIterator<Item = String>>(
        self: &Self,
        client: &reqwest::blocking::Client,
        filenames: I,
    ) -> &[File]
    where
        I: Clone,
    {
        &self.submissions
    }

    fn select_files_to_delete<'a, I: Iterator<Item = FileData>>(
        self: &'a Self,
        preselect_setting: PreselectDeleteSetting,
        file_data: &I,
        conflicting_files: &'a [Self::UploadedFile],
    ) -> Result<Box<dyn Iterator<Item = File> + '_>>
    where
        I: Clone,
    {
        let mapped_files: Vec<_> = conflicting_files
            .iter()
            .map(|file| {
                (
                    file.name.clone(),
                    match preselect_setting {
                        PreselectDeleteSetting::ALL => true,
                        PreselectDeleteSetting::NONE => false,
                        PreselectDeleteSetting::SMART => file_data
                            .clone()
                            .any(|file_data| file_data.name == file.name),
                    },
                )
            })
            .collect();

        let selection = MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Which files do you want to delete")
            .items_checked(&mapped_files)
            .interact()?
            .into_iter()
            .map(move |index| conflicting_files[index].clone());
        return Ok(Box::new(selection));
    }
}
