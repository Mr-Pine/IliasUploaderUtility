use anyhow::{Result, Ok};
use reqwest::{blocking::{Client, multipart::Form}, Url};

use crate::uploaders::file_with_filename::AddFileWithFilename;

use super::file_data::FileData;

pub fn upload_files_to_url<I: Iterator<Item = FileData>>(client: &Client, file_data: I, target: Url) -> Result<()> {
    let mut form = Form::new();

        for (index, file_path) in file_data.enumerate() {
            dbg!(&file_path);
            form = form.file_with_name(format!("deliver[{}]", index), file_path.path, file_path.name)?;
        }
    

        let result = client.post(target).multipart(form).send()?;
        dbg!(result);
        Ok(())
}