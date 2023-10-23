pub mod existing_file;
use std::fmt::Debug;

use anyhow::{Result, Context, anyhow, Ok};
use dialoguer::{MultiSelect, theme::ColorfulTheme};
use reqwest::{
    blocking::Client,
    Url
};
use scraper::{ElementRef, Html, Selector};

use crate::{util::SetQuerypath, preselect_delete_setting::PreselectDeleteSetting};

use self::existing_file::ExistingFile;

use super::{upload_provider::UploadProvider, file_data::FileData, delete_selection::DeleteSelection, upload_utils::upload_files_to_url};

#[derive(Debug)]
pub struct Excercise {
    pub active: bool,
    pub name: String,
    has_files: bool,
    submit_url: Url,
    overview_page: Option<Html>,
}

impl Excercise {
    #[allow(dead_code)]
    pub fn new(
        client: &Client,
        excercise: ElementRef<'_>,
        base_url: Url,
    ) -> Result<Excercise> {
        let mut raw = Self::parse_from(excercise, base_url)?;
        let overview_page = raw.get_overview_page(client)?;
        {
            let ref mut this = raw;
            this.overview_page = Some(overview_page);
        };
        Ok(raw)
    }

    pub fn parse_from(excercise: ElementRef, base_url: Url) -> Result<Excercise> {
        let name_selector = Selector::parse(r#".il_VAccordionHead span.ilAssignmentHeader"#).or_else(|err| Err(anyhow!("Could not parse scraper: {:?}", err)))?;
        let name = excercise
            .select(&name_selector)
            .next().context("Did not find name for execise")?
            .text()
            .collect();

        let submit_button_selector = Selector::parse(r#"a.btn.btn-default.btn-primary"#).or_else(|err| Err(anyhow!("Could not parse scraper: {:?}", err)))?;
        let button = excercise.select(&submit_button_selector).next();

        let mut url = base_url.clone();

        let (has_files, subit_url_option) = match button {
            Some(submit_button) => {
                let querypath = submit_button.value().attr("href").context("Did not find href")?;
                url.set_querypath(querypath);

                (
                    // TODO: Improve
                    submit_button.text().collect::<String>().contains("Lösung"),
                    Some(url.clone()),
                )
            }
            None => (false, None),
        };

        Ok(Excercise {
            active: subit_url_option.is_some(),
            submit_url: match subit_url_option {
                Some(url) => url,
                None => url,
            },
            has_files: has_files,
            name: name,
            overview_page: None,
        })
    }

    fn get_overview_page(&self, client: &Client) -> Result<Html> {
        if let Some(page) = &self.overview_page {
            Ok(page.clone())
        } else {
            let response = client.get(self.submit_url.clone()).send().unwrap();

            Ok(Html::parse_document(response.text()?.as_str()))
        }
    }
}

impl UploadProvider for Excercise {
    type UploadedFile = ExistingFile;

    fn upload_files<I: Iterator<Item = FileData>>(&self, client: &Client, file_data_iter: I) -> Result<()> {
                let upload_button_selector = Selector::parse(r#"nav div.navbar-header button"#).unwrap();
        let page = self.get_overview_page(client)?;
        let upload_querypath = page
            .select(&upload_button_selector)
            .next()
            .unwrap()
            .value()
            .attr("data-action")
            .unwrap();

        let mut url = self.submit_url.clone();
        url.set_querypath(upload_querypath);

        let upload_page = client.post(url.clone()).send()?;
        let form_selector = Selector::parse(r#"div#ilContentContainer form"#)
        .or_else(|err| Err(anyhow!("Could not parse scraper: {:?}", err)))?;
        let page = Html::parse_document(upload_page.text()?.as_str());
        let submit_querypath = page
            .select(&form_selector)
            .next()
            .unwrap()
            .value()
            .attr("action")
            .unwrap();

        url.set_querypath(submit_querypath);

        upload_files_to_url(&client, file_data_iter, url)
    }



    fn get_conflicting_files(self: &Self, client: &Client) -> Vec<ExistingFile> {
        if !self.has_files {
            return vec![];
        }
        let page = self.get_overview_page(&client).unwrap();
        let files = ExistingFile::parse_uploaded_files(&page);
        return files;
    }

    fn delete_files<I: IntoIterator<Item = Self::UploadedFile>>(self: &Self, client: &Client, files: I) -> Result<()>{
        let page = self.get_overview_page(client)?;
        let ids = files.into_iter().map(|file| file.id.clone());
        let form_selector = Selector::parse(r#"div#ilContentContainer form"#)
        .or_else(|err| Err(anyhow!("Could not parse scraper: {:?}", err)))?;
        let delete_querypath = page
            .select(&form_selector)
            .next()
            .unwrap()
            .value()
            .attr("action")
            .unwrap();

        let mut url = self.submit_url.clone();
        url.set_querypath(delete_querypath);

        let mut form_args = ids.map(|id| ("delivered[]", id)).collect::<Vec<_>>();
        form_args.push(("cmd[deleteDelivered]", String::from("Löschen")));

        let _confirm_response = client.post(url.clone()).form(&form_args).send()?;
        Ok(())
    }
}

impl DeleteSelection for Excercise {
    type UploadedFile = ExistingFile;

    fn select_files_to_delete<'a, I: Iterator<Item = FileData>>(self: &'a Self, preselect_setting: PreselectDeleteSetting, file_data: &I, conflicting_files: &'a [Self::UploadedFile]) -> Result<Box<dyn Iterator<Item = ExistingFile> + '_>> where I: Clone {
            let mapped_files: Vec<(&str, bool)> = conflicting_files
                .iter()
                .map(|file| {
                    (
                        file.name.as_str(),
                        if preselect_setting == PreselectDeleteSetting::ALL {
                            true
                        } else if preselect_setting == PreselectDeleteSetting::NONE {
                            false
                        } else {
                            file_data
                                .clone()
                                .any(|file_data| file_data.name == file.name)
                        }
                    )
                })
                .collect();
            let selection = MultiSelect::with_theme(&ColorfulTheme::default())
                .with_prompt("Which files do you want to delete")
                .items_checked(&mapped_files)
                .interact()?
                .into_iter()
                .map(move |index| conflicting_files[index].clone());
            return Ok(Box::new(selection));
    }
}