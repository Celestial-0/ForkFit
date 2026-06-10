use std::sync::Arc;
use axum::{
    Json,
    extract::{Path, Query, State},
    response::Redirect,
};
use oauth2::{
    AuthorizationCode, AuthUrl, ClientId, ClientSecret, CsrfToken, RedirectUrl,
    Scope, TokenResponse, TokenUrl,
    basic::BasicClient,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use crate::{
    app::AppState,
    common::{AppError, AppResult},
};
use crate::auth::handlers::create_session_response;
use crate::user::User;

#[derive(Debug, Deserialize)]
pub struct CallbackParams {
    pub code: String,
    pub state: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OAuthUserInfo {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
}

pub async fn authorize(
    State(state): State<Arc<AppState>>,
    Path(provider): Path<String>,
) -> AppResult<Redirect> {
    let (client_id_env, client_secret_env, auth_url_str, token_url_str) = match provider.to_lowercase().as_str() {
        "google" => (
            state.config.google_client_id.clone(),
            state.config.google_client_secret.clone(),
            "https://accounts.google.com/o/oauth2/v2/auth",
            "https://oauth2.googleapis.com/token",
        ),
        "github" => (
            state.config.github_client_id.clone(),
            state.config.github_client_secret.clone(),
            "https://github.com/login/oauth/authorize",
            "https://github.com/login/oauth/access_token",
        ),
        _ => return Err(AppError::BadRequest(format!("unsupported oauth provider: {provider}"))),
    };

    let client_id = ClientId::new(client_id_env);
    let client_secret = ClientSecret::new(client_secret_env);
    let auth_url = AuthUrl::new(auth_url_str.to_string())
        .map_err(|e| AppError::BadRequest(format!("invalid auth url: {e}")))?;
    let token_url = TokenUrl::new(token_url_str.to_string())
        .map_err(|e| AppError::BadRequest(format!("invalid token url: {e}")))?;

    let redirect_url = format!(
        "http://{}/api/v1/auth/oauth/{}/callback",
        state.config.bind_addr,
        provider.to_lowercase()
    );
    let redirect = RedirectUrl::new(redirect_url)
        .map_err(|e| AppError::BadRequest(format!("invalid redirect url: {e}")))?;

    let client = BasicClient::new(client_id)
        .set_client_secret(client_secret)
        .set_auth_uri(auth_url)
        .set_token_uri(token_url)
        .set_redirect_uri(redirect);
    
    let mut auth_req = client.authorize_url(CsrfToken::new_random);
    
    auth_req = match provider.to_lowercase().as_str() {
        "google" => auth_req
            .add_scope(Scope::new("https://www.googleapis.com/auth/userinfo.email".to_string()))
            .add_scope(Scope::new("https://www.googleapis.com/auth/userinfo.profile".to_string())),
        "github" => auth_req
            .add_scope(Scope::new("user:email".to_string())),
        _ => auth_req,
    };

    let (auth_url, _csrf_token) = auth_req.url();
    Ok(Redirect::to(auth_url.as_str()))
}

pub async fn callback(
    State(state): State<Arc<AppState>>,
    Path(provider): Path<String>,
    Query(params): Query<CallbackParams>,
) -> AppResult<Json<crate::auth::types::AuthResponse>> {
    let (client_id_env, client_secret_env, auth_url_str, token_url_str) = match provider.to_lowercase().as_str() {
        "google" => (
            state.config.google_client_id.clone(),
            state.config.google_client_secret.clone(),
            "https://accounts.google.com/o/oauth2/v2/auth",
            "https://oauth2.googleapis.com/token",
        ),
        "github" => (
            state.config.github_client_id.clone(),
            state.config.github_client_secret.clone(),
            "https://github.com/login/oauth/authorize",
            "https://github.com/login/oauth/access_token",
        ),
        _ => return Err(AppError::BadRequest(format!("unsupported oauth provider: {provider}"))),
    };

    let client_id = ClientId::new(client_id_env);
    let client_secret = ClientSecret::new(client_secret_env);
    let auth_url = AuthUrl::new(auth_url_str.to_string())
        .map_err(|e| AppError::BadRequest(format!("invalid auth url: {e}")))?;
    let token_url = TokenUrl::new(token_url_str.to_string())
        .map_err(|e| AppError::BadRequest(format!("invalid token url: {e}")))?;

    let redirect_url = format!(
        "http://{}/api/v1/auth/oauth/{}/callback",
        state.config.bind_addr,
        provider.to_lowercase()
    );
    let redirect = RedirectUrl::new(redirect_url)
        .map_err(|e| AppError::BadRequest(format!("invalid redirect url: {e}")))?;

    let client = BasicClient::new(client_id)
        .set_client_secret(client_secret)
        .set_auth_uri(auth_url)
        .set_token_uri(token_url)
        .set_redirect_uri(redirect);

    let token_client = oauth2::reqwest::Client::new();
    
    let token_result = client
        .exchange_code(AuthorizationCode::new(params.code))
        .request_async(&token_client)
        .await
        .map_err(|e| AppError::BadRequest(format!("token exchange failed: {e}")))?;

    let access_token = token_result.access_token().secret();

    let api_client = reqwest::Client::new();
    let user_info = match provider.to_lowercase().as_str() {
        "google" => {
            let res = api_client
                .get("https://www.googleapis.com/oauth2/v2/userinfo")
                .bearer_auth(access_token)
                .send()
                .await
                .map_err(|e| AppError::BadRequest(format!("failed fetching google user info: {e}")))?;
            
            res.json::<OAuthUserInfo>()
                .await
                .map_err(|e| AppError::BadRequest(format!("failed parsing google user info: {e}")))?
        }
        "github" => {
            let res = api_client
                .get("https://api.github.com/user")
                .header("User-Agent", "forkfit-api")
                .bearer_auth(access_token)
                .send()
                .await
                .map_err(|e| AppError::BadRequest(format!("failed fetching github user info: {e}")))?;
            
            #[derive(Deserialize)]
            struct GithubUser {
                id: u64,
                email: Option<String>,
                name: Option<String>,
            }
            
            let github_user = res.json::<GithubUser>()
                .await
                .map_err(|e| AppError::BadRequest(format!("failed parsing github user info: {e}")))?;
            
            let email = match github_user.email {
                Some(email) => email,
                None => {
                    let emails_res = api_client
                        .get("https://api.github.com/user/emails")
                        .header("User-Agent", "forkfit-api")
                        .bearer_auth(access_token)
                        .send()
                        .await
                        .map_err(|e| AppError::BadRequest(format!("failed fetching github emails: {e}")))?;
                    
                    #[derive(Deserialize)]
                    struct GithubEmail {
                        email: String,
                        primary: bool,
                    }
                    
                    let emails: Vec<GithubEmail> = emails_res.json()
                        .await
                        .map_err(|e| AppError::BadRequest(format!("failed parsing github emails: {e}")))?;
                    
                    emails.into_iter()
                        .find(|e| e.primary)
                        .map(|e| e.email)
                        .ok_or_else(|| AppError::BadRequest("no primary email found for github account".into()))?
                }
            };

            OAuthUserInfo {
                id: github_user.id.to_string(),
                email,
                name: github_user.name,
            }
        }
        _ => return Err(AppError::BadRequest(format!("unsupported oauth provider: {provider}"))),
    };

    let normalized_email = crate::common::normalize_email(&user_info.email)?;

    let existing_oauth = sqlx::query(
        "SELECT user_id FROM oauth_accounts WHERE provider = $1 AND provider_user_id = $2"
    )
    .bind(provider.to_lowercase())
    .bind(&user_info.id)
    .fetch_optional(&state.db)
    .await?;

    let user = match existing_oauth {
        Some(row) => {
            let user_id: uuid::Uuid = row.try_get("user_id")?;
            sqlx::query_as::<_, User>(
                "SELECT id, email::text AS email, email_verified, status, created_at, updated_at, deleted_at FROM users WHERE id = $1 AND deleted_at IS NULL AND status = 'active'"
            )
            .bind(user_id)
            .fetch_one(&state.db)
            .await?
        }
        None => {
            let existing_user = sqlx::query_as::<_, User>(
                "SELECT id, email::text AS email, email_verified, status, created_at, updated_at, deleted_at FROM users WHERE email = $1 AND deleted_at IS NULL AND status = 'active'"
            )
            .bind(&normalized_email)
            .fetch_optional(&state.db)
            .await?;

            let user_id = match existing_user {
                Some(u) => u.id,
                None => {
                    let mut tx = state.db.begin().await?;
                    let u = sqlx::query_as::<_, User>(
                        r#"
                        INSERT INTO users (email, email_verified, status)
                        VALUES ($1, true, 'active')
                        RETURNING id, email::text AS email, email_verified, status, created_at, updated_at, deleted_at
                        "#
                    )
                    .bind(&normalized_email)
                    .fetch_one(&mut *tx)
                    .await?;

                    sqlx::query("INSERT INTO profiles (user_id, full_name, timezone) VALUES ($1, $2, 'UTC')")
                        .bind(u.id)
                        .bind(&user_info.name)
                        .execute(&mut *tx)
                        .await?;

                    sqlx::query(
                        "INSERT INTO user_preferences (user_id, theme, language, measurement_system, preferences) VALUES ($1, 'system', 'en', 'metric', '{}'::jsonb)"
                    )
                    .bind(u.id)
                    .execute(&mut *tx)
                    .await?;

                    tx.commit().await?;
                    u.id
                }
            };

            sqlx::query("INSERT INTO oauth_accounts (user_id, provider, provider_user_id, provider_email) VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING")
                .bind(user_id)
                .bind(provider.to_lowercase())
                .bind(&user_info.id)
                .bind(&normalized_email)
                .execute(&state.db)
                .await?;

            sqlx::query_as::<_, User>(
                "SELECT id, email::text AS email, email_verified, status, created_at, updated_at, deleted_at FROM users WHERE id = $1"
            )
            .bind(user_id)
            .fetch_one(&state.db)
            .await?
        }
    };

    let auth_resp = create_session_response(&state, user, Some("OAuth".to_string()), None, Some("OAuth Agent".to_string())).await?;
    Ok(Json(auth_resp))
}
