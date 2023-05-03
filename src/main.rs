use reqwest::{Client, redirect};
mod authentication;
mod constants;
use crate::authentication::authenticate;


#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let log_redirects = redirect::Policy::custom(|attempt| {
        println!("{}, Location: {:?}", attempt.status(), attempt.url());
        if attempt.previous().len() >= 1 {
            attempt.stop()
        } else {
            attempt.follow()
        }
    });

    let reqwest_client = Client::builder().redirect(log_redirects).cookie_store(true).build().unwrap();

    authenticate(reqwest_client).await;
}
