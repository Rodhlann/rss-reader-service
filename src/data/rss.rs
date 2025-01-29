use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{de, Deserialize, Deserializer, Serialize};
use serde_json::{from_value, Value};

#[derive(Deserialize, Serialize, Debug)]
pub struct RSSLink {
    #[serde(rename = "@href")]
    pub href: String,
    #[serde(rename = "@type", default)]
    pub link_type: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RSSItem {
    #[serde(deserialize_with = "link")]
    pub link: String,
    #[serde(rename = "pubDate", deserialize_with = "updated_date_time")]
    pub pub_date: DateTime<Utc>,
    #[serde(deserialize_with = "title")]
    pub title: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RSSChannel {
    pub item: Vec<RSSItem>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RSSRoot {
    pub channel: RSSChannel,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RSSObject {
    pub rss: RSSRoot,
}

//  "link": [{"@href": String("https://letscast.fm/podcasts/rust-in-production-82281512/feed"), "@rel": String("self"), "@title": String("Rust in Production"), "@type": String("application/rss+xml")},
//  {"@href": String("https://letscast.fm/podcasts/rust-in-production-82281512/feed"), "@rel": String("first")}, String("https://corrode.dev/podcast")]

fn link<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum LinkElement {
        Complex(RSSLink),
        Simple(String),
    }

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum LinkMultiType {
        Vec(Vec<LinkElement>),
        Single(LinkElement),
        String(String),
    }

    match LinkMultiType::deserialize(deserializer)? {
        LinkMultiType::Vec(v) => {
            if let Some(link) = v.iter().find_map(|element| match element {
                LinkElement::Complex(link) if link.link_type == "text/html" => {
                    Some(link.href.clone())
                }
                _ => None,
            }) {
                return Ok(link);
            }

            for element in v {
                match element {
                    LinkElement::Simple(s) => return Ok(s),
                    LinkElement::Complex(link) => return Ok(link.href),
                }
            }

            Err(serde::de::Error::custom("No valid link found in array"))
        }
        LinkMultiType::Single(element) => match element {
            LinkElement::Complex(link) => Ok(link.href),
            LinkElement::Simple(s) => Ok(s),
        },
        LinkMultiType::String(s) => Ok(s),
    }
}

fn title<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum TitleMultiType {
        Vec(Vec<String>),
        Single(String),
    }

    match TitleMultiType::deserialize(deserializer)? {
        TitleMultiType::Vec(m) => Ok(m[0].clone()),
        TitleMultiType::Single(title) => Ok(title),
    }
}

fn updated_date_time<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    // Define multiple date formats
    let formats = [
        "%a, %d %b %Y %H:%M:%S %z",  // Example: Wed, 11 Sep 2024 00:00:00 -0400
        "%a, %d %b %Y %H:%M:%S GMT", // Example: Tue, 03 Sep 2024 13:51:48 GMT
        "%a, %d %b %Y %H:%M:%S UTC", // Example: Tue, 26 Nov 2024 17:21:05 UTC
    ];

    for format in &formats {
        if let Ok(dt) = NaiveDateTime::parse_from_str(&s, format) {
            return Ok(dt.and_utc());
        }
    }

    Err(de::Error::custom(&format!(
        "Failed to parse RSS date: {}",
        &s
    )))
}

pub fn rss_to_json(value: Value) -> Result<RSSObject, anyhow::Error> {
    from_value(value).map_err(|e| anyhow::Error::from(e))
}
