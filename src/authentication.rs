use crate::constants::ILIAS_URL;
use reqwest::Client;

pub async fn authenticate(client: Client) -> Result<(), Box<dyn std::error::Error>> {
    println!("Authenticating!");

    let shib_url = format!("{}/shib_login.php", ILIAS_URL);

    let shib_post_params = [
        ("sendLogin", "1"),
        ("idp_selection", "https://idp.scc.kit.edu/idp/shibboleth"),
        ("il_target", ""),
        ("home_organization_selection", "Weiter"),
    ];
    let shib_post_redirect_response = client.post(shib_url).form(&shib_post_params).send().await?;
    let shib_post_redirect_url = shib_post_redirect_response.headers()["location"].to_str()?;

    if shib_post_redirect_url.starts_with(ILIAS_URL) {
        return Ok(());
    };

    let shib_get_response = client.post(shib_post_redirect_url).send().await?;
    let shib_get_redirect = shib_get_response.headers().get("location");

    if shib_get_redirect.is_some() {
        println!("Do something else!")
    }

    

    println!("{:?}", shib_get_response);
    println!("{:?}", shib_get_redirect);

    Ok(())
}
