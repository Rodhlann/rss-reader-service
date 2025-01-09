use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{de, Deserialize, Deserializer, Serialize};
use serde_json::{from_value, Value};

#[derive(Deserialize, Serialize, Debug)]
pub struct RSSItem {
    pub link: String,
    #[serde(rename = "pubDate", deserialize_with = "updated_date_time")]
    pub pub_date: DateTime<Utc>,
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

fn updated_date_time<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    // Define multiple date formats
    let formats = [
        "%a, %d %b %Y %H:%M:%S %z",  // Example: Wed, 11 Sep 2024 00:00:00 -0400
        "%a, %d %b %Y %H:%M:%S GMT", // Example: Tue, 03 Sep 2024 13:51:48 GMT
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
