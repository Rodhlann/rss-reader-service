use axum::{response::IntoResponse, Json};
use serde::Serialize;
use serde_json::json;

use crate::error::ServiceError;

#[derive(Serialize)]
struct Feed {
    name: String,
}

#[axum::debug_handler]
pub async fn get_feeds() -> Result<impl IntoResponse, ServiceError> {
    let feeds: Vec<Feed> = vec![Feed {
        name: "Test".to_string(),
    }];
    Ok(Json(json!(feeds)))
}
