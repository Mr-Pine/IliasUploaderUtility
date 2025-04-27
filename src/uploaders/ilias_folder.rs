use crate::preselect_delete_setting::PreselectDeleteSetting;
use ilias::{
    client::IliasClient,
    folder::{Folder, FolderElement},
    local_file::NamedLocalFile,
};
use snafu::{ResultExt, Whatever};

use super::upload_provider::UploadProvider;
impl UploadProvider for Folder {
    type UploadedFile = FolderElement;

    fn upload_files(&self, ilias_client: &IliasClient, file_data: &[NamedLocalFile]) -> Result<(), Whatever> {
        self.upload_files(ilias_client, file_data)
    }

    fn delete_files(
        &self,
        ilias_client: &IliasClient,
        files: &[&Self::UploadedFile],
    ) -> Result<(), Whatever> {
        for file in files {
            file.delete(ilias_client).whatever_context("Unable to delete file")?;
        }
        Ok(())
    }

    fn get_existing_files(&self) -> Vec<&FolderElement> {
        let files = self
            .elements
            .iter()
            .filter(|element| {
                matches!(
                    element,
                    FolderElement::File {
                        file: _,
                        deletion_querypath: _
                    }
                )
            })
            .collect::<Vec<_>>();
        files
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
