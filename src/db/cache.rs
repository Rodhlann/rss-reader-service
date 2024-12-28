use anyhow::Context;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

#[derive(Deserialize, Serialize, Debug)]
pub struct CachedFeedInput {
    pub name: String,
    pub json_string: String,
}

#[derive(Deserialize, Serialize, Debug, FromRow)]
pub struct CachedFeed {
    pub name: String,
    pub json_string: String,
    pub created_date: DateTime<Utc>,
}

pub struct CacheDataSource {
    pool: PgPool,
}

impl CacheDataSource {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_cached_feed(self, feed_name: &str) -> Result<Option<CachedFeed>, anyhow::Error> {
        let res = sqlx::query_as::<_, CachedFeed>(
            "SELECT * FROM cache where name = $1;"
        ).bind(feed_name)
        .fetch_optional(&self.pool)
        .await
        .context(format!("Failed to get cached feed: {}", feed_name))?;

        Ok(res)
    }

    pub async fn cache_feed(self, input: CachedFeedInput) -> Result<(), anyhow::Error> {
        sqlx::query(
            "INSERT INTO cache (name, json_string) VALUES ($1, $2);"
        )
        .bind(&input.name)
        .bind(&input.json_string)
        .execute(&self.pool)
        .await
        .context(format!("Failed to cache {}", input.name))?;

        Ok(())
    }
}
