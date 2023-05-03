use crate::constants::ILIAS_URL;
use reqwest::{Client, Url};
use scraper::Html;

pub async fn authenticate(client: Client) -> Result<(), Box<dyn std::error::Error>> {
    println!("Authenticating!");

    let shib_url = format!("{}/shib_login.php", ILIAS_URL);

    let shib_post_params = [
        ("sendLogin", "1"),
        ("idp_selection", "https://idp.scc.kit.edu/idp/shibboleth"),
        ("il_target", ""),
        ("home_organization_selection", "Weiter"),
    ];
    let shib_post_redirect_response = client.get(shib_url).form(&shib_post_params).send().await?;
    println!("post resp: {:?}", shib_post_redirect_response);
    let shib_post_redirect_url = Url::parse(shib_post_redirect_response.headers()["location"].to_str()?)?;
    println!("redirect Url: {:?}", shib_post_redirect_url);

    if shib_post_redirect_url.host_str().unwrap().starts_with(ILIAS_URL) {
        return Ok(());
    };

    let shib_get_response = client.post("https://idp.scc.kit.edu/idp/profile/SAML2/Redirect/SSO").form(
        &[
            ("SAMLRequest", "fVLLTsMwEPyVyPfGadpSajWRQnugUoGqCRy4IMfeEAvHDl6Hx9+TNgWVS4+rncfOaJfIG92yrPO12cN7B+iDr0YbZMdFQjpnmOWokBneADIvWJ7dbVkcRqx11lthNQkyRHBeWbOyBrsGXA7uQwl43G8TUnvfIqNUacUxRN9J1TXhm/IhyI7mtSpLq8HXIaKlB/GY7h7yggTr/hpl+EH3TEW2IQrxx+9n2h9SKQ0n8h6kciA8zfMHEmzWCXmpuByLMUSyiiWvxGIB1+N5GU/npawWV9N5D0PsYGPQc+MTEkfxZBTNRtGkiCcsmrFp/EyC3SnvjTJSmdfL5ZQDCNltUexGQ6AncHgM0wNIujxUzI7G7qz0y7L8t2mSXu4V2yU9MxjcWnbfK27WO6uV+A4yre3nygH3kJAxoelA+f8P6Q8="),
            ("RelayState", "https://ilias.studium.kit.edu/shib_login.php"),
            ("SigAlg", "http://www.w3.org/2001/04/xmldsig-more#rsa-sha256"),
            ("Signature", "hH/YVCCNJ10Je6ysXUiz4igShZ36+6NsIkeNuerMNFCPpdHqMSHF04HO0UvJce0AuabKl+7Opo5uhsZtZcWqjJ8c7cnRHZQNnQ6XZ2MWfSFSQ1UiIaKtE6q+OvZ72gc/uM8w0PeLrB8OggH9vmwcXd7BxhnTx27KZtU2Y44pU2LFx9Pad7JyaYlwYnI2GrTAUYx+ix/eTxDVXq4AKjHLpY2eSRafZk0dzmKbseOfgreFQoYfFXCVtFCggkDxrWL30ChsukTWsyfv+TgrROBfyG80nn9/SmG3bVs/YlEbJ3kWUkQF3e6e/O3aFgf7SQzCrVvGVgpES/XmS79hpeXYzm+H9nDQeO80Ac4CfwVkGDtFLptkH7u5SmnFvNWO3zEPXcN0LETtBBeuruv/VJQDwZd4ThDTlt24zqOXp0UY+7uNdwZCBy84F2XPLYsFZRGKBRQcJrSKv19BkSAVvvLrsxvvpIIlBX3U7omhCo2EVOlAb2Jw2GlBGsTji6RRtHrcTxbDKZNytHhyGjAvR+jM5S+fqcMbtQHBAAkVG9Q7K1NDDwsGcNHxalSjaTSDXCpEBlhr4s9E5VEeai5lpBVCEkM7JDAWFs2E8+xJbsoDzpfMG2vCEHCyEyWtR9VYfPEsyEsGWD9BBiYBuMeWAAddIIPa9B7yLvhDX9IuucXG5OA=")
        ]
    ).send().await?;
    println!("get resp: {:?}", shib_get_response);
    //println!("get resp text: {:?}", shib_get_response.text().await);
    /* let get_headers = shib_get_response.headers().to_owned();
    let shib_get_redirect = get_headers.get("location");

    let html;
    
    if shib_get_redirect.is_none() {
        println!("Do something else!");
        let html_raw = shib_get_response.text().await;
        html = Html::parse_document(html_raw?.as_str());
    } else {
        let redirect_url = shib_get_redirect.unwrap().to_str();
    } */




    Ok(())
}
