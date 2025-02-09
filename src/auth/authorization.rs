use anyhow::Context;
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::IntoResponse,
};
use chrono::{Duration, NaiveDateTime, Utc};
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use shuttle_runtime::SecretStore;

use crate::{error::ServiceError, AppState};

#[derive(Debug, Deserialize)]
struct AuthenticatedUser {
    id: i32,
}

#[derive(Debug, Deserialize)]
struct GitHubTokenCheck {
    created_at: String,
    user: AuthenticatedUser,
}

async fn invalidate_expired_token(
    secrets: &SecretStore,
    access_token: &str,
) -> Result<(), ServiceError> {
    let github_client_id = SecretStore::get(secrets, "GITHUB_CLIENT_ID")
        .context("Missing expected ENV_VAR: GITHUB_CLIENT_ID")?;
    let github_client_secret = SecretStore::get(secrets, "GITHUB_CLIENT_SECRET")
        .context("Missing expected ENV_VAR: GITHUB_CLIENT_SECRET")?;

    let http_client = Client::new();
    let response = http_client
        .delete(format!(
            "https://api.github.com/applications/{github_client_id}/token"
        ))
        .header("Accept", "application/vnd.github+json")
        .header("content-type", "application/json")
        .header("User-Agent", "rss-reader-service")
        .basic_auth(github_client_id, Some(github_client_secret))
        .body(format!("{{\"access_token\":\"{access_token}\"}}"))
        .send()
        .await
        .inspect_err(|e| {
            eprintln!("Authentication error: {:?}", e);
        })
        .context("Failed to invalidate user token")?;

    let status_code = &response.status();
    if status_code != &StatusCode::OK {
        let response_text: String = response
            .text()
            .await
            .inspect_err(|e| {
                eprintln!("Authentication error: {:?}", e);
            })
            .context("Error reading GitHub API response")?;
        return Err(ServiceError::from(anyhow::Error::msg(format!(
            "Unable to invalidate expired token! {}",
            response_text,
        ))));
    }

    Ok(())
}

async fn fetch_github_user_id(
    secrets: &SecretStore,
    access_token: &str,
) -> Result<String, ServiceError> {
    let github_client_id = SecretStore::get(secrets, "GITHUB_CLIENT_ID")
        .context("Missing expected ENV_VAR: GITHUB_CLIENT_ID")?;
    let github_client_secret = SecretStore::get(secrets, "GITHUB_CLIENT_SECRET")
        .context("Missing expected ENV_VAR: GITHUB_CLIENT_SECRET")?;

    let http_client = Client::new();
    let response = http_client
        .post(format!(
            "https://api.github.com/applications/{github_client_id}/token"
        ))
        .header("Accept", "application/vnd.github+json")
        .header("content-type", "application/json")
        .header("User-Agent", "rss-reader-service")
        .basic_auth(github_client_id, Some(github_client_secret))
        .body(format!("{{\"access_token\":\"{access_token}\"}}"))
        .send()
        .await
        .inspect_err(|e| {
            eprintln!("Authentication error: {:?}", e);
        })
        .context("Failed to generate user token")?;

    let status_code = &response.status();
    if status_code != &StatusCode::OK {
        return Err(ServiceError::from(anyhow::Error::msg(
            "Unable to verify user credential!",
        )));
    }

    let response_text: String = response
        .text()
        .await
        .inspect_err(|e| {
            eprintln!("Error reading auth response body: {:?}", e);
        })
        .context("Failed to generate user token")?;

    let token_info: GitHubTokenCheck = serde_json::from_str(&response_text)
        .inspect_err(|e| {
            eprintln!("Error decoding auth JSON: {:?}", e);
        })
        .context("Failed to generate user token")?;

    let now = Utc::now();
    let token_created = NaiveDateTime::parse_from_str(&token_info.created_at, "%Y-%m-%dT%H:%M:%SZ")
        .inspect_err(|e| {
            eprintln!("Error parsing token created_at: {:?}", e);
        })
        .context("Failed to generate user token")?;

    if token_created + Duration::hours(1) < now.naive_utc() {
        invalidate_expired_token(secrets, access_token).await?;
        return Err(ServiceError::from(anyhow::Error::msg(
            "Expired access token! Token invalidated...",
        )));
    }

    Ok(token_info.user.id.to_string())
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, ServiceError> {
    let (parts, body) = req.into_parts();
    let headers = parts.headers.clone();

    let bearer_token = headers
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .context("Missing or invalid Authorization header")?;

    let access_token = bearer_token.replace("Bearer ", "");
    let user_id = fetch_github_user_id(&state.secrets, &access_token).await?;

    let admin_user_id = SecretStore::get(&state.secrets, "GITHUB_USER_ID")
        .context("Missing expected ENV_VAR: GITHUB_USER_ID")?;

    if !user_id.eq(&admin_user_id) {
        return Err(ServiceError::from(anyhow::Error::msg(
            "Unauthorized user action",
        )));
    }

    let req = Request::from_parts(parts, body);
    let response = next.run(req).await;

    Ok(response)
}
