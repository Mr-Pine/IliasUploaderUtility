use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use regex::Regex;

pub struct Transformer {
    regex: Regex,
    format: String,
}

impl Transformer {
    pub fn new(
        regex_string: Option<String>,
        format: Option<String>,
    ) -> Result<Option<Transformer>> {
        if regex_string.is_none() && format.is_none() {
            return Ok(None);
        }

        let regex = Regex::new(regex_string.context("No regex string provided")?.as_str())?;

        return Ok(Some(Transformer {
            regex: regex,
            format: format.context("No transform format string provided")?,
        }));
    }

    pub fn transform(
        self: &Self,
        filename: &str,
    ) -> String {
        let transformed_filename = self.regex.replace_all(
            filename,
            self.format.clone(),
        );
    
        return transformed_filename.into_owned();
    }

    pub fn transform_path<T: AsRef<Path>>(self: &Self, path: T) -> Result<PathBuf> {
        let path = path.as_ref();
        let filename = path.file_name().context("Unable to get filename")?;
        let transformed_filename = self.transform(filename.to_str().context("Can't transform filename to string")?);
        return Ok(path.with_file_name(transformed_filename));
    }
        
}

