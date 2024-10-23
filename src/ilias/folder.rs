use std::sync::OnceLock;

use anyhow::{Context, Result};
use regex::Regex;
use reqwest::Url;
use scraper::{selectable::Selectable, Selector};

use crate::util::Querypath;

use super::{file::File, parse_date, IliasElement};

struct Folder {
    name: String,
    description: String,
    id: String,
    material: Vec<File>,
}

static NAME_SELECTOR: OnceLock<Selector> = OnceLock::new();
static DESCRIPTION_SELECTOR: OnceLock<Selector> = OnceLock::new();
static ID_SELECTOR: OnceLock<Selector> = OnceLock::new();
static UPLOAD_FILE_LINK_SELECTOR: OnceLock<Selector> = OnceLock::new();

static ELEMENT_SELECTOR: OnceLock<Selector> = OnceLock::new();
static ELEMENT_NAME_SELECTOR: OnceLock<Selector> = OnceLock::new();
static ELEMENT_PROPERTIES_SELECTOR: OnceLock<Selector> = OnceLock::new();

impl IliasElement for Folder {
    fn type_identifier() -> &'static str {
        "fold"
    }

    fn parse(
        element: &scraper::ElementRef,
        ilias_client: &super::client::IliasClient,
    ) -> Result<Self> {
        let name_selector = NAME_SELECTOR.get_or_init(|| {
            Selector::parse(".il-page-content-header").expect("Could not parse selector")
        });
        let description_selector = DESCRIPTION_SELECTOR
            .get_or_init(|| Selector::parse(".ilHeaderDesc").expect("Could not parse selector"));
        let id_selector = ID_SELECTOR.get_or_init(|| {
            Selector::parse(".breadcrumbs span:lastChild a").expect("Could not parse selector")
        });
        let upload_file_link_selector = UPLOAD_FILE_LINK_SELECTOR.get_or_init(|| {
            Selector::parse(r#"#il-add-new-item-gl #file"#).expect("Could not parse selector")
        });
        let element_properties_selector = ELEMENT_PROPERTIES_SELECTOR.get_or_init(|| {
            Selector::parse(".il_ItemProperties").expect("Could not parse selector")
        });

        let element_selector = ELEMENT_SELECTOR.get_or_init(|| {
            Selector::parse(".ilContainerListItemContent").expect("Could not parse selector")
        });
        let element_name_selector = ELEMENT_NAME_SELECTOR.get_or_init(|| {
            Selector::parse(".il_ContainerItemTitle a").expect("Could not parse selector")
        });

        let name = element
            .select(name_selector)
            .next()
            .context("Could not find name")?
            .text()
            .collect();
        let description = element
            .select(description_selector)
            .next()
            .context("Could not find description")?
            .text()
            .collect();
        let id = element
            .select(id_selector)
            .next()
            .context("Could not find link in breadcrumbs")?
            .attr("href")
            .context("Link missing href attribute")?
            .to_string();

        let elements: Vec<File> = element
            .select(element_selector)
            .filter_map(|element| {
                let name_element = element.select(element_name_selector).next()?;
                let name: String = name_element.text().collect();
                let link = name_element.attr("href")?;
                let mut properties = element.select(element_properties_selector);

                if link.contains("target=file_") {
                    let extension: String = properties
                        .next()
                        .expect("Could not find file extension")
                        .text()
                        .collect();
                    let mut date = None;
                    while date.is_none() {
                        let next_property = properties.next();
                        if next_property.is_none() {
                            continue;
                        }
                        date = parse_date(&next_property.unwrap().text().collect::<String>()).ok();
                    }

                    let download_querypath = Url::parse(link)
                        .expect("Could not parse link")
                        .get_querypath();
                    let id = Regex::new("target=file_(?<id>\\d+)")
                        .unwrap()
                        .captures(&download_querypath)
                        .expect("Could not capture id")
                        .name("id")
                        .expect("Could not get id")
                        .as_str();
                    let name = format!("{}.{}", name, extension);

                    Some(File {
                        name,
                        date,
                        id: Some(id.to_string()),
                        download_querypath: Some(download_querypath),
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok(Folder {
            name,
            description,
            id,
            material: elements,
        })
    }
}
