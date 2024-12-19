use axum::Router;
use shuttle_runtime::SecretStore;
use sqlx::PgPool;

#[shuttle_runtime::main]
pub async fn rss_reader_service (
    #[shuttle_shared_db::Postgres] pool: PgPool,
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> shuttle_axum::ShuttleAxum {
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Migration failed...");

    let state = AppState { pool, secrets };

    let routes = Router::new()
        .with_state(state);

    Ok(routes.into())
}

#[derive(Clone)]
struct AppState {
    pool: PgPool,
    secrets: SecretStore
}
