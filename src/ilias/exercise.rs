use std::sync::OnceLock;

use anyhow::{Context, Result};
use assignment::Assignment;
use scraper::Selector;

pub mod assignment;


use super::{client::IliasClient, IliasElement};

#[derive(Debug)]
pub struct Exercise {
    pub name: String,
    pub description: String,
    pub assignments: Vec<Assignment>,
}

static ASSIGNMENT_SELECTOR: OnceLock<Selector> = OnceLock::new();
static NAME_SELECTOR: OnceLock<Selector> = OnceLock::new();
static DESCRIPTION_SELECTOR: OnceLock<Selector> = OnceLock::new();

impl IliasElement for Exercise {
    fn type_identifier() -> &'static str {
        "exc"
    }

    fn parse(element: &scraper::ElementRef, ilias_client: &IliasClient) -> Result<Exercise> {
        let name_selector = NAME_SELECTOR.get_or_init(|| {
            Selector::parse(".il-page-content-header").expect("Could not parse scraper")
        });
        let description_selector = DESCRIPTION_SELECTOR
            .get_or_init(|| Selector::parse(".ilHeaderDesc").expect("Could not parse scraper"));
        let assignment_selector = ASSIGNMENT_SELECTOR.get_or_init(|| {
            Selector::parse(r#"div.il_VAccordionContainer div.il_VAccordionInnerContainer"#)
                .expect("Could not parse scraper")
        });

        let name = element.select(name_selector).next().context("No \"name\" Element found")?.text().collect();
        let description = element.select(description_selector).next().context("No \"description\" Element found")?.text().collect();
        let assignments = element
            .select(&assignment_selector)
            .map(|assignment| Assignment::parse(&assignment, ilias_client).expect("Could not parse assignment"))
            .collect();

        Ok(Exercise {
            name,
            description,
            assignments
        })
    }
}
