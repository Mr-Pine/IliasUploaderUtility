use std::{borrow::Cow, path::Path};

use anyhow::{anyhow, Context, Ok, Result};
use reqwest::{
    blocking::{multipart::{self, Form, Part}, Client, Response},
    Url,
};
use scraper::{Html, Selector};
use serde::Serialize;

use super::Querypath;

#[derive(Debug)]
pub struct IliasClient {
    client: Client,
    base_url: Url,
}

impl IliasClient {
    pub fn new(base_url: Url) -> Result<IliasClient> {
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

    pub fn post_querypath_form<T: Serialize + ?Sized>(
        &self,
        querypath: &str,
        form: &T,
    ) -> Result<()> {
        let mut url = self.base_url.clone();
        url.set_querypath(querypath);

        let response = self.client.post(url).form(form).send()?;
        response.error_for_status()?;
        Ok(())
    }

    pub fn post_querypath_multipart(
        &self,
        querypath: &str,
        form: multipart::Form,
    ) -> Result<Response> {
        let mut url = self.base_url.clone();
        url.set_querypath(querypath);

        let response = self.client.post(url).multipart(form).send()?;

        Ok(response.error_for_status()?)
    }

    pub fn authenticate(&self, username: &str, password: &str) -> Result<()> {
        println!("Authenticating!");

        let shib_path = "shib_login.php";

        let shib_params = [
            ("sendLogin", "1"),
            ("idp_selection", "https://idp.scc.kit.edu/idp/shibboleth"),
            ("il_target", ""),
            ("home_organization_selection", "Weiter"),
        ];

        let mut url = self.base_url.clone();
        url.set_path(shib_path);
        let shib_url = url.as_str().to_owned();

        let shib_login_page = self.client.post(url).form(&shib_params).send()?;

        let mut url = shib_login_page.url().to_owned();
        let is_ilias = url
            .as_str()
            .starts_with(self.base_url.host_str().context("Base url has no host")?);
        if is_ilias {
            println!("Exiting auth, already logged in");
            return Ok(());
        }

        let shib_login_fragment = Html::parse_document(shib_login_page.text()?.as_str());
        let csrf_selector =
            Selector::parse(r#"input[name="csrf_token"]"#).expect("Could not parse selector");
        let crsf_field = shib_login_fragment.select(&csrf_selector).next();

        let shib_continue_fragment: Html;

        let path_selector =
            Selector::parse(r#"form[method="post"]"#).expect("Could not parse selector");

        if crsf_field.is_some() {
            let crsf = crsf_field.unwrap().value().attr("value").unwrap();

            let form_data = [
                ("csrf_token", crsf),
                ("j_username", username),
                ("j_password", password),
                ("_eventId_proceed", ""),
            ];

            let post_querypath = shib_login_fragment
                .select(&path_selector)
                .next()
                .unwrap()
                .value()
                .attr("action")
                .unwrap();

            url.set_querypath(post_querypath);
            let continue_response = self.client.post(url).form(&form_data).send()?;

            shib_continue_fragment = Html::parse_document(continue_response.text()?.as_str());
        } else {
            shib_continue_fragment = shib_login_fragment;
        }

        let saml_selector =
            Selector::parse(r#"input[name="SAMLResponse"]"#).expect("Could not parse selector");
        let saml = shib_continue_fragment
            .select(&saml_selector)
            .next()
            .context("Did not find SAML Response input")?
            .value()
            .attr("value")
            .context("Could not get SAML response value")?;

        let continue_form_data = [("RelayState", shib_url.as_str()), ("SAMLResponse", saml)];

        let continue_url = shib_continue_fragment
            .select(&path_selector)
            .next()
            .unwrap()
            .value()
            .attr("action")
            .unwrap();

        let ilias_home = self
            .client
            .post(continue_url)
            .form(&continue_form_data)
            .send();

        if ilias_home?.status().is_success() {
            println!("Logged in!");
            Ok(())
        } else {
            Err(anyhow!("Ilias login not successful!"))
        }
    }
}

pub trait AddFileWithFilename {
    fn file_with_name<T, U, V>(self, name: T, path: U, filename: V) -> Result<Form>
    where
        T: Into<Cow<'static, str>>,
        U: AsRef<Path>,
        V: Into<Cow<'static, str>>;
}

impl AddFileWithFilename for Form {
    fn file_with_name<T, U, V>(self, name: T, path: U, filename: V) -> Result<Form>
    where
        T: Into<Cow<'static, str>>,
        U: AsRef<Path>,
        V: Into<Cow<'static, str>>,
    {
        Ok(self.part(name, Part::file(path)?.file_name(filename)))
    }
}
