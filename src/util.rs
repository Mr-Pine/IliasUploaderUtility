use clap::ValueEnum;
use reqwest::Url;
use serde::Deserialize;

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

pub trait Querypath {
    fn get_querypath(&self) -> String;
    fn set_querypath(self: &mut Self, querypath: &str);
}

impl Querypath for Url {
    fn get_querypath(&self) -> String {
        format!("{}?{}", self.path(), self.query().unwrap_or(""))
    }

    fn set_querypath(self: &mut Self, querypath: &str) {
        let mut parts = querypath.split("?");
        self.set_path(parts.next().unwrap());
        self.set_query(parts.next());
    }
}

#[derive(Debug, Deserialize, Clone, ValueEnum, PartialEq)]
#[clap(rename_all = "kebab_case")]
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

    pub fn get_delete_message(self: &Self) -> &str{
        match self {
            UploadType::EXERCISE => "This excercise already has uploaded files. Do you want to delete them?",
            UploadType::FOLDER => "There are files with the same name in this folder. Do you want to delete them?",
        }
    }
}
