use anyhow::{Ok, Result};
use reqwest::{blocking::{multipart, Client, Response}, Url};
use scraper::Html;
use serde::Serialize;

use crate::util::Querypath;

#[derive(Debug)]
pub struct IliasClient {
    client: Client,
    base_url: Url,
}

impl IliasClient {
    fn new(base_url: Url) -> Result<IliasClient> {
        let client = Client::builder().cookie_store(true).build()?;

        Ok(IliasClient { client, base_url })
    }

    pub fn get_querypath(&self, querypath: &str) -> Result<Html> {
        let mut url = self.base_url.clone();
        url.set_querypath(querypath);

        let response = self.client.get(url).send()?;
        let html = Html::parse_document(&response.text()?);

        Ok(html)
    }

    pub fn post_querypath_form<T: Serialize + ?Sized>(&self, querypath: &str, form: &T) -> Result<()> {
        let mut url = self.base_url.clone();
        url.set_querypath(querypath);

        let response = self.client.post(url).form(form).send()?;
        response.error_for_status()?;
        Ok(())
    }

    pub fn post_querypath_multipart(&self, querypath: &str, form: multipart::Form) -> Result<Response> {
        let mut url = self.base_url.clone();
        url.set_querypath(querypath);

        let response = self.client.post(url).multipart(form).send()?;

        Ok(response.error_for_status()?)
    }
}
