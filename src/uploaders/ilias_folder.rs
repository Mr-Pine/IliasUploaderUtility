use std::iter::empty;

use anyhow::{anyhow, Context, Ok, Result};
use reqwest::{blocking::Client, Url};
use scraper::{Html, Selector};

use crate::{
    ilias_url,
    uploaders::upload_utils::upload_files_to_url,
    util::{SetQuerypath, UploadType},
};

use super::upload_provider::UploadProvider;

#[derive(Debug)]
pub struct IliasFolder {
    base_url: Url,
    page: Html,
}

impl IliasFolder {
    pub fn from_id(client: &Client, id: &str) -> Result<IliasFolder> {
        let base_url = ilias_url!(id, UploadType::FOLDER)?;

        let response = client.get(base_url.clone()).send()?;
        let html_source = response.text()?;
        let page = Html::parse_document(html_source.as_str());

        Ok(IliasFolder {
            base_url: base_url,
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
    ) -> Result<()> {
        let upload_file_page_selecor = Selector::parse(r#"#il-add-new-item-gl #file"#)
            .or_else(|err| Err(anyhow!("Could not parse scraper: {:?}", err)))?;

        println!("{}", self.page.html());

        let upload_file_element = self
            .page
            .select(&upload_file_page_selecor)
            .next()
            .context("Did not find link")?;

        let mut upload_page_url = self.base_url.clone();

        let querypath = upload_file_element
            .value()
            .attr("href")
            .context("Did not find href")?;
        let mut parts = querypath.split("?");
        let path = parts.next().context("Did not get any parts")?;
        let query = parts.next();

        upload_page_url.set_path(path);
        upload_page_url.set_query(query);

        let upload_page_response = client.get(upload_page_url).send()?;
        let upload_page = Html::parse_document(upload_page_response.text()?.as_str());

        let upload_link_selector = Selector::parse("#form_")
            .or_else(|err| Err(anyhow!("Could not parse scraper: {:?}", err)))?;

        let upload_querypath = upload_page
            .select(&upload_link_selector)
            .next()
            .unwrap()
            .value()
            .attr("action")
            .unwrap();

        let mut url = self.base_url.clone();
        url.set_querypath(upload_querypath);

        upload_files_to_url(&client, file_data_iter, url)
    }

    fn get_conflicting_files(self: &Self, client: &Client) -> Vec<Self::UploadedFile> {
        vec![]
    }

    fn delete_files<I: IntoIterator<Item = Self::UploadedFile>>(
        self: &Self,
        client: &Client,
        files: I,
    ) -> Result<()> {
        //todo!();
        Ok(())
    }

    fn select_files_to_delete<'a, I: Iterator<Item = super::file_data::FileData>>(
        self: &'a Self,
        preselect_setting: crate::preselect_delete_setting::PreselectDeleteSetting,
        file_data: &I,
        conflicting_files: &'a [Self::UploadedFile],
    ) -> Result<Box<dyn Iterator<Item = ()> + '_>>
    where
        I: Clone,
    {
        let vec: Vec<()> = vec![];
        return Ok(Box::new(empty()));
    }
}
