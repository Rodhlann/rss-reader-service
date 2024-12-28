use axum::{extract::State, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{ db::{CacheDataSource, CachedFeedInput, FeedDataSource, RawFeedInput}, error::ServiceError, AppState };

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
        if let Some(cached_feed) = CacheDataSource::new(state.pool.clone()).get_cached_feed(&raw_feed.name).await? {
            // feeds.push(cached_feed.json_string);
        } else {
            // TODO fetch live feed data
            CacheDataSource::new(state.pool.clone())
                .cache_feed(CachedFeedInput { name: raw_feed.name, json_string: "json".to_string() })
                .await?;
            feeds.push(Feed { name: "live".to_string(), category: "live".to_string(), entries: vec!(Entry { title: "live".to_string(), url: "live".to_string(), created_date: "live".to_string() }) });
        };
    }

    Ok(Json(json!(feeds)))
}

pub async fn get_raw_feeds(
    State(state): State<AppState>
) -> Result<impl IntoResponse, ServiceError> {
    let raw_feeds = FeedDataSource::new(state.pool.clone())
        .get_raw_feeds()
        .await?;

    Ok(Json(json!(raw_feeds)))
}

pub async fn create_raw_feed(
    State(state): State<AppState>,
    Json(body): Json<RawFeedInput>
) -> Result<impl IntoResponse, ServiceError> {
   let raw_feed = FeedDataSource::new(state.pool.clone())
        .create_raw_feed(body)
        .await?;
    Ok(Json(raw_feed))
}
