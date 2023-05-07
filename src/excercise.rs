use std::error::Error;

use scraper::{ElementRef, Selector};

#[derive(Debug)]
pub struct Excercise {
    active: bool,
    name: String,
    submit_querypath: Option<String>
}

impl Excercise {
    pub fn parse_from(excercise: ElementRef) -> Result<Excercise, Box<dyn Error>> {
        let name_selector = Selector::parse(r#".il_VAccordionHead span.ilAssignmentHeader"#)?;
        let name = excercise.select(&name_selector).next().unwrap().text().collect();

        let submit_button_selector = Selector::parse(r#"a.btn.btn-default.btn-primary"#).unwrap();
        let button = excercise.select(&submit_button_selector).next();

        let submit_querypath = match button {
            Some(submit_button) => {
                Some(submit_button.value().attr("href").unwrap().to_string())
            },
            None => None,
        };

        Ok(Excercise { active: submit_querypath.is_some(), submit_querypath: submit_querypath, name: name })
    }
}