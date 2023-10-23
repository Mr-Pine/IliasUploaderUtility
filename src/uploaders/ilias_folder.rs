use anyhow::{anyhow, Context, Result};
use reqwest::{blocking::Client, Url};
use scraper::{Html, Selector};

use crate::{ilias_url, util::UploadType};

use super::upload_provider::UploadProvider;

#[derive(Debug)]
pub struct IliasFolder {
    file_upload_url: Url,
    page: Html,
}

impl IliasFolder {
    pub fn from_id(client: &Client, id: &str) -> Result<IliasFolder> {
        let base_url = ilias_url!(id, UploadType::FOLDER)?;

        let response = client.get(base_url.clone()).send()?;
        let html_source = dbg!(response.text()?);
        println!("{}", html_source.as_str());
        let page = Html::parse_document(html_source.as_str());


        let upload_file_page_selecor = Selector::parse(r#"#il-add-new-item-gl #file"#)
            .or_else(|err| Err(anyhow!("Could not parse scraper: {:?}", err)))?;
        let upload_file_element = page
            .select(&upload_file_page_selecor)
            .next()
            .context("Did not find link")?;

        let mut upload_url = base_url.clone();

        let querypath = upload_file_element
            .value()
            .attr("href")
            .context("Did not find href")?;
        let mut parts = querypath.split("?");
        let path = parts.next().context("Did not get any parts")?;
        let query = parts.next();

        upload_url.set_path(path);
        upload_url.set_query(query);

        dbg!(&upload_url);

        Ok(IliasFolder {
            file_upload_url: Url::parse(format!("{}", id).as_str())?,
            page: page,
        })
    }
}

impl UploadProvider for IliasFolder {
    type UploadedFile = ();

    fn upload_files<I: Iterator<Item = super::file_data::FileData>>(
        &self,
        client: &Client,
        file_data_iter: I,
    ) {
        todo!()
    }

    fn get_conflicting_files(self: &Self, client: &Client) -> Vec<Self::UploadedFile> {
        todo!()
    }

    fn delete_files<I: IntoIterator<Item = Self::UploadedFile>>(
        self: &Self,
        client: &Client,
        files: I,
    ) {
        todo!()
    }
}
