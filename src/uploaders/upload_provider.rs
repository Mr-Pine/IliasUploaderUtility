use ilias::{client::IliasClient, local_file::NamedLocalFile};
use snafu::Whatever;

use crate::preselect_delete_setting::PreselectDeleteSetting;

pub trait UploadProvider {
    type UploadedFile: ToString;
    fn upload_files(&self, ilias_client: &IliasClient, file_data: &[NamedLocalFile]) -> Result<(), Whatever>;
    fn get_existing_files(&self) -> Vec<&Self::UploadedFile>;
    fn delete_files(&self, ilias_client: &IliasClient, files: &[&Self::UploadedFile])
        -> Result<(), Whatever>;
    fn preselect_files<'a>(
        &self,
        preselect_setting: PreselectDeleteSetting,
        upload_files: &[NamedLocalFile],
        existing_files: Vec<&'a Self::UploadedFile>,
    ) -> Vec<(&'a Self::UploadedFile, bool)>;
    /*fn select_files_to_delete<'a, I: Iterator<Item = FileData>>(
        self: &'a Self,
        preselect_setting: PreselectDeleteSetting,
        file_data: &I,
        conflicting_files: &'a [Self::UploadedFile],
    ) -> Result<Box<dyn Iterator<Item = Self::UploadedFile> + '_>>
    where
        I: Clone;*/
}
