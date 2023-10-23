use std::{borrow::Cow, path::Path};

use anyhow::Result;
use reqwest::blocking::multipart::{Form, Part};

pub trait AddFileWithFilename {
    fn file_with_name<T, U, V>(self, name: T, path: U, filename: V) -> Result<Form>
    where
        T: Into<Cow<'static, str>>,
        U: AsRef<Path>,
        V: Into<Cow<'static, str>>;
}

impl AddFileWithFilename for Form {
    fn file_with_name<T, U, V>(self, name: T, path: U, filename: V) -> Result<Form>
    where
        T: Into<Cow<'static, str>>,
        U: AsRef<Path>,
        V: Into<Cow<'static, str>>
    {
        Ok(self.part(name, Part::file(path)?.file_name(filename)))
    }
}