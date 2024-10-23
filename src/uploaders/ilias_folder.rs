use anyhow::{Ok, Result};

use crate::{
    ilias::{
        client::IliasClient,
        folder::{Folder, FolderElement},
    },
    preselect_delete_setting::PreselectDeleteSetting,
};

use super::{file_data::FileData, upload_provider::UploadProvider};
impl UploadProvider for Folder {
    type UploadedFile = FolderElement;

    fn upload_files(&self, ilias_client: &IliasClient, file_data: &[FileData]) -> Result<()> {
        self.upload_files(ilias_client, file_data)
    }

    fn delete_files(
        self: &Self,
        ilias_client: &IliasClient,
        files: &[&Self::UploadedFile],
    ) -> Result<()> {
        for file in files {
            file.delete(ilias_client)?;
        }
        Ok(())
    }

    fn get_existing_files(self: &Self) -> Vec<&FolderElement> {
        let files = self
            .elements
            .iter()
            .filter_map(|element| match element {
                FolderElement::File {
                    file: _,
                    deletion_querypath: _,
                } => Some(element),
                _ => None,
            })
            .collect::<Vec<_>>();
        files
    }

    fn preselect_files<'a>(
        &self,
        preselect_setting: PreselectDeleteSetting,
        upload_files: &[FileData],
        existing_files: Vec<&'a Self::UploadedFile>,
    ) -> Vec<(&'a Self::UploadedFile, bool)> {
        existing_files
            .into_iter()
            .map(|existing_file| {
                (
                    existing_file,
                    match preselect_setting {
                        PreselectDeleteSetting::ALL => true,
                        PreselectDeleteSetting::NONE => false,
                        PreselectDeleteSetting::SMART => {
                            let filename = &existing_file
                                .file()
                                .expect("Encountered non-file existing element")
                                .name;
                            upload_files.iter().any(|file| file.name == *filename)
                        }
                    },
                )
            })
            .collect()
    }
}
