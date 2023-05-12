pub mod file;
use std::error::Error;

use reqwest::{
    blocking::{multipart, Client},
    Url,
};
use scraper::{ElementRef, Html, Selector};

use crate::util::set_querypath;

use self::file::File;

#[derive(Debug)]
pub struct Excercise {
    pub active: bool,
    pub name: String,
    pub has_files: bool,
    submit_url: Option<Url>,
    overview_page: Option<Html>,
}

impl Excercise {
    #[allow(dead_code)]
    pub fn new(
        client: &Client,
        excercise: ElementRef<'_>,
        base_url: Url,
    ) -> Result<Excercise, Box<dyn Error>> {
        let mut raw = Self::parse_from(excercise, base_url)?;
        let overview_page = raw.get_overview_page(client).unwrap();
        {
            let ref mut this = raw;
            this.overview_page = Some(overview_page);
        };
        Ok(raw)
    }

    pub fn parse_from(excercise: ElementRef, base_url: Url) -> Result<Excercise, Box<dyn Error>> {
        let name_selector = Selector::parse(r#".il_VAccordionHead span.ilAssignmentHeader"#)?;
        let name = excercise
            .select(&name_selector)
            .next()
            .unwrap()
            .text()
            .collect();

        let submit_button_selector = Selector::parse(r#"a.btn.btn-default.btn-primary"#).unwrap();
        let button = excercise.select(&submit_button_selector).next();

        let mut url = base_url.clone();

        let (has_files, subit_url) = match button {
            Some(submit_button) => {
                let querypath = submit_button.value().attr("href").unwrap().to_string();
                let mut parts = querypath.split("?");
                let path = parts.next().unwrap();
                let query = parts.next();

                url.set_path(path);
                url.set_query(query);

                (
                    submit_button.text().collect::<String>().contains("Lösung"),
                    Some(url),
                )
            }
            None => (false, None),
        };

        Ok(Excercise {
            active: subit_url.is_some(),
            submit_url: subit_url,
            has_files: has_files,
            name: name,
            overview_page: None,
        })
    }

    fn get_overview_page(&self, client: &Client) -> Option<Html> {
        if self.overview_page.is_some() {
            self.overview_page.clone()
        } else {
            let response = client.get(self.submit_url.clone().unwrap()).send().unwrap();

            Some(Html::parse_document(response.text().unwrap().as_str()))
        }
    }

    pub fn delete_all_files(self: &Self, client: &Client) {
        let page = self.get_overview_page(&client).unwrap();
        let files = File::parse_uploaded_files(&page);
        let ids = files.iter().map(|file| file.id.clone());
        let form_selector = Selector::parse(r#"div#ilContentContainer form"#).unwrap();
        let delete_querypath = page
            .select(&form_selector)
            .next()
            .unwrap()
            .value()
            .attr("action")
            .unwrap();

        let url = set_querypath(self.submit_url.clone().unwrap(), delete_querypath);

        let mut form_args = ids.map(|id| ("delivered[]", id)).collect::<Vec<_>>();
        form_args.push(("cmd[deleteDelivered]", String::from("Löschen")));

        let _confirm_response = client.post(url.clone()).form(&form_args).send().unwrap();
    }

    pub fn upload_files(&self, client: &Client, file_paths: &Vec<String>) {
        let mut form = multipart::Form::new();

        for (index, file_path) in file_paths.iter().enumerate() {
            form = form.file(format!("deliver[{}]", index), file_path).unwrap();
        }

        let upload_button_selector = Selector::parse(r#"nav div.navbar-header button"#).unwrap();
        let page = self.get_overview_page(client).unwrap();
        let upload_querypath = page
            .select(&upload_button_selector)
            .next()
            .unwrap()
            .value()
            .attr("data-action")
            .unwrap();

        let url = set_querypath(self.submit_url.clone().unwrap(), upload_querypath);

        let upload_page = client.post(url.clone()).send().unwrap();
        let form_selector = Selector::parse(r#"div#ilContentContainer form"#).unwrap();
        let page = Html::parse_document(upload_page.text().unwrap().as_str());
        let submit_querypath = page
            .select(&form_selector)
            .next()
            .unwrap()
            .value()
            .attr("action")
            .unwrap();

        let url = set_querypath(url, submit_querypath);

        client.post(url).multipart(form).send().unwrap();
    }
}
