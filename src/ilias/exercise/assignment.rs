use std::sync::OnceLock;

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Local};
use reqwest::blocking::multipart::Form;
use scraper::{selectable::Selectable, ElementRef, Selector};

use crate::{
    ilias::{client::IliasClient, file::File, parse_date, IliasElement}, uploaders::{file_data::FileData, file_with_filename::AddFileWithFilename}, util::Querypath
};

#[derive(Debug)]
pub enum Submission {
    Unresolved(String),
    Parsed(AssignmentSubmission),
}

#[derive(Debug)]
pub struct Assignment {
    pub name: String,
    pub instructions: String,
    pub submission_date: DateTime<Local>,
    pub attachments: Vec<File>,
    submission: Option<Submission>,
}

static INFO_SCREEN_SELECTOR: OnceLock<Selector> = OnceLock::new();
static INFO_SCREEN_NAME_SELECTOR: OnceLock<Selector> = OnceLock::new();

static NAME_SELECTOR: OnceLock<Selector> = OnceLock::new();
static FIRST_INFO_VALUE_SELECTOR: OnceLock<Selector> = OnceLock::new();
static ATTACHMENT_ROW_SELECTOR: OnceLock<Selector> = OnceLock::new();
static SUBMISSION_PAGE_SELECTOR: OnceLock<Selector> = OnceLock::new();

impl IliasElement for Assignment {
    fn type_identifier() -> &'static str {
        "ass"
    }

    fn parse(element: &scraper::ElementRef, ilias_client: &IliasClient) -> Result<Self> {
        let name_selector = NAME_SELECTOR.get_or_init(|| {
            Selector::parse(".ilAssignmentHeader").expect("Could not parse selector")
        });

        let info_screen_selector = INFO_SCREEN_SELECTOR
            .get_or_init(|| Selector::parse(".ilInfoScreenSec").expect("Could not parse selector"));
        let info_screen_name_selector = INFO_SCREEN_NAME_SELECTOR
            .get_or_init(|| Selector::parse(".ilHeader").expect("Could not parse selector"));
        let first_info_value_selector = FIRST_INFO_VALUE_SELECTOR.get_or_init(|| {
            Selector::parse(".il_InfoScreenPropertyValue").expect("Could not parse selector")
        });
        let attachment_row_selector = ATTACHMENT_ROW_SELECTOR
            .get_or_init(|| Selector::parse(".form-group").expect("Could not parse selector"));
        let submission_page_selector = SUBMISSION_PAGE_SELECTOR
            .get_or_init(|| Selector::parse("a").expect("Could not parse selector"));

        let name = element
            .select(name_selector)
            .next()
            .context("Did not find name")?
            .text()
            .collect();

        let info_screens: Vec<_> = element
            .select(info_screen_selector)
            .map(|info_screen| {
                (
                    info_screen,
                    info_screen
                        .select(info_screen_name_selector)
                        .next()
                        .context(anyhow!("Could not find name of info screen"))
                        .unwrap()
                        .text()
                        .collect::<String>(),
                )
            })
            .collect();

        let instruction_info = info_screens
            .iter()
            .find_map(|(screen, name)| {
                if ["Arbeitsanweisung", "Work Instructions"].contains(&name.as_str()) {
                    Some(screen)
                } else {
                    None
                }
            })
            .context("Did not find any instructions")?;
        let instructions = instruction_info
            .select(first_info_value_selector)
            .next()
            .context("Did not find instruction text")?
            .text()
            .collect();

        let schedule_info = info_screens
            .iter()
            .find_map(|(screen, name)| {
                if ["Schedule", "Terminplan"].contains(&name.as_str()) {
                    Some(screen)
                } else {
                    None
                }
            })
            .context("Did not find schedule")?;
        let submission_date: String = schedule_info
            .select(first_info_value_selector)
            .next()
            .context("Did not find date")?
            .text()
            .collect();
        let submission_date = parse_date(&submission_date)?;

        let attachment_info = info_screens.iter().find_map(|(screen, name)| {
            if ["Dateien", "Files"].contains(&name.as_str()) {
                Some(screen)
            } else {
                None
            }
        });
        let attachments = attachment_info.map_or(vec![], |attachment_info| {
            let file_rows = attachment_info.select(attachment_row_selector);
            file_rows
                .map(|file_row| {
                    let mut children = file_row.child_elements();
                    let filename = children
                        .next()
                        .expect("Did not find filename")
                        .text()
                        .collect();
                    let download_link = children
                        .next()
                        .expect("Did not find download button")
                        .child_elements()
                        .next()
                        .expect("Did not find download link")
                        .attr("href")
                        .expect("Did not find download href");

                    File {
                        name: filename,
                        download_querypath: Some(download_link.to_string()),
                        date: None,
                        id: None,
                    }
                })
                .collect()
        });

        let submission_info = info_screens
            .iter()
            .find_map(|(screen, name)| {
                if ["Ihre Einreichung", "Your Submission"].contains(&name.as_str()) {
                    Some(screen)
                } else {
                    None
                }
            })
            .context("Did not find submission info")?;
        let submission_page_querypath = submission_info
            .select(submission_page_selector)
            .next()
            .map(|link| link.attr("href").expect("Could not find href in link"))
            .map(|querypath| querypath.to_string());

        Ok(Assignment {
            name,
            instructions,
            submission_date,
            attachments,
            submission: submission_page_querypath
                .map(|querypath| Submission::Unresolved(querypath)),
        })
    }
}

impl Assignment {
    pub fn is_active(&self) -> bool {
        self.submission_date > Local::now()
    }

    pub fn get_submission(&mut self, ilias_client: &IliasClient) -> Option<&AssignmentSubmission> {
        match self.submission.as_mut() {
            None => None,
            Some(submission) => Some({
                match submission {
                    Submission::Parsed(ass_sub) => ass_sub,
                    Submission::Unresolved(querypath) => {
                        let ass_sub = AssignmentSubmission::parse_submissions_page(
                            &ilias_client
                                .get_querypath(querypath)
                                .expect("Could not get submission page")
                                .root_element(),
                            ilias_client,
                        )
                        .expect("Could not parse submission page");
                        *submission = Submission::Parsed(ass_sub);
                        let ass_sub = match submission {
                            Submission::Parsed(ref x) => x,
                            _ => unreachable!(),
                        };

                        ass_sub
                    }
                }
            }),
        }
    }
}

#[derive(Debug)]
pub struct AssignmentSubmission {
    pub submissions: Vec<File>,
    delete_querypath: String,
    upload_querypath: String,
}

static UPLOAD_BUTTON_SELECTOR: OnceLock<Selector> = OnceLock::new();
static UPLOAD_FORM_SELECTOR: OnceLock<Selector> = OnceLock::new();
static DELETE_FORM_SELECTOR: OnceLock<Selector> = OnceLock::new();
static FILE_ROW_SELECTOR: OnceLock<Selector> = OnceLock::new();

impl AssignmentSubmission {
    fn parse_submissions_page(
        submission_page: &ElementRef,
        ilias_client: &IliasClient,
    ) -> Result<AssignmentSubmission> {
        let upload_button_selector = UPLOAD_BUTTON_SELECTOR.get_or_init(|| {
            Selector::parse(r#"nav div.navbar-header button"#).expect("Could not parse selector")
        });
        let upload_form_selector = UPLOAD_FORM_SELECTOR.get_or_init(|| {
            Selector::parse(r#"div#ilContentContainer form"#).expect("Could not parse selector")
        });
        let delete_form_selector = UPLOAD_BUTTON_SELECTOR.get_or_init(|| {
            Selector::parse(r#"div#ilContentContainer form"#).expect("Could not parse selector")
        });
        let file_row_selector = FILE_ROW_SELECTOR
            .get_or_init(|| Selector::parse(r#"form tbody tr"#).expect("Could not parse selector"));

        let file_rows = submission_page.select(file_row_selector);
        let uploaded_files = file_rows
            .map(|file_row| {
                let mut children = file_row.child_elements();

                let id = children
                    .next()
                    .expect("Did not find first column in table")
                    .child_elements()
                    .next()
                    .expect("Did not find checkbox")
                    .attr("value")
                    .expect("Did not find id");
                let file_name = children
                    .next()
                    .expect("Did not find second column")
                    .text()
                    .collect();
                let submission_date = parse_date(
                    &children
                        .next()
                        .expect("Did not find third column")
                        .text()
                        .collect::<String>(),
                )
                .expect("Could not parse date");
                let download_querypath = children
                    .next()
                    .expect("Did not find fourth column")
                    .child_elements()
                    .next()
                    .expect("Did not find download link")
                    .attr("href")
                    .expect("Did not find href attribute");

                File {
                    id: Some(id.to_string()),
                    name: file_name,
                    date: Some(submission_date),
                    download_querypath: Some(download_querypath.to_string()),
                }
            })
            .collect();

        let delete_querypath = submission_page
            .select(delete_form_selector)
            .next()
            .context("Did not find deltion form")?
            .value()
            .attr("action")
            .context("Did not find action attribute")?
            .to_string();

        let upload_form_querypath = submission_page
            .select(upload_button_selector)
            .next()
            .context("Did not find upload button")?
            .attr("data-action")
            .context("Did not find data-action on upload button")?;
        let upload_page = ilias_client.get_querypath(upload_form_querypath)?;
        let upload_querypath = upload_page
            .select(upload_form_selector)
            .next()
            .context("Did not find upload form")?
            .value()
            .attr("action")
            .context("Did not find action attribute")?
            .to_string();

        Ok(AssignmentSubmission {
            submissions: uploaded_files,
            delete_querypath,
            upload_querypath,
        })
    }

    pub fn delete_files<I: IntoIterator<Item = File>>(&self, ilias_client: &IliasClient, files: I) -> Result<()> {
        let mut form_args = files.into_iter().map(|file| file.id.expect("Files to delete must have an id")).map(|id| ("delivered[]", id)).collect::<Vec<_>>();
        form_args.push(("cmd[deleteDelivered]", String::from("Löschen")));

        ilias_client.push_querypath_form(&self.delete_querypath, &form_args)
    }

    pub fn upload_files<I: IntoIterator<Item = FileData>>(&self, ilias_client: &IliasClient, files: I) -> Result<()> {
        let mut form = Form::new();


        for (index, file_data) in files.into_iter().enumerate() {
            form = form.file_with_name(
                format!("deliver[{}]", index),
                file_data.path,
                file_data.name,
            )?
            .text("cmd[uploadFile]", "Hochladen")
            .text("ilfilehash", "aaaa");
        }

        ilias_client.push_querypath_multipart(&self.delete_querypath, form)
        // TODO: Maybe push files to submission here
    }
}
