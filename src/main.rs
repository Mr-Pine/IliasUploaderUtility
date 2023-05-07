use reqwest::Client;
mod authentication;
mod util;
mod course;
mod excercise;
use crate::{authentication::authenticate, course::Course};


#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let reqwest_client = Client::builder().cookie_store(true).build().unwrap();

    let username = include_str!("username.txt");
    let password = include_str!("password.txt");

    authenticate(&reqwest_client, username, password).await.unwrap();

    let target_id = "2107145";
    let target = Course::from_id(&reqwest_client, target_id, "test_HM").await.unwrap();
}
