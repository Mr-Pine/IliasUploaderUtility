use reqwest::blocking::Client;
use scraper::Selector;
mod authentication;
mod util;
mod course;
mod excercise;
use crate::{authentication::authenticate, course::Course};

fn main() {
    println!("Hello, world!");

    let reqwest_client = Client::builder().cookie_store(true).build().unwrap();

    let username = include_str!("username.txt");
    let password = include_str!("password.txt");

    authenticate(&reqwest_client, username, password).unwrap();

    let target_id = "2107145";
    let target = Course::from_id(&reqwest_client, target_id, "test_HM").unwrap();

    let active_excercise = target.excercises.iter().find(|&excercise| excercise.active);

    //active_excercise.unwrap().delete_all_files(reqwest_client);
    active_excercise.unwrap().upload_files(&reqwest_client, vec!["test1.txt", "test2.txt"]);
}
