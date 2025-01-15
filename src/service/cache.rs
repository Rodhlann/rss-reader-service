use anyhow::Context;
use shuttle_runtime::SecretStore;
use sqlx::PgPool;
use tokio::time::{self, Duration};

use crate::data::CacheDataSource;

pub async fn schedule_cache_clear(pool: PgPool, secrets: &SecretStore) -> Result<(), anyhow::Error> {
    let cache_duration: i32 = SecretStore::get(secrets, "CACHE_DURATION_MINS")
        .context("Missing expected ENV_VAR: CACHE_DURATION_MINS")?
        .parse::<i32>()
        .context("CACHE_DURATION_MINS is not a valid integer")?;

    println!("Scheduling cache clear job for once every [{}] mins", cache_duration);

    let mut interval = time::interval(Duration::from_secs(cache_duration as u64 * 60));

    let cache = CacheDataSource::new(pool);
    loop {
        interval.tick().await;
        println!("Attempting to clear cache");
        cache.cache_clear(cache_duration)
            .await
            .inspect_err(|e| { eprintln!("Database error: {}", e); })
            .context("Failed to clear cache")?;
    }
}
