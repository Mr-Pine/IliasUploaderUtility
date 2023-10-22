use anyhow::Result;

use crate::preselect_delete_setting::PreselectDeleteSetting;

use super::{excercise::existing_file::ExistingFile, file_data::FileData};

pub trait DeleteSelection {
    type UploadedFile;
    fn select_files_to_delete<'a, I: Iterator<Item = FileData>>(
        self: &'a Self,
        preselect_setting: PreselectDeleteSetting,
        file_data: &I,
        conflicting_files: &'a [Self::UploadedFile],
    ) -> Result<Box<dyn Iterator<Item = ExistingFile> + '_>>
    where
        I: Clone;
}
