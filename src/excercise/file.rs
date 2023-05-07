use scraper::{Selector, Html};

#[derive(Debug)]
pub struct File {
    pub name: String,
    pub id: String
}

impl File {
    pub fn parse_uploaded_files(page: &Html) -> Vec<File> {
        let file_row_selector = Selector::parse(r#"form tbody tr"#).unwrap();
        let file_rows = page.select(&file_row_selector);
        
        let id_selector = Selector::parse(r#"input[type="checkbox"][name="delivered[]"]"#).unwrap();
        let name_selector = Selector::parse(r#"td:nth-child(2)"#).unwrap();
        
        file_rows.map(|file_row| {
            let file_id = file_row.select(&id_selector).next().unwrap().value().attr("value").unwrap();
            let file_name = file_row.select(&name_selector).next().unwrap().text().collect::<String>();

            File { name: file_name, id: file_id.to_string() }
        }).collect()
    }
}