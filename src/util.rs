use reqwest::Url;

pub const ILIAS_URL: &str = "https://ilias.studium.kit.edu";

#[macro_export]
macro_rules! ilias_url {
    ($id:tt, $target:expr) => {
        Url::parse(
            format!(
                "https://ilias.studium.kit.edu/goto.php?target={}_{}&client_id=produktiv",
               //https://ilias.studium.kit.edu/goto.php?target=fold_2240661&client_id=produktiv
                UploadType::ilias_target_identifier(&$target), $id
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

pub enum UploadType {
    EXERCISE, FOLDER
}

impl UploadType {
    pub fn ilias_target_identifier(self: &Self) -> &str{
        match self {
            UploadType::EXERCISE => "exc",
            UploadType::FOLDER => "fold",
        }
    }
}