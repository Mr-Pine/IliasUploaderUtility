use std::sync::OnceLock;

use anyhow::{Context, Result};
use regex::Regex;
use reqwest::{blocking::multipart::Form, Url};
use scraper::{element_ref::Select, selectable::Selectable, ElementRef, Selector};
use serde::{Deserialize, Serialize};

use crate::{uploaders::{file_data::FileData, file_with_filename::AddFileWithFilename}, util::Querypath};

use super::{client::IliasClient, file::File, parse_date, IliasElement};

pub enum FolderElement {
    File {
        file: File,
        deletion_querypath: Option<String>,
    },
    Exercise {
        name: String,
        description: String,
        id: String,
        querypath: String,
        deletion_querypath: Option<String>,
    },
    Opencast {
        name: String,
        description: String,
        id: String,
        querypath: String,
        deletion_querypath: Option<String>,
    },
    Viewable {
        name: String,
        description: String,
        id: String,
        querypath: String,
        deletion_querypath: Option<String>,
    },
}

pub struct Folder {
    name: String,
    description: String,
    id: String,
    elements: Vec<FolderElement>,
    upload_page_querypath: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IliasUploadResponse {
    status: u8,
    message: String,
    file_id: String
}

static NAME_SELECTOR: OnceLock<Selector> = OnceLock::new();
static DESCRIPTION_SELECTOR: OnceLock<Selector> = OnceLock::new();
static ID_SELECTOR: OnceLock<Selector> = OnceLock::new();
static UPLOAD_FILE_PAGE_SELECTOR: OnceLock<Selector> = OnceLock::new();

static ELEMENT_SELECTOR: OnceLock<Selector> = OnceLock::new();

impl IliasElement for Folder {
    fn type_identifier() -> &'static str {
        "fold"
    }

    fn parse(element: &ElementRef, ilias_client: &super::client::IliasClient) -> Result<Self> {
        let name_selector = NAME_SELECTOR.get_or_init(|| {
            Selector::parse(".il-page-content-header").expect("Could not parse selector")
        });
        let description_selector = DESCRIPTION_SELECTOR
            .get_or_init(|| Selector::parse(".ilHeaderDesc").expect("Could not parse selector"));
        let id_selector = ID_SELECTOR.get_or_init(|| {
            Selector::parse(".breadcrumbs span:lastChild a").expect("Could not parse selector")
        });
        let upload_file_page_selector = UPLOAD_FILE_PAGE_SELECTOR.get_or_init(|| {
            Selector::parse(r#"#il-add-new-item-gl #file"#).expect("Could not parse selector")
        });

        let element_selector = ELEMENT_SELECTOR.get_or_init(|| {
            Selector::parse(".ilContainerListItemContent").expect("Could not parse selector")
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

        let elements: Vec<FolderElement> = element
            .select(element_selector)
            .filter_map(|element| FolderElement::parse(element))
            .collect();

        let upload_page_querypath = element.select(upload_file_page_selector).next().and_then(|link| link.attr("href")).map(&str::to_string);

        Ok(Folder {
            name,
            description,
            id,
            elements,
            upload_page_querypath
        })
    }
}

static UPLOAD_FORM_SELECTOR: OnceLock<Selector> = OnceLock::new();
static SCRIPT_TAG_SELECTOR: OnceLock<Selector> = OnceLock::new();

impl Folder {
    pub fn upload_files<I: IntoIterator<Item = FileData>>(&self, ilias_client: &IliasClient, files: I) -> Result<()> {
        let upload_page = ilias_client.get_querypath(&self.upload_page_querypath.clone().context("No upload available for this folder")?)?;
        let upload_form_selector = UPLOAD_FORM_SELECTOR.get_or_init(|| Selector::parse("main form")
            .expect("Could not parse scraper"));
        let script_tag_selector = SCRIPT_TAG_SELECTOR.get_or_init(|| Selector::parse("body script:not([src])")
            .expect("Could not parse scraper"));

        let finish_upload_querypath = upload_page
            .select(&upload_form_selector)
            .next()
            .unwrap()
            .value()
            .attr("action")
            .unwrap();

        let relevant_script_tag = upload_page
            .select(&script_tag_selector)
            .next()
            .unwrap()
            .text()
            .collect::<String>();

        let path_regex = Regex::new(r".*il\.UI\.Input\.File\.init\([^']*'[^']*',[^']*'(?<querypath>[^']+)'.*").expect("cursed regex lol");
        let upload_querypath = &path_regex.captures(&relevant_script_tag).expect("no match found :()")["querypath"];

        for file_data in files {
            let form = Form::new()
                .file("file[0]", file_data.path)?;

            let response: IliasUploadResponse = ilias_client.post_querypath_multipart(upload_querypath, form)?.json()?;
            let file_id = response.file_id;

            let finish_form = Form::new()
                .text("form/input_0[input_1][]", file_data.name.clone())
                .text("form/input_0[input_2][]", "")
                .text("form/input_0[input_3][]", file_id)
                .percent_encode_noop();

            ilias_client.post_querypath_multipart(finish_upload_querypath, finish_form)?;
        }

        Ok(())
        // TODO: Maybe push files to submission here
    }
}

static ELEMENT_NAME_SELECTOR: OnceLock<Selector> = OnceLock::new();
static ELEMENT_DESCRIPTION_SELECTOR: OnceLock<Selector> = OnceLock::new();
static ELEMENT_ACTIONS_SELECTOR: OnceLock<Selector> = OnceLock::new();
static ELEMENT_PROPERTIES_SELECTOR: OnceLock<Selector> = OnceLock::new();

impl FolderElement {
    fn parse(element: ElementRef) -> Option<FolderElement> {
        let element_name_selector = ELEMENT_NAME_SELECTOR.get_or_init(|| {
            Selector::parse(".il_ContainerItemTitle a").expect("Could not parse selector")
        });
        let element_description_selector = ELEMENT_DESCRIPTION_SELECTOR
            .get_or_init(|| Selector::parse(".il_Description").expect("Could not parse selector"));
        let element_actions_selector = ELEMENT_ACTIONS_SELECTOR
            .get_or_init(|| Selector::parse(".dropdown-menu li>a").expect("Could not parse selector"));
        let element_properties_selector = ELEMENT_PROPERTIES_SELECTOR.get_or_init(|| {
            Selector::parse(".il_ItemProperties").expect("Could not parse selector")
        });

        let name_element = element.select(element_name_selector).next()?;
        let description_element = element.select(element_description_selector).next()?;
        let mut properties = element.select(element_properties_selector);
        let actions = element.select(element_actions_selector);

        let name: String = name_element.text().collect();
        let link = name_element.attr("href")?;
        let description = description_element.text().collect();
        let querypath = Url::parse(link)
            .expect("Could not parse link")
            .get_querypath();
        let deletion_querypath = actions.filter_map(|action| action.attr("href")).find(|&action| action.contains("cmd=delete")).map(&str::to_string);

        Self::extract_from_querypath(querypath, name, description, deletion_querypath, &mut properties)
    }

    fn extract_from_querypath(
        querypath: String,
        name: String,
        description: String,
        deletion_querypath: Option<String>,
        properties: &mut Select<'_, '_>,
    ) -> Option<FolderElement> {
        if querypath.contains("target=file_") {
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

            let id = Regex::new("target=file_(?<id>\\d+)")
                .unwrap()
                .captures(&querypath)
                .expect("Could not capture id")
                .name("id")
                .expect("Could not get id")
                .as_str();
            let name = format!("{}.{}", name, extension);

            let file = File {
                name,
                description,
                date,
                id: Some(id.to_string()),
                download_querypath: Some(querypath),
            };

            Some(FolderElement::File {
                file,
                deletion_querypath,
            })
        } else if querypath.contains("baseClass=ilObjPluginDispatchGUI")
            && querypath.contains("cmd=forward")
            && querypath.contains("forwardCmd=showContent")
        {
            let id = Regex::new("ref_id=(?<id>\\d+)")
                .ok()?
                .captures(&querypath)?
                .name("id")?
                .as_str()
                .to_string();
            Some(FolderElement::Opencast {
                name,
                description,
                id,
                querypath,
                deletion_querypath,
            })
        } else if querypath.contains("baseClass=ilrepositorygui") && querypath.contains("cmd=view")
        {
            let id = Regex::new("ref_id=(?<id>\\d+)")
                .ok()?
                .captures(&querypath)?
                .name("id")?
                .as_str()
                .to_string();
            Some(FolderElement::Viewable {
                name,
                description,
                id,
                querypath,
                deletion_querypath,
            })
        } else {
            None
        }
    }
}
