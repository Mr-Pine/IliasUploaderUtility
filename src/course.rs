use anyhow::{Result, anyhow};
use reqwest::{blocking::Client, Url};
use scraper::{Html, Selector};

use crate::{ilias_url, uploaders::excercise::Excercise, util::UploadType};

#[derive(Debug)]
pub struct Course {
    pub name: String,
    pub excercises: Vec<Excercise>
}

impl Course {
    pub fn from_id(client: &Client, id: &str, name: &str) -> Result<Course> {
        let ilias_url = ilias_url!(id, UploadType::EXERCISE)?;

        let course_response = client.get(ilias_url.clone()).send()?;
        let course_page = Html::parse_document(course_response.text()?.as_str());

        let part_selector = Selector::parse(r#"div.il_VAccordionContainer div.il_VAccordionInnerContainer"#).or_else(|err| Err(anyhow!("Could not parse scraper: {:?}", err)))?;
        let excercises = course_page.select(&part_selector).map(|excercise| {Excercise::parse_from(excercise, ilias_url.clone()).unwrap()}).collect::<Vec<Excercise>>();

        
        Ok(Course {
            name: String::from(name),
            excercises: excercises
        })
    }
}
