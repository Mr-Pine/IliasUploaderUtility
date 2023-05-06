use reqwest::Client;
mod authentication;
mod constants;
use crate::authentication::authenticate;


#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let reqwest_client = Client::builder().cookie_store(true).build().unwrap();

    let username = include_str!("username.txt");
    let password = include_str!("password.txt");

    authenticate(reqwest_client, username, password).await.unwrap();
}
