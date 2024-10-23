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

        Ok(Some(Transformer {
            regex,
            format: format.context("No transform format string provided")?,
        }))
    }

    pub fn transform(
        &self,
        filename: &str,
    ) -> Option<String> {
        let matches = self.regex.is_match(filename);
        if !matches {
            return None;
        }
        let transformed_filename = self.regex.replace_all(
            filename,
            self.format.clone(),
        );
    
        Some(transformed_filename.into_owned())
    }
        
}

