use std::sync::Arc;

use anyhow::Context;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

#[derive(Clone, Deserialize, Serialize, Debug, FromRow)]
pub struct CachedEntry {
    pub title: String,
    pub url: String,
    pub created_date: DateTime<Utc>
}

#[derive(Deserialize, Serialize, Debug, FromRow)]
struct DBCachedFeed {
    id: i32,
    name: String,
    category: String,
    created_date: DateTime<Utc>, // Used to determine if cache is expired
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct CachedFeed {
    pub name: String,
    pub category: String,
    pub entries: Vec<CachedEntry>
}

#[derive(Deserialize, Serialize, Debug, FromRow)]
struct DBCachedFeedName {
    name: String,
}

pub struct CacheDataSource {
    pool: PgPool,
}

impl CacheDataSource {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_cached_feed(&self, feed_name: &str) -> Result<Option<CachedFeed>, anyhow::Error> {
        let cached_feed = sqlx::query_as::<_, DBCachedFeed>(
            r#"SELECT
                cached.id,
                cached.name,
                c.name as category,
                cached.created_date
            FROM cached_feeds cached
            JOIN categories c ON cached.category_id = c.id
            WHERE cached.name = $1;"#
        )
        .bind(feed_name)
        .fetch_optional(&self.pool)
        .await
        .inspect_err(|e| { eprintln!("Database error: {:?}", e); })
        .context(format!("Failed to get cached feed: {}", feed_name))?;

        if let Some(feed) = cached_feed {
            let entries = sqlx::query_as::<_, CachedEntry>(
                r#"SELECT
                    title,
                    url,
                    created_date
                FROM cached_entries
                WHERE feed_id = $1
                ORDER BY created_date DESC"#
            )
            .bind(feed.id)
            .fetch_all(&self.pool)
            .await
            .inspect_err(|e| { eprintln!("Database error: {:?}", e); })
            .context(format!("Failed to get cached entries for feed: {}", feed.name))?;

            Ok(Some(CachedFeed {
                name: feed.name,
                category: feed.category,
                entries
            }))
        } else {
            return Ok(None);
        }
    }

    pub async fn cache_feed(&self, input: CachedFeed) -> Result<(), anyhow::Error> {
        println!("Caching feed: {}", input.name);

        let mut tx = self.pool.begin().await
            .inspect_err(|e| { eprintln!("Database error: {:?}", e); })
            .context("Failed to start transaction")?;

        let category_id: i32 = sqlx::query_scalar("SELECT id FROM categories WHERE name = $1")
            .bind(&input.category)
            .fetch_one(&mut *tx)
            .await
            .inspect_err(|e| { eprintln!("Database error: {:?}", e); })
            .context("Failed to fetch existing category ID")?;

        let cached_feed_id: i32 = sqlx::query_scalar(
            "INSERT INTO cached_feeds (name, category_id)
            VALUES ($1, $2)
            RETURNING id"
        )
        .bind(&input.name)
        .bind(category_id)
        .fetch_one(&mut *tx)
        .await
        .inspect_err(|e| { eprintln!("Database error: {:?}", e) })
        .context(format!("Error while caching feed: {}", input.name))?;

        for entry in input.entries {
            sqlx::query(
                "INSERT INTO cached_entries (feed_id, title, url, created_date)
                VALUES ($1, $2, $3, $4)"
            )
            .bind(cached_feed_id)
            .bind(&entry.title)
            .bind(&entry.url)
            .bind(&entry.created_date)
            .execute(&mut *tx)
            .await
            .inspect_err(|e| { eprintln!("Database error: {:?}", e); })
            .context(format!("Failed to cache entry '{}' for feed: {}", &entry.title, &input.name))?;
        }

        tx.commit().await
            .inspect_err(|e| { eprintln!("Database error: {:?}", e); })
            .context("Failed to commit transaction")?;

        Ok(())
    }

    pub async fn cache_clear(&self) -> Result<(), anyhow::Error> {
        let stale_cache = sqlx::query_as::<_, DBCachedFeedName>(
            "SELECT * FROM cached_feeds
            WHERE created_date < NOW() - INTERVAL '10 minutes';"
        )
        .fetch_all(&self.pool)
        .await
        .inspect_err(|e| { eprintln!("Database error: {:?}", e); })
        .context("Failed to fetch stale cached feeds")?;

        let stale_names: Vec<String> = stale_cache.iter().map(|c| c.name.clone()).collect();
        if !stale_names.is_empty() {
            println!("Clearing stale cache items: [{}]", stale_names.join(", "));

            sqlx::query_as::<_, DBCachedFeed>(
                "DELETE FROM cached_feeds
                WHERE created_date < NOW() - INTERVAL '10 minutes';"
            )
            .fetch_all(&self.pool)
            .await
            .inspect_err(|e| { eprintln!("Database error: {:?}", e); })
            .context("Failed to clear stale cached feeds")?;
        }

        Ok(())
    }
}

