use std::{env, fs, path::PathBuf};

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use dialoguer::{theme::ColorfulTheme, Confirm, Password, Select};
use keyring::Entry;
use reqwest::blocking::Client;

mod arguments;
mod authentication;
mod config;
mod course;
mod excercise;
mod transform;
mod util;
use crate::{
    arguments::Arguments, authentication::authenticate, config::Config, course::Course,
    excercise::FileData, transform::Transformer,
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

    println!("Checking ilias course {}", ilias_id);

    let reqwest_client = Client::builder().cookie_store(true).build().unwrap();
    authenticate(&reqwest_client, &username, &password).unwrap();
    let target = Course::from_id(&reqwest_client, &ilias_id, "unknown").unwrap();

    let active_excercises: Vec<_> = target
        .excercises
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

    // TODO: Select what to delete
    if selected_excercise.has_files {
        let delete = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("This excercise already has uploaded files. Do you want to delete them?")
            .default(true)
            .interact()
            .unwrap();

        if delete {
            selected_excercise.delete_all_files(&reqwest_client)
        }
    }

    let transform_regex = file_config.transform_regex;
    let transform_format = file_config.transform_format;

    let transformer = Transformer::new(transform_regex, transform_format)?;

    let transformed_file_paths = cli_args.file_paths.iter().map(|path| FileData {
        name: match &transformer {
            Some(transformer) => transformer.transform_path(path).unwrap().to_string_lossy().into_owned(),
            None => path.to_string(),
        },
        path: path.to_string(),
    });

    dbg!(&transformed_file_paths);
    dbg!(&cli_args.file_paths);

    selected_excercise.upload_files(&reqwest_client, transformed_file_paths);

    println!("Uploaded {} successfully!", cli_args.file_paths.join(", "));
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
