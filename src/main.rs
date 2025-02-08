use anyhow::Context;
use axum::{
    routing::{get, post},
    Router,
};
use shuttle_runtime::SecretStore;
use sqlx::PgPool;

mod service;
use service::{
    batch_create_raw_feeds, create_raw_feed, delete_raw_feed, get_categories, get_feeds,
    get_raw_feeds, schedule_cache_clear, update_raw_feed,
};

mod data;
mod error;

#[shuttle_runtime::main]
pub async fn rss_reader_service(
    #[shuttle_shared_db::Postgres(
        local_uri = "postgres://postgres:password@localhost:5432/postgres"
    )]
    pool: PgPool,
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> shuttle_axum::ShuttleAxum {
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Migration failed...");

    let state = AppState {
        pool: pool.clone(),
        secrets: secrets.clone(),
    };

    let scheduler_pool = pool.clone();
    let scheduler_secrets = secrets.clone();
    tokio::spawn(async move {
        let _ = schedule_cache_clear(scheduler_pool, &scheduler_secrets)
            .await
            .inspect_err(|e| {
                eprintln!("Failed to schedule cache clear: {}", e);
            })
            .context("Failed to schedule cache clear");
    });

    let unprotected_routes = Router::new()
        .route("/feeds", get(get_feeds))
        .route("/categories", get(get_categories));

    let protected_routes = Router::new()
        .route("/admin", get(get_raw_feeds).post(create_raw_feed))
        .route("/admin/batch", post(batch_create_raw_feeds))
        .route("/admin/:id", post(update_raw_feed).delete(delete_raw_feed));
    //     .layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    let routes = Router::new()
        .merge(unprotected_routes)
        .merge(protected_routes)
        .with_state(state);

    Ok(routes.into())
}

#[derive(Clone)]
struct AppState {
    pool: PgPool,
    secrets: SecretStore,
}
