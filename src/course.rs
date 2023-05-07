use std::error::Error;

use reqwest::{Client, Url};
use scraper::{Html, Selector};

use crate::{ilias_url, excercise::{self, Excercise}};

pub struct Course {
    name: String,
}

impl Course {
    pub async fn from_id(client: &Client, id: &str, name: &str) -> Result<Course, Box<dyn Error>> {
        let ilias_id = ilias_url!(id);

        let course_response = client.get(ilias_id).send().await.unwrap();
        let course_page = Html::parse_document(course_response.text().await.unwrap().as_str());

        let part_selector = Selector::parse(r#"div.il_VAccordionContainer div.il_VAccordionInnerContainer"#).unwrap();
        let excercises = course_page.select(&part_selector).map(|excercise| {Excercise::parse_from(excercise).unwrap()});

        dbg!(&excercises.collect::<Vec<Excercise>>());

        Ok(Course {
            name: String::from(name),
        })
    }
}
