use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use futures::future;
use serde::{Deserialize, Serialize};

use crate::{
    data::{
        CacheDataSource, CachedFeed, Duration, FeedDataSource, RawFeed, RawFeedInput, XmlDataSource,
    },
    error::ServiceError,
    AppState,
};

#[derive(Deserialize, Serialize, Debug)]
struct Entry {
    title: String,
    url: String,
    created_date: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct Feed {
    name: String,
    category: String,
    entries: Vec<Entry>,
}

#[derive(Deserialize, Debug)]
pub struct FeedsParam {
    pub duration: Option<Duration>,
    pub max_entries: Option<usize>,
}

#[axum::debug_handler]
pub async fn get_feeds(
    State(state): State<AppState>,
    Query(params): Query<FeedsParam>,
) -> Result<impl IntoResponse, ServiceError> {
    let feed_data_source = FeedDataSource::new(state.pool.clone());
    let raw_feeds = feed_data_source.get_raw_feeds().await?;

    let duration = params.duration.unwrap_or(Duration::WEEK);
    let max_entries = params.max_entries.unwrap_or(5);

    let mut cached_feeds: Vec<CachedFeed> = Vec::new();
    let mut new_feeds: Vec<RawFeed> = Vec::new();
    for raw_feed in raw_feeds {
        if let Some(cached_feed) = CacheDataSource::new(state.pool.clone())
            .get_cached_feed(&raw_feed.name, duration, max_entries)
            .await?
        {
            cached_feeds.push(cached_feed);
        } else {
            new_feeds.push(raw_feed)
        }
    }

    let futures = new_feeds
        .into_iter()
        .map(|raw_feed| async move {
            let result = XmlDataSource::get(&raw_feed.url).await;
            (raw_feed, result)
        })
        .collect::<Vec<_>>();

    let results: Vec<(RawFeed, Result<String, anyhow::Error>)> = future::join_all(futures).await;

    for (raw_feed, result) in results {
        let xml_string = match result {
            Ok(xml_string) => xml_string,
            Err(e) => return Err(ServiceError::from(anyhow::Error::msg(e))),
        };

        let feed = match XmlDataSource::parse_xml_string(
            &xml_string,
            &raw_feed.name,
            &raw_feed.category,
        ) {
            Ok(feed) => feed,
            Err(_) => {
                eprintln!("Failed to parse xml for feed: {}", raw_feed.name);
                continue;
            }
        };

        let datasource = CacheDataSource::new(state.pool.clone());
        match datasource.cache_feed(feed.clone()).await {
            Ok(_) => {
                let filtered_feed = datasource
                    .get_cached_feed(&raw_feed.name, duration, max_entries)
                    .await?;
                cached_feeds.push(
                    filtered_feed
                        .expect(format!("Cached feed '{}' not found", &raw_feed.name).as_str()),
                );
            }
            Err(e) => eprintln!("Failed to get feed: {:?}", e),
        };
    }
    Ok(Json(cached_feeds))
}

pub async fn get_categories(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ServiceError> {
    let categories = FeedDataSource::new(state.pool.clone())
        .get_categories()
        .await?;

    Ok(Json(categories))
}

pub async fn get_raw_feeds(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ServiceError> {
    let raw_feeds = FeedDataSource::new(state.pool.clone())
        .get_raw_feeds()
        .await?;

    Ok(Json(raw_feeds))
}

pub async fn create_raw_feed(
    State(state): State<AppState>,
    Json(body): Json<RawFeedInput>,
) -> Result<impl IntoResponse, ServiceError> {
    let raw_feed = FeedDataSource::new(state.pool.clone())
        .create_raw_feed(body)
        .await?;
    Ok(Json(raw_feed))
}

pub async fn batch_create_raw_feeds(
    State(state): State<AppState>,
    Json(body): Json<Vec<RawFeedInput>>,
) -> Result<impl IntoResponse, ServiceError> {
    let raw_feeds = FeedDataSource::new(state.pool.clone())
        .batch_create_raw_feeds(body)
        .await?;
    Ok(Json(raw_feeds))
}

pub async fn update_raw_feed(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(body): Json<RawFeedInput>,
) -> Result<impl IntoResponse, ServiceError> {
    FeedDataSource::new(state.pool.clone())
        .update_raw_feed(id, body)
        .await?;
    Ok(())
}

pub async fn delete_raw_feed(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ServiceError> {
    FeedDataSource::new(state.pool.clone())
        .delete_raw_feed(id)
        .await?;
    Ok(())
}
