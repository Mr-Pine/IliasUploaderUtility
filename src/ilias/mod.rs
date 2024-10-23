use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Days, Local, NaiveTime, TimeZone};
use client::IliasClient;
use regex::Regex;
use scraper::ElementRef;

pub mod exercise;
pub mod folder;
pub mod file;
pub mod client;

trait IliasElement: Sized {
    fn type_identifier() -> &'static str;

    fn parse(element: &ElementRef, ilias_client: &IliasClient) -> Result<Self>;
}

fn parse_date(date_string: &str) -> Result<DateTime<Local>> {
    let (date, time) = date_string.split_once(',').context(anyhow!("Could not separate date and time in {}", date_string))?;
    let date = date.trim();
    let time = time.trim();

    let time = NaiveTime::parse_from_str(time, "%H:%M:%S")?;

    let date = if ["Gestern", "Yesterday"].contains(&date) {
        Local::now() - Days::new(1)
    } else if ["Heute", "Today"].contains(&date) {
        Local::now()
    } else if ["Morgen", "Tomorrow"].contains(&date) {
        Local::now() + Days::new(1)
    } else {
        let months: [&[&str]; 11] = [&["Jan"], &["Feb"], &["Mär", "Mar"], &["Apr"], &["Mai", "May"], &["Jun"], &["Aug"], &["Sep"], &["Okt", "Oct"], &["Nov"], &["Dez", "Dec"]];

        let date_regex = Regex::new("^(?<day>\\d+)\\. (?<month>\\w+) (?<year>\\w+)$")?;
        let date_split = date_regex.captures(date_string).context(anyhow!("Could not match date {}", date_string))?;
        let (day, month, year) = (date_split.name("day").unwrap().as_str(), date_split.name("month").unwrap().as_str(), date_split.name("year").unwrap().as_str());
        let day: u32 = day.parse()?;
        let month = months.iter().enumerate().find_map(|(index, &names)| if names.contains(&month) {Some(index as u32)} else {None}).context(anyhow!("Could not parse month {}", month))?;
        let year: i32 = year.parse()?;

        Local.with_ymd_and_hms(year, month, day, 0, 0, 0).earliest().context("Could not construct date")?
    };

    let datetime =  date.with_time(time).earliest().context("Could not set time")?;
    Ok(datetime)
}
