use anyhow::Context;
use axum::{routing::get, Router};
use shuttle_runtime::SecretStore;
use sqlx::PgPool;

mod service;
use service::{ create_raw_feed, get_feeds, get_raw_feeds, schedule_cache_clear };

mod data;
mod error;

#[shuttle_runtime::main]
pub async fn rss_reader_service (
    #[shuttle_shared_db::Postgres] pool: PgPool,
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> shuttle_axum::ShuttleAxum {
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Migration failed...");

    let state = AppState { pool: pool.clone(), secrets: secrets.clone() };

    let scheduler_pool = pool.clone();
    let scheduler_secrets = secrets.clone();
    tokio::spawn(async move {
        let _ = schedule_cache_clear(scheduler_pool, &scheduler_secrets)
            .await
            .inspect_err(|e| { eprintln!("Failed to schedule cache clear: {}", e); })
            .context("Failed to schedule cache clear");
    });

    let unprotected_routes = Router::new()
        .route("/feeds",
            get(get_feeds)
        );

    let protected_routes = Router::new()
        .route("/admin",
            get(get_raw_feeds)
            .post(create_raw_feed)
        );
    //     .route("/admin/batch",
    //         post(batch_create_feeds)
    //     )
    //     .route("/admin/:id",
    //         delete(delete_feed)
    //     )
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
    secrets: SecretStore
}
