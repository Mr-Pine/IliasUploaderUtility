use clap::Parser;

use crate::{preselect_delete_setting::PreselectDeleteSetting, util::UploadType};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Arguments {
    #[arg(required=true, num_args=1..)]
    pub file_paths: Vec<String>,

    #[arg(short, long)]
    pub ilias_id: Option<String>,

    #[arg(short('d'), long, default_value_t = 3)]
    pub search_depth: i16,

    #[arg(short, long)]
    pub username: Option<String>,

    #[arg(short, long)]
    pub password: Option<String>,

    #[arg(long, default_value_t = true)]
    pub store_password: bool,

    #[arg(long, value_enum)]
    pub preselect_delete: Option<PreselectDeleteSetting>,

    #[arg(long, value_enum)]
    pub upload_type: Option<UploadType>
}