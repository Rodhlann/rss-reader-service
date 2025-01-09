use crate::data::{CachedFeed, XmlDataSource};

pub async fn fetch_feed(name: &str, url: &str, category: &str) -> Result<CachedFeed, anyhow::Error> {
    let xml_string = XmlDataSource::get(url).await?;
    let feed = XmlDataSource::parse_xml_string(&xml_string, name, category)?;
    Ok(feed)
}
