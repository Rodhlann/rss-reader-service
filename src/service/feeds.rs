use axum::{extract::State, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{ db::FeedDataSource, error::ServiceError, AppState };

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
pub async fn get_feeds() -> Result<impl IntoResponse, ServiceError> {
    let feeds: Vec<Feed> = vec![Feed {
        name: "Test".to_string(),
        category: "Test".to_string(),
        entries: vec!(Entry { title: "Test".to_string(), url: "Test".to_string(), created_date: "Test".to_string() })
    }];
    Ok(Json(json!(feeds)))
}

pub async fn get_raw_feeds(
    State(state): State<AppState>
) -> Result<impl IntoResponse, ServiceError> {
    let feed_data_source = FeedDataSource::new(state.pool.clone());
    let raw_feeds = feed_data_source.get_raw_feeds().await?;
    Ok(Json(json!(raw_feeds)))
}
