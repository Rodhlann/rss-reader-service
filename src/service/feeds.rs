use axum::{extract::State, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{ db::{CacheDataSource, CachedFeedInput, FeedDataSource}, error::ServiceError, AppState };

#[derive(Deserialize, Serialize, Debug)]
struct Entry {
    title: String,
    url: String,
    created_date: String
}

#[derive(Deserialize, Serialize, Debug)]
struct Feed {
    name: String,
    category: String,
    entries: Vec<Entry>
}

#[axum::debug_handler]
pub async fn get_feeds(
    State(state): State<AppState>
) -> Result<impl IntoResponse, ServiceError> {
    let feed_data_source = FeedDataSource::new(state.pool.clone());
    let raw_feeds = feed_data_source.get_raw_feeds().await?;
    let mut feeds: Vec<Feed> = Vec::new();

    for raw_feed in raw_feeds {
        let cache_data_source = CacheDataSource::new(state.pool.clone());
        if let Some(cached_feed) = cache_data_source.get_cached_feed(&raw_feed.name).await? {
            feeds.push(Feed { name: cached_feed.name, category: "cached".to_string(), entries: vec!(Entry { title: "cached".to_string(), url: "cached".to_string(), created_date: "cached".to_string() }) });
        } else {
            feeds.push(Feed { name: "live".to_string(), category: "live".to_string(), entries: vec!(Entry { title: "live".to_string(), url: "live".to_string(), created_date: "live".to_string() }) });
            let cache_data_source = CacheDataSource::new(state.pool.clone());
            cache_data_source.cache_feed(CachedFeedInput { name: raw_feed.name, json_string: "json".to_string() }).await?;
        };
    }

    Ok(Json(json!(feeds)))
}

pub async fn get_raw_feeds(
    State(state): State<AppState>
) -> Result<impl IntoResponse, ServiceError> {
    let feed_data_source = FeedDataSource::new(state.pool.clone());
    let raw_feeds = feed_data_source.get_raw_feeds().await?;
    Ok(Json(json!(raw_feeds)))
}
