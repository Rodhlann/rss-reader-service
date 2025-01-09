use anyhow::Context;
use chrono::DateTime;
use quickxml_to_serde::{xml_string_to_json, Config};

use super::{atom::atom_to_json, rss::rss_to_json, CachedEntry, CachedFeed};

pub struct XmlDataSource;

impl XmlDataSource {
    pub async fn get(url: &str) -> Result<String, anyhow::Error> {
        reqwest::get(url)
            .await
            .inspect_err(|e| { eprintln!("GET request error: {:?}", e) })
            .context("Failed to request feed data")?
            .text()
            .await
            .inspect_err(|e| { eprintln!("XML response parsing error: {:?}", e) })
            .context("Failed to parse xml response")
    }

    pub fn parse_xml_string(xml_string: &str, name: &str, category: &str) -> Result<CachedFeed, anyhow::Error> {
        if xml_string.contains("<rss") {
            parse_rss(xml_string, name, category)
        } else if xml_string.contains("<feed") {
            parse_atom(xml_string, name, category)
        } else {
            anyhow::bail!("Unknown feed syntax".to_string())
        }
    }
}

fn parse_atom(xml_string: &str, name: &str, category: &str) -> Result<CachedFeed, anyhow::Error> {
    let value = xml_string_to_json(xml_string.into(), &Config::new_with_defaults())?;
    let json = atom_to_json(value)?;

    let entries = json.feed.entry.into_iter().map(|entry| CachedEntry {
        title: entry.title,
        url: entry.link,
        created_date: entry.published
            .or(entry.updated)
            .unwrap_or(DateTime::UNIX_EPOCH)
    }).collect();

    Ok(CachedFeed {
        name: name.into(),
        category: category.into(),
        entries
    })
}

fn parse_rss(xml_string: &str, name: &str, category: &str) -> Result<CachedFeed, anyhow::Error> {
    let value = xml_string_to_json(xml_string.into(), &Config::new_with_defaults())?;
    let json = rss_to_json(value)?;

    let entries = json.rss.channel.item.into_iter().map(|entry| CachedEntry {
        title: entry.title,
        url: entry.link,
        created_date: entry.pub_date
    }).collect();

    Ok(CachedFeed {
        name: name.into(),
        category: category.into(),
        entries
    })
}
