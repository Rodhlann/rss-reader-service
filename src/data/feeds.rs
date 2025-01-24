use anyhow::Context;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

#[derive(Deserialize, Serialize, Debug)]
pub struct RawFeedInput {
    name: String,
    url: String,
    category: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RawFeedIdInput {
    id: i32
}

#[derive(Deserialize, Serialize, Debug, FromRow)]
pub struct RawFeedName {
    pub name: String
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
        let res = sqlx::query_as::<_, RawFeed>(
            "SELECT raw_feeds.id, raw_feeds.name, raw_feeds.url, categories.name AS category
            FROM raw_feeds
            INNER JOIN categories
            ON
            raw_feeds.category_id = categories.id;",
        )
        .fetch_all(&self.pool)
        .await
        .inspect_err(|e| { eprintln!("Database error: {:?}", e); })
        .context("Failed to get raw feeds from db")?;

        Ok(res)
    }

    pub async fn create_raw_feed(&self, input: RawFeedInput) -> Result<RawFeed, anyhow::Error> {
        println!("Creating new feed: {}", input.name);

        let new_category_id = sqlx::query_scalar(
            "INSERT INTO categories (name)
            VALUES ($1)
            ON CONFLICT (name) DO NOTHING
            RETURNING id"
        )
        .bind(&input.category)
        .fetch_optional(&self.pool)
        .await?;

        let category_id: i32 = if let Some(id) = new_category_id {
            id
        } else {
            sqlx::query_scalar("SELECT id FROM categories WHERE name = $1")
                .bind(&input.category)
                .fetch_one(&self.pool)
                .await
                .inspect_err(|e| { eprintln!("Database error: {:?}", e); })
                .context("Failed to fetch existing category ID")?
        };

        sqlx::query(
            "INSERT INTO raw_feeds (name, url, category_id)
                VALUES ($1, $2, $3)"
        )
        .bind(&input.name)
        .bind(&input.url)
        .bind(category_id)
        .execute(&self.pool)
        .await
        .inspect_err(|e| { eprintln!("Database error: {:?}", e); })
        .context("Failed to create new feed")?;

        let res = sqlx::query_as::<_, RawFeed>(
            "SELECT raw_feeds.id, raw_feeds.name, raw_feeds.url, categories.name AS category
            FROM raw_feeds
            INNER JOIN categories
            ON
            raw_feeds.category_id = categories.id
            WHERE raw_feeds.name = $1;"
        )
        .bind(&input.name)
        .fetch_one(&self.pool)
        .await
        .inspect_err(|e| { eprintln!("Database error: {:?}", e); })
        .context(format!("Error while getting feed: {}", input.name))?;

        Ok(res)
    }

    pub async fn delete_raw_feed(&self, id: i32) -> Result<(), anyhow::Error> {
        let res = sqlx::query_as::<_, RawFeedName>(
            "WITH deleted_row as (
                DELETE FROM raw_feeds WHERE id = $1
                RETURNING name
            )
            SELECT name FROM deleted_row;"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .inspect_err(|e| { eprintln!("Database error: {:?}", e); })
        .context(format!("Error while deleting raw feed: {}", id))?;

        println!("Successfully deleted feed: {}", res.name);

        Ok(())
    }
}
