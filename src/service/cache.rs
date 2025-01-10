use anyhow::Context;
use sqlx::PgPool;
use tokio::time::{self, Duration};

use crate::data::CacheDataSource;

pub async fn schedule_cache_clear(pool: PgPool) -> Result<(), anyhow::Error> {
    println!("Scheduling cache clear job");

    let mut interval = time::interval(Duration::from_secs(10 * 60));

    let cache = CacheDataSource::new(pool);
    loop {
        interval.tick().await;
        println!("Attempting to clear cache");
        cache.cache_clear()
            .await
            .inspect_err(|e| { eprintln!("Database error: {}", e); })
            .context("Failed to clear cache")?;
    }
}
