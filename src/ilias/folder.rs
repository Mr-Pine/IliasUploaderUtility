use std::{fmt::Display, sync::OnceLock};

use anyhow::{Context, Result};
use regex::Regex;
use reqwest::{blocking::multipart::Form, Url};
use scraper::{element_ref::Select, selectable::Selectable, ElementRef, Selector};
use serde::{Deserialize, Serialize};

use crate::{uploaders::file_data::FileData, util::Querypath};

use super::{client::IliasClient, file::File, parse_date, IliasElement};

#[derive(Clone)]
#[allow(dead_code)]
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

#[derive(Clone)]
#[allow(dead_code)]
pub struct Folder {
    name: String,
    description: String,
    id: String,
    pub elements: Vec<FolderElement>,
    upload_page_querypath: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IliasUploadResponse {
    status: u8,
    message: String,
    file_id: String,
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

    fn querypath_from_id(id: &str) -> String {
        format!(
            "goto.php?target={}_{}&client_id=produktiv",
            Self::type_identifier(),
            id
        )
    }

    fn parse(element: ElementRef) -> Result<Self> {
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

        let upload_page_querypath = element
            .select(upload_file_page_selector)
            .next()
            .and_then(|link| link.attr("href"))
            .map(str::to_string);

        Ok(Folder {
            name,
            description,
            id,
            elements,
            upload_page_querypath,
        })
    }
}

static MAIN_FORM_SELECTOR: OnceLock<Selector> = OnceLock::new();
static SCRIPT_TAG_SELECTOR: OnceLock<Selector> = OnceLock::new();

impl Folder {
    pub fn upload_files(&self, ilias_client: &IliasClient, files: &[FileData]) -> Result<()> {
        let upload_page = ilias_client.get_querypath(
            &self
                .upload_page_querypath
                .clone()
                .context("No upload available for this folder")?,
        )?;
        let upload_form_selector = MAIN_FORM_SELECTOR
            .get_or_init(|| Selector::parse("main form").expect("Could not parse scraper"));
        let script_tag_selector = SCRIPT_TAG_SELECTOR.get_or_init(|| {
            Selector::parse("body script:not([src])").expect("Could not parse scraper")
        });

        let finish_upload_querypath = upload_page
            .select(upload_form_selector)
            .next()
            .unwrap()
            .value()
            .attr("action")
            .unwrap();

        let relevant_script_tag = upload_page
            .select(script_tag_selector)
            .next()
            .unwrap()
            .text()
            .collect::<String>();

        let path_regex =
            Regex::new(r".*il\.UI\.Input\.File\.init\([^']*'[^']*',[^']*'(?<querypath>[^']+)'.*")
                .expect("cursed regex lol");
        let upload_querypath = &path_regex
            .captures(&relevant_script_tag)
            .expect("no match found :()")["querypath"];

        for file_data in files {
            let form = Form::new().file("file[0]", file_data.path.clone())?;

            let response: IliasUploadResponse = ilias_client
                .post_querypath_multipart(upload_querypath, form)?
                .json()?;
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
        let element_actions_selector = ELEMENT_ACTIONS_SELECTOR.get_or_init(|| {
            Selector::parse(".dropdown-menu li>a").expect("Could not parse selector")
        });
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
        let deletion_querypath = actions
            .filter_map(|action| action.attr("href"))
            .find(|&action| action.contains("cmd=delete"))
            .map(str::to_string);

        Self::extract_from_querypath(
            querypath,
            name,
            description,
            deletion_querypath,
            &mut properties,
        )
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

    fn deletion_querypath(&self) -> Option<&String> {
        match self {
            Self::File {
                file: _,
                deletion_querypath,
            } => deletion_querypath,
            Self::Exercise {
                name: _,
                description: _,
                id: _,
                querypath: _,
                deletion_querypath,
            } => deletion_querypath,
            Self::Opencast {
                name: _,
                description: _,
                id: _,
                querypath: _,
                deletion_querypath,
            } => deletion_querypath,
            Self::Viewable {
                name: _,
                description: _,
                id: _,
                querypath: _,
                deletion_querypath,
            } => deletion_querypath,
        }
        .as_ref()
    }

    pub fn file(&self) -> Option<&File> {
        match self {
            Self::File {
                file,
                deletion_querypath: _,
            } => Some(file),
            _ => None,
        }
    }

    fn id(&self) -> &str {
        match self {
            Self::File {
                file,
                deletion_querypath: _,
            } => file.id.as_ref().unwrap(),
            Self::Exercise {
                name: _,
                description: _,
                id,
                querypath: _,
                deletion_querypath: _,
            } => id,
            Self::Opencast {
                name: _,
                description: _,
                id,
                querypath: _,
                deletion_querypath: _,
            } => id,
            Self::Viewable {
                name: _,
                description: _,
                id,
                querypath: _,
                deletion_querypath: _,
            } => id,
        }
    }

    pub fn delete(&self, ilias_client: &IliasClient) -> Result<()> {
        let deletion_querypath = self.deletion_querypath();
        let delete_page = ilias_client
            .get_querypath(deletion_querypath.context("You can not delete this element")?)?;

        let form_selector = MAIN_FORM_SELECTOR
            .get_or_init(|| Selector::parse("main form").expect("Could not parse scraper"));
        let confirm_querypath = delete_page
            .select(form_selector)
            .next()
            .context("Could not find confirmation form")?
            .value()
            .attr("action")
            .context("Could not find action on form")?;

        let form_data = [
            ("id[]", self.id()),
            ("cmd[confirmedDelete]", "I fucking hate ILIAS"),
        ];

        let _ = ilias_client.post_querypath_form(confirm_querypath, &form_data);
        Ok(())
    }
}

impl Display for FolderElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FolderElement::File {
                file,
                deletion_querypath: _,
            } => write!(f, "{}", file),
            FolderElement::Exercise {
                name,
                description: _,
                id: _,
                querypath: _,
                deletion_querypath: _,
            } => write!(f, "Exercise {}", name),
            FolderElement::Opencast {
                name,
                description: _,
                id: _,
                querypath: _,
                deletion_querypath: _,
            } => write!(f, "OpenCast {}", name),
            FolderElement::Viewable {
                name,
                description: _,
                id: _,
                querypath: _,
                deletion_querypath: _,
            } => write!(f, "Folder(-like) {}", name),
        }
    }
}
