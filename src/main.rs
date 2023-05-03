use reqwest::{Client, redirect::{self, Policy}, cookie::Jar};
mod authentication;
mod constants;
use crate::authentication::authenticate;


#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let log_redirects = redirect::Policy::custom(|attempt| {
        println!("{}, Location: {:?}", attempt.status(), attempt.url());
        if attempt.previous().len() >= 0 {
            attempt.stop()
        } else {
            attempt.follow()
        }
    });

    let reqwest_client = Client::builder().redirect(Policy::none()).cookie_store(true).build().unwrap();

    authenticate(reqwest_client).await;
}
