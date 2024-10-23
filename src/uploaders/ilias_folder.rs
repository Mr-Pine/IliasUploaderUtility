use anyhow::{Ok, Result};
use dialoguer::{theme::ColorfulTheme, MultiSelect};

use crate::{
    ilias::{client::IliasClient, folder::{Folder, FolderElement}}, preselect_delete_setting::PreselectDeleteSetting
};

use super::{file_data::FileData, upload_provider::UploadProvider};
impl UploadProvider for Folder {
    type UploadedFile = FolderElement;

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
        for file in files {
            file.delete(ilias_client)?;
        }
        Ok(())
    }

    fn get_conflicting_files<I: IntoIterator<Item = String>>(
        self: &Self,
        client: &reqwest::blocking::Client,
        filenames: I,
    ) -> Vec<&FolderElement>
    where
        I: Clone,
    {
        let files = self.elements.iter().filter_map(|element| match element {
            FolderElement::File {file: _, deletion_querypath: _ } => Some(element),
            _ => None
        }).collect::<Vec<_>>();
        files
    }

    fn select_files_to_delete<'a, I: Iterator<Item = FileData>>(
        self: &'a Self,
        preselect_setting: PreselectDeleteSetting,
        file_data: &I,
        conflicting_files: &'a [&FolderElement],
    ) -> Result<Box<dyn Iterator<Item = &FolderElement> + '_>>
    where
        I: Clone,
    {
        let mapped_files: Vec<_> = conflicting_files
            .iter()
            .map(|element| {
                let file_name = &element.file().expect("Encountered non-file conflicting element").name;
                (
                    file_name.clone(),
                    match preselect_setting {
                        PreselectDeleteSetting::ALL => true,
                        PreselectDeleteSetting::NONE => false,
                        PreselectDeleteSetting::SMART => file_data
                            .clone()
                            .any(|file_data| file_data.name == *file_name),
                    },
                )
            })
            .collect();

        let selection = MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Which files do you want to delete")
            .items_checked(&mapped_files)
            .interact()?
            .into_iter()
            .map(move |index| conflicting_files[index]);
        return Ok(Box::new(selection));
    }
}
