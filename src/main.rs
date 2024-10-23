use std::{
    env,
    fmt::Display,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use dialoguer::{theme::ColorfulTheme, Confirm, MultiSelect, Password, Select};
use ilias::{
    client::IliasClient,
    exercise::{self, assignment::Assignment},
    folder::Folder,
    IliasElement,
};
use keyring::Entry;
use preselect_delete_setting::PreselectDeleteSetting;
use reqwest::{blocking::Client, Url};
use util::{UploadType, ILIAS_URL};

mod arguments;
mod authentication;
mod config;
mod ilias;
mod preselect_delete_setting;
mod transform;
mod uploaders;
mod util;

use crate::{
    arguments::Arguments,
    authentication::authenticate,
    config::Config,
    ilias::exercise::Exercise,
    transform::Transformer,
    uploaders::{file_data::FileData, upload_provider::UploadProvider},
};

fn main() -> Result<()> {
    let cli_args: Arguments = Arguments::parse();
    let config_file_content = search_config(&cli_args.search_depth);
    let file_config: Config = match config_file_content {
        Ok(content) => toml::from_str::<Config>(&content).context("Could not parse config")?,
        Err(_) => Config::default(),
    };

    let ilias_id = match cli_args.ilias_id {
        Some(id) => id,
        None => file_config.ilias_id.unwrap(),
    };

    let username = match cli_args.username {
        Some(user) => user,
        None => file_config.username.unwrap(),
    };

    let password = match cli_args.password {
        Some(pw) => pw,
        None => {
            let keyring_entry = Entry::new("ilias_uploader", &username).unwrap();

            let stored_password = keyring_entry.get_password();

            match stored_password {
                Ok(pw) => pw,
                Err(_) => {
                    let pw = Password::with_theme(&ColorfulTheme::default())
                        .with_prompt(format!("Ilias password for user: {}", &username))
                        .interact()
                        .unwrap();

                    if cli_args.store_password {
                        keyring_entry.set_password(&pw).unwrap();
                    }

                    pw
                }
            }
        }
    };

    let preselect_delete_setting = match cli_args.preselect_delete {
        Some(setting) => setting,
        None => match file_config.preselect_delete {
            Some(setting) => setting,
            None => preselect_delete_setting::PreselectDeleteSetting::SMART,
        },
    };

    let upload_type = match cli_args.upload_type {
        Some(upload_type) => upload_type,
        None => match file_config.upload_type {
            Some(upload_type) => upload_type,
            None => util::UploadType::EXERCISE,
        },
    };

    println!("Checking ilias {:?} {}", upload_type, ilias_id);

    let ilias_client = IliasClient::new(Url::parse(ILIAS_URL)?)?;
    ilias_client.authenticate(&username, &password);
    let reqwest_client = Client::builder().cookie_store(true).build().unwrap();
    authenticate(&reqwest_client, &username, &password).unwrap();

    let transform_regex = file_config.transform_regex;
    let transform_format = file_config.transform_format;

    let transformer = Transformer::new(transform_regex, transform_format)?;

    let transformed_file_data = cli_args
        .file_paths
        .iter()
        .map(|path| FileData {
            name: match match &transformer {
                Some(transformer) => transformer.transform(path),
                None => None,
            } {
                Some(transformed) => transformed,
                None => Path::new(path)
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
            },
            path: path.to_string(),
        })
        .collect::<Vec<_>>();

    match upload_type {
        util::UploadType::EXERCISE => {
            let exercise = Exercise::parse(
                ilias_client
                    .get_querypath(&Exercise::querypath_from_id(&ilias_id))?
                    .root_element(),
                &ilias_client,
            )?;

            let mut active_assignments = exercise
                .assignments
                .into_iter()
                .filter(Assignment::is_active)
                .collect::<Vec<_>>();
            let selected_index = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Assignment to upload to:")
                .default(0)
                .items(
                    &active_assignments
                        .iter()
                        .map(|assignment| &assignment.name)
                        .collect::<Vec<_>>(),
                )
                .interact()
                .unwrap();

            let selected_assignment = &mut active_assignments[selected_index];
            let selected_submission = selected_assignment
                .get_submission(&ilias_client)
                .context("Assignment did not have a submission")?;
            upload_files(
                &ilias_client,
                selected_submission,
                &transformed_file_data,
                upload_type,
                preselect_delete_setting,
            )
        }
        util::UploadType::FOLDER => {
            let folder = Folder::parse(
                ilias_client
                    .get_querypath(&Folder::querypath_from_id(&ilias_id))?
                    .root_element(),
                &ilias_client,
            )?;
            upload_files(
                &ilias_client,
                &folder,
                &transformed_file_data,
                upload_type,
                preselect_delete_setting,
            )
        }
    }
}

fn upload_files<T: UploadProvider>(
    ilias_client: &IliasClient,
    target: &T,
    transformed_files: &[FileData],
    upload_type: UploadType,
    preselect_delete_setting: PreselectDeleteSetting,
) -> Result<()>
where
    T::UploadedFile: Display,
{
    let existing_files = target.get_existing_files();
    if !existing_files.is_empty() {
        let delete = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(upload_type.get_delete_message())
            .default(true)
            .interact()
            .unwrap();

        if delete {
            let preselection = target.preselect_files(
                preselect_delete_setting,
                &transformed_files,
                existing_files,
            );

            let selection = MultiSelect::with_theme(&ColorfulTheme::default())
                .with_prompt("Which files do you want to delete")
                .items_checked(&preselection)
                .interact()?
                .into_iter()
                .map(|i| preselection[i].0)
                .collect::<Vec<_>>();

            target.delete_files(ilias_client, &selection)?;
        }
    }

    target.upload_files(ilias_client, transformed_files.clone())?;

    println!(
        "Uploaded {} successfully!",
        &transformed_files.iter()
            .map(|item| format!("{} as {}", item.path, item.name))
            .collect::<Vec<String>>()
            .join(", ")
    );
    Ok(())
}

const CONFIG_FILE_NAME: &str = ".ilias_upload";

fn search_config(depth: &i16) -> Result<String> {
    let mut current_dir = env::current_dir()?;
    if contains_config_file(&current_dir)? {
        return match fs::read_to_string(current_dir.join(CONFIG_FILE_NAME)) {
            Ok(file) => Ok(file),
            Err(_) => Err(anyhow!("Could not read config file")),
        };
    }

    for _ in 0..(depth - 1) {
        current_dir.pop();
        if contains_config_file(&current_dir)? {
            return match fs::read_to_string(current_dir.join(CONFIG_FILE_NAME)) {
                Ok(file) => Ok(file),
                Err(_) => Err(anyhow!("Could not read config file")),
            };
        }
    }

    Err(anyhow!("Could not find config file"))
}

fn contains_config_file(path: &PathBuf) -> Result<bool> {
    let found = path
        .read_dir()?
        .into_iter()
        .map(|file_res| match file_res {
            Ok(file) => file.file_name(),
            Err(_) => "".into(),
        })
        .any(|file_name| file_name == CONFIG_FILE_NAME);
    return Ok(found);
}
