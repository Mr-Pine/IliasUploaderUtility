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
    fn set_querypath(&mut self, querypath: &str);
}

impl Querypath for Url {
    fn get_querypath(&self) -> String {
        format!("{}?{}", self.path(), self.query().unwrap_or(""))
    }

    fn set_querypath(&mut self, querypath: &str) {
        let mut parts = querypath.split("?");
        self.set_path(parts.next().unwrap());
        self.set_query(parts.next());
    }
}

#[derive(Debug, Deserialize, Clone, ValueEnum, PartialEq)]
#[clap(rename_all = "kebab_case")]
pub enum UploadType {
    Exercise, Folder
}

impl UploadType {
    pub fn get_delete_message(&self) -> &str{
        match self {
            UploadType::Exercise => "This excercise already has uploaded files. Do you want to delete any of them?",
            UploadType::Folder => "There are already files in this folder. Do you want to delete any of them?",
        }
    }
}
