use std::error::Error;

use crate::util::ILIAS_URL;
use reqwest::Client;
use scraper::{Html, Selector};

pub async fn authenticate(
    client: &Client,
    username: &str,
    password: &str,
) -> Result<(), Box<dyn Error>> {
    println!("Authenticating!");

    let shib_url = format!("{}/shib_login.php", ILIAS_URL);

    let shib_params = [
        ("sendLogin", "1"),
        ("idp_selection", "https://idp.scc.kit.edu/idp/shibboleth"),
        ("il_target", ""),
        ("home_organization_selection", "Weiter"),
    ];
    let shib_login_page = client
        .post(shib_url.clone())
        .form(&shib_params)
        .send()
        .await?;

    let mut url = shib_login_page.url().to_owned();
    let is_ilias = url.as_str().starts_with(ILIAS_URL);
    if is_ilias {
        dbg!("Exiting auth, already logged in");
        return Ok(());
    }

    let shib_login_fragment = Html::parse_document(shib_login_page.text().await?.as_str());
    let csrf_selector = Selector::parse(r#"input[name="csrf_token"]"#)?;
    let crsf_field = shib_login_fragment.select(&csrf_selector).next();

    let shib_continue_fragment: Html;

    let path_selector = Selector::parse(r#"form[method="post"]"#)?;

    if crsf_field.is_some() {
        let crsf = crsf_field.unwrap().value().attr("value").unwrap();

        let form_data = [
            ("csrf_token", crsf),
            ("j_username", username),
            ("j_password", password),
            ("_eventId_proceed", ""),
        ];

        let post_path = shib_login_fragment
            .select(&path_selector)
            .next()
            .unwrap()
            .value()
            .attr("action")
            .unwrap();

        let mut parts = post_path.split("?");
        url.set_path(parts.next().unwrap());
        url.set_query(parts.next());
        let continue_response = client.post(url.clone()).form(&form_data).send().await?;

        shib_continue_fragment = Html::parse_document(continue_response.text().await?.as_str());
    } else {
        shib_continue_fragment = shib_login_fragment;
    }

    let saml_selector = Selector::parse(r#"input[name="SAMLResponse"]"#)?;
    let saml = shib_continue_fragment
        .select(&saml_selector)
        .next()
        .unwrap()
        .value()
        .attr("value")
        .unwrap();

    let continue_form_data = [("RelayState", shib_url.as_str()), ("SAMLResponse", saml)];

    let continue_url = shib_continue_fragment
        .select(&path_selector)
        .next()
        .unwrap()
        .value()
        .attr("action")
        .unwrap();

    let ilias_home = client.post(continue_url).form(&continue_form_data).send();

    if ilias_home.await?.status().is_success() {
        println!("Logged in!");
        Ok(())
    } else {
        Err("Ilias login not successful!".into())
    }
}
