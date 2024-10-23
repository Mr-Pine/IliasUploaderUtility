use std::{
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use dialoguer::{theme::ColorfulTheme, Confirm, Password, Select};
use keyring::Entry;
use preselect_delete_setting::PreselectDeleteSetting;
use reqwest::blocking::Client;
use util::UploadType;

mod arguments;
mod authentication;
mod config;
mod preselect_delete_setting;
mod transform;
mod uploaders;
mod util;
mod ilias;

use crate::{
    arguments::Arguments,
    authentication::authenticate,
    config::Config,
    ilias::exercise::Exercise,
    transform::Transformer,
    uploaders::{file_data::FileData, ilias_folder::IliasFolder, upload_provider::UploadProvider},
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

    let reqwest_client = Client::builder().cookie_store(true).build().unwrap();
    authenticate(&reqwest_client, &username, &password).unwrap();

    let transform_regex = file_config.transform_regex;
    let transform_format = file_config.transform_format;

    let transformer = Transformer::new(transform_regex, transform_format)?;

    let transformed_file_data = cli_args.file_paths.iter().map(|path| FileData {
        name: match match &transformer {
            Some(transformer) => transformer.transform(path),
            None => None,
        } {
            Some(transformed) => transformed,
            None => Path::new(path)
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned()
        },
        path: path.to_string(),
    });

    match upload_type {
        util::UploadType::EXERCISE => {
            let course = Exercise::from_id(&reqwest_client, &ilias_id, "unknown").unwrap();

            let active_excercises: Vec<_> = course
                .assignments
                .iter()
                .filter(|&excercise| excercise.active)
                .collect();

            let selected_index = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Excercise to upload to:")
                .default(0)
                .items(
                    &active_excercises
                        .iter()
                        .map(|excercise| &excercise.name)
                        .collect::<Vec<_>>(),
                )
                .interact()
                .unwrap();

            let selected_excercise = active_excercises[selected_index];
            upload_files(
                &reqwest_client,
                selected_excercise,
                transformed_file_data,
                upload_type,
                preselect_delete_setting,
            )
        }
        util::UploadType::FOLDER => {
            let target = IliasFolder::from_id(&reqwest_client, &ilias_id)?;
            upload_files(
                &reqwest_client,
                &target,
                transformed_file_data,
                upload_type,
                preselect_delete_setting,
            )
        }
    }
}

fn upload_files<T: UploadProvider, I: Iterator<Item = FileData>>(
    client: &Client,
    target: &T,
    transformed_files: I,
    upload_type: UploadType,
    preselect_delete_setting: PreselectDeleteSetting,
) -> Result<()>
where
    I: Clone,
{
    let conflicting_files =
        target.get_conflicting_files(&client, transformed_files.clone().map(|data| data.name));
    if !conflicting_files.is_empty() {
        let delete = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(upload_type.get_delete_message())
            .default(true)
            .interact()
            .unwrap();

        if delete {
            let selection = target.select_files_to_delete(
                preselect_delete_setting,
                &transformed_files,
                conflicting_files.as_slice(),
            );
            target.delete_files(&client, selection?)?;
        }
    }

    target.upload_files(&client, transformed_files.clone())?;

    println!(
        "Uploaded {} successfully!",
        &transformed_files
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
