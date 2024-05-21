use anyhow::{anyhow, Context, Ok, Result};
use regex::Regex;
use reqwest::{
    blocking::{multipart::Form, Client}, Url
};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

use crate::{
    ilias_url,
    uploaders::existing_file::ExistingFile,
    util::{SetQuerypath, UploadType},
};

use super::upload_provider::UploadProvider;

#[derive(Debug)]
pub struct IliasFolder {
    id: String,
    base_url: Url,
    page: Html,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IliasUploadResponse {
    status: u8,
    message: String,
    file_id: String
}

impl IliasFolder {
    pub fn from_id(client: &Client, id: &str) -> Result<IliasFolder> {
        let base_url = ilias_url!(id, UploadType::FOLDER)?;

        let response = client.get(base_url.clone()).send()?;
        let html_source = response.text()?;
        let page = Html::parse_document(html_source.as_str());

        Ok(IliasFolder {
            id: id.to_string(),
            base_url: base_url,
            page: page,
        })
    }

    fn delete_file(self: &Self, client: &Client, file: ExistingFile) -> Result<()> {
        let delete_page_url = Url::parse(format!("https://ilias.studium.kit.edu/ilias.php?ref_id={}&item_ref_id={}&cmd=delete&cmdClass=ilobjfoldergui&cmdNode=x1:nk&baseClass=ilrepositorygui", self.id, file.id).as_str())?;
        let delete_page_response = client.get(delete_page_url).send()?;
        let delete_page = Html::parse_document(delete_page_response.text()?.as_str());

        let form_selector = Selector::parse("main form").unwrap();
        let confirm_querypath = delete_page
            .select(&form_selector)
            .next()
            .unwrap()
            .value()
            .attr("action")
            .unwrap();

        let mut url = self.base_url.clone();
        url.set_querypath(confirm_querypath);

        let form_data = [("id[]", file.id.as_str()),("cmd[confirmedDelete]", "I fucking hate ILIAS")];

        client.post(url).form(&form_data).send()?;
        Ok(())
    }
}

impl UploadProvider for IliasFolder {
    type UploadedFile = ExistingFile;

    fn upload_files<I: Iterator<Item = super::file_data::FileData>>(
        &self,
        client: &Client,
        file_data_iter: I,
    ) -> Result<()> {
        let upload_file_page_selecor = Selector::parse(r#"#il-add-new-item-gl #file"#)
            .or_else(|err| Err(anyhow!("Could not parse scraper: {:?}", err)))?;

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

        let upload_form_selector = Selector::parse("main form")
            .or_else(|err| Err(anyhow!("Could not parse scraper: {:?}", err)))?;

        let finish_upload_querypath = upload_page
            .select(&upload_form_selector)
            .next()
            .unwrap()
            .value()
            .attr("action")
            .unwrap();

        let script_tag_selector = Selector::parse("body script:not([src])")
            .or_else(|err| Err(anyhow!("Could not parse scraper: {:?}", err)))?;
        let relevant_script_tag = upload_page
            .select(&script_tag_selector)
            .next()
            .unwrap()
            .text()
            .collect::<String>();

        let path_regex = Regex::new(r".*il\.UI\.Input\.File\.init\([^']*'[^']*',[^']*'(?<querypath>[^']+)'.*").expect("cursed regex lol");
        let upload_querypath = &path_regex.captures(&relevant_script_tag).expect("no match found :()")["querypath"];

        let mut url = self.base_url.clone();
        url.set_querypath(upload_querypath);

        for file_data in file_data_iter {
            let form = Form::new()
                .file("file[0]", file_data.path)?;

            let response: IliasUploadResponse = client.post(url.clone()).multipart(form).send()?.json()?;
            let file_id = response.file_id;

            let finish_form = Form::new()
                .text("form/input_0[input_1][]", file_data.name.clone())
                .text("form/input_0[input_2][]", "")
                .text("form/input_0[input_3][]", file_id)
                .percent_encode_noop();

            url.set_querypath(finish_upload_querypath);
            client.post(url.clone()).multipart(finish_form).send()?;
        }

        Ok(())
    }

    fn get_conflicting_files<I: IntoIterator<Item = String>>(self: &Self, _client: &Client, filenames: I) -> Vec<Self::UploadedFile> where I: Clone {
        let file_link_selector = Selector::parse("a.il_ContainerItemTitle").unwrap();
        let file_property_selector = Selector::parse(".il_ItemProperties span").unwrap();
        let file_row_selector = Selector::parse("div.ilContainerListItemOuter").unwrap();
        let file_id_regex = Regex::new("lg_div_(?P<id>\\d+)_pref_\\d+").unwrap();
        let file_rows = self.page.select(&file_row_selector);
        let files = file_rows.map(|row| {
            let file_link = row.select(&file_link_selector).next().unwrap();
            let element_id = row.value().id().unwrap();
            let id = file_id_regex.replace(element_id, "$id");
            let filename = file_link.text().next().unwrap();
            let filetype = row.select(&file_property_selector).next().unwrap().text().next().unwrap().trim();

            let filename = format!("{}.{}", filename, filetype);
            ExistingFile {
                name: filename,
                id: id.to_string()
            }
        });
        return files.filter(|file| filenames.clone().into_iter().collect::<Vec<_>>().contains(&file.name)).collect();
    }

    fn delete_files<I: IntoIterator<Item = Self::UploadedFile>>(
        self: &Self,
        client: &Client,
        files: I,
    ) -> Result<()> {
        for file in files.into_iter() {
            self.delete_file(client, file)?;
        };
        println!("Successfully deleted other files");
        Ok(())
    }

    fn select_files_to_delete<'a, I: Iterator<Item = super::file_data::FileData>>(
        self: &'a Self,
        _preselect_setting: crate::preselect_delete_setting::PreselectDeleteSetting,
        _file_data: &I,
        conflicting_files: &'a [Self::UploadedFile],
    ) -> Result<Box<dyn Iterator<Item = Self::UploadedFile> + '_>>
    where
        I: Clone,
    {
        return Ok(Box::new(conflicting_files.iter().map(|f| f.clone())));
    }
}
