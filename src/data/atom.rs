use chrono::{DateTime, Utc};
use serde::{de, Deserialize, Deserializer, Serialize};
use serde_json::{from_value, Value};

#[derive(Deserialize, Serialize, Debug)]
pub struct AtomLink {
    #[serde(rename = "@href")]
    pub href: String,
    #[serde(rename = "@type")]
    pub link_type: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AtomTitle {
    #[serde(rename = "#text")]
    pub title: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AtomEntry {
    #[serde(deserialize_with = "link")]
    pub link: String,
    #[serde(deserialize_with = "updated_date_time")]
    pub updated: Option<DateTime<Utc>>,
    #[serde(deserialize_with = "updated_date_time", default)]
    pub published: Option<DateTime<Utc>>,
    #[serde(deserialize_with = "title")]
    pub title: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AtomRoot {
    pub entry: Vec<AtomEntry>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AtomFeed {
    pub feed: AtomRoot,
}

fn link<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum LinkMultiType {
        Vec(Vec<AtomLink>),
        Single(AtomLink),
    }

    match LinkMultiType::deserialize(deserializer)? {
        LinkMultiType::Vec(v) => {
            let link = v
                .iter()
                .find(|link| link.link_type == "text/html")
                .unwrap()
                .href
                .to_string();
            Ok(link)
        }
        LinkMultiType::Single(link) => Ok(link.href),
    }
}

fn title<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum TitleMultiType {
        Map(AtomTitle),
        Single(String),
    }

    match TitleMultiType::deserialize(deserializer)? {
        TitleMultiType::Map(m) => Ok(m.title),
        TitleMultiType::Single(title) => Ok(title),
    }
}

fn updated_date_time<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    // Atom Date: 2024-07-23T07:28:00+00:00
    if let Ok(dt) = DateTime::parse_from_rfc3339(&s) {
        return Ok(Some(dt.with_timezone(&Utc)));
    }

    Err(de::Error::custom(&format!(
        "Failed to parse Atom date: {}",
        &s
    )))
}

pub fn atom_to_json(value: Value) -> Result<AtomFeed, anyhow::Error> {
    from_value(value).map_err(|e| anyhow::Error::from(e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn assert_atom_feed_parsed(result: Result<AtomFeed, anyhow::Error>) {
        match result {
            Ok(feed) => {
                assert!(!feed.feed.entry.is_empty(), "Expected at least one entry");

                let first_entry = &feed.feed.entry[0];
                assert_eq!(
                    first_entry.link,
                    "https://technicalgrimoire.com/david/2024/10/keyburg-videogame"
                );
                assert_eq!(first_entry.title, "I Made a Terrible Video Game");
                assert_eq!(
                    first_entry.updated.unwrap().to_string(),
                    "2024-10-09 18:55:25 UTC"
                );
            }
            Err(e) => panic!("Parsing failed: {:?}", e),
        }
    }

    #[test]
    fn test_atom_to_json_no_published() {
        let data = json!({
          "feed": {
            "entry": [
              {
                "link": {
                  "@href": "https://technicalgrimoire.com/david/2024/10/keyburg-videogame",
                  "@rel": "alternate",
                  "@title": "I Made a Terrible Video Game",
                  "@type": "text/html"
                },
                "title": {
                  "#text": "I Made a Terrible Video Game",
                  "@type": "html"
                },
                "updated": "2024-10-09T18:55:25+00:00"
              }
            ],
          }
        });

        let result = atom_to_json(data);
        assert_atom_feed_parsed(result);
    }

    #[test]
    fn test_atom_to_json_title_map() {
        let data = json!({
          "feed": {
            "entry": [
              {
                "link": {
                  "@href": "https://technicalgrimoire.com/david/2024/10/keyburg-videogame",
                  "@rel": "alternate",
                  "@title": "I Made a Terrible Video Game",
                  "@type": "text/html"
                },
                "title": {
                  "#text": "I Made a Terrible Video Game",
                  "@type": "html"
                },
                "published": "2024-09-01T00:00:00+00:00",
                "updated": "2024-10-09T18:55:25+00:00"
              }
            ],
          }
        });

        let result = atom_to_json(data);
        assert_atom_feed_parsed(result);
    }

    #[test]
    fn test_atom_to_json_title_string() {
        let data = json!({
          "feed": {
            "entry": [
              {
                "link": {
                  "@href": "https://technicalgrimoire.com/david/2024/10/keyburg-videogame",
                  "@rel": "alternate",
                  "@title": "I Made a Terrible Video Game",
                  "@type": "text/html"
                },
                "title": "I Made a Terrible Video Game",
                "published": "2024-09-01T00:00:00+00:00",
                "updated": "2024-10-09T18:55:25+00:00"
              }
            ],
          }
        });

        let result = atom_to_json(data);
        assert_atom_feed_parsed(result);
    }

    #[test]
    fn test_atom_to_json_link_map() {
        let data = json!({
          "feed": {
            "entry": [
              {
                "link": {
                  "@href": "https://technicalgrimoire.com/david/2024/10/keyburg-videogame",
                  "@rel": "alternate",
                  "@title": "I Made a Terrible Video Game",
                  "@type": "text/html"
                },
                "title": "I Made a Terrible Video Game",
                "published": "2024-09-01T00:00:00+00:00",
                "updated": "2024-10-09T18:55:25+00:00"
              }
            ],
          }
        });

        let result = atom_to_json(data);
        assert_atom_feed_parsed(result);
    }

    #[test]
    fn test_atom_to_json_link_vec() {
        let data = json!({
          "feed": {
            "entry": [
              {
                "link": [
                  {
                    "@href": "https://technicalgrimoire.com/david/2024/10/keyburg-videogame",
                    "@rel": "alternate",
                    "@title": "I Made a Terrible Video Game",
                    "@type": "text/html"
                  }
                ],
                "title": "I Made a Terrible Video Game",
                "published": "2024-09-01T00:00:00+00:00",
                "updated": "2024-10-09T18:55:25+00:00"
              }
            ],
          }
        });

        let result = atom_to_json(data);
        assert_atom_feed_parsed(result);
    }
}
