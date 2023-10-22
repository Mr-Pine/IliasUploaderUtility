use clap::ValueEnum;
use serde::Deserialize;

#[derive(Debug, Clone, ValueEnum, Deserialize, PartialEq)]
#[clap(rename_all = "kebab_case")]
pub enum PreselectDeleteSetting {
    ALL, SMART, NONE
}