use reqwest::Url;

pub const ILIAS_URL: &str = "https://ilias.studium.kit.edu";

#[macro_export]
macro_rules! ilias_url {
    ($id:tt) => {
        Url::parse(
            format!(
                "https://ilias.studium.kit.edu/goto.php?target=exc_{}&client_id=produktiv",
                $id
            )
            .as_str(),
        )
    };
}

pub fn set_querypath(mut url: Url, querypath: &str) -> Url {
    let mut parts = querypath.split("?");
    url.set_path(parts.next().unwrap());
    url.set_query(parts.next());

    url
}
