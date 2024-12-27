use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

#[derive(Deserialize, Serialize, Debug)]
struct RawFeedInput {
    name: String,
    url: String,
    category: String,
}

#[derive(Deserialize, Serialize, Debug, FromRow)]
pub struct RawFeed {
   pub id: i32,
   pub name: String,
   pub url: String,
   pub category: String,
}

pub struct FeedDataSource {
    pool: PgPool,
}

impl FeedDataSource {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_raw_feeds(&self) -> Result<Vec<RawFeed>, anyhow::Error> {
        let res = match sqlx::query_as::<_, RawFeed>(
            "SELECT feeds.id, feeds.name, feeds.url, categories.name AS category
            FROM feeds
            INNER JOIN categories
            ON
            feeds.category_id = categories.id;",
        )
        .fetch_all(&self.pool)
        .await
        {
            Ok(res) => res,
            Err(e) => anyhow::bail!("Failed to get raw feeds from db"),
        };
        Ok(res)
    }
}
