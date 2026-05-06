//! Authentication API handlers

use axum::{extract::State, http::HeaderMap, Json};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::auth::{
    extract_bearer_token, generate_access_token, generate_refresh_token, hash_password, hash_token,
    validate_token, verify_password,
};
use crate::error::{Result, ServerError};
use crate::AppState;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub user_id: String,
    pub email: String,
}

#[derive(Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

/// POST /api/v1/auth/register
pub async fn register(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>> {
    // Validate email
    if !req.email.contains('@') || req.email.len() < 3 {
        return Err(ServerError::BadRequest("Invalid email address".into()));
    }

    // Validate password length
    if req.password.len() < 8 {
        return Err(ServerError::BadRequest(
            "Password must be at least 8 characters".into(),
        ));
    }

    // Check if email already exists
    if state.db.find_user_by_email(&req.email).await?.is_some() {
        return Err(ServerError::BadRequest("Email already registered".into()));
    }

    // Hash password with Argon2id
    let password_hash = hash_password(&req.password)?;

    // Create user
    let user = state.db.create_user(&req.email, &password_hash).await?;

    // Generate tokens
    let access_token = generate_access_token(&user.id, &user.email, &state.config)?;
    let refresh_token = generate_refresh_token(&user.id, &user.email, &state.config)?;

    // Store refresh token hash
    let token_hash = hash_token(&refresh_token);
    let expires_at = Utc::now()
        + Duration::seconds(crate::config::ServerConfig::REFRESH_TOKEN_EXPIRY_SECS as i64);
    let ip = extract_ip(&headers);
    state
        .db
        .create_session(&user.id, &token_hash, expires_at, ip.as_deref())
        .await?;

    // Audit log
    state
        .db
        .log_event(Some(&user.id), "user_registered", ip.as_deref(), None)
        .await?;

    tracing::info!("New user registered: {}", user.email);

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: crate::config::ServerConfig::TOKEN_EXPIRY_SECS,
        user_id: user.id,
        email: user.email,
    }))
}

/// POST /api/v1/auth/login
pub async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>> {
    let ip = extract_ip(&headers);

    // Find user
    let user = state
        .db
        .find_user_by_email(&req.email)
        .await?
        .ok_or(ServerError::Unauthorized)?;

    // Verify password (constant-time comparison via Argon2)
    if !verify_password(&req.password, &user.password_hash)? {
        state
            .db
            .log_event(Some(&user.id), "login_failed", ip.as_deref(), None)
            .await?;
        return Err(ServerError::Unauthorized);
    }

    // Generate tokens
    let access_token = generate_access_token(&user.id, &user.email, &state.config)?;
    let refresh_token = generate_refresh_token(&user.id, &user.email, &state.config)?;

    // Store refresh token hash
    let token_hash = hash_token(&refresh_token);
    let expires_at = Utc::now()
        + Duration::seconds(crate::config::ServerConfig::REFRESH_TOKEN_EXPIRY_SECS as i64);
    state
        .db
        .create_session(&user.id, &token_hash, expires_at, ip.as_deref())
        .await?;

    // Update last login
    state.db.update_last_login(&user.id).await?;

    state
        .db
        .log_event(Some(&user.id), "login_success", ip.as_deref(), None)
        .await?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: crate::config::ServerConfig::TOKEN_EXPIRY_SECS,
        user_id: user.id,
        email: user.email,
    }))
}

/// POST /api/v1/auth/refresh
pub async fn refresh_token(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<AuthResponse>> {
    // Validate refresh token
    let claims = validate_token(&req.refresh_token, &state.config)?;
    if claims.token_type != "refresh" {
        return Err(ServerError::Unauthorized);
    }

    // Check session exists
    let token_hash = hash_token(&req.refresh_token);
    let user_id = state
        .db
        .find_session(&token_hash)
        .await?
        .ok_or(ServerError::Unauthorized)?;

    // Get user
    let user = state
        .db
        .find_user_by_id(&user_id)
        .await?
        .ok_or(ServerError::Unauthorized)?;

    // Rotate tokens (delete old, create new)
    state.db.delete_session(&token_hash).await?;

    let new_access = generate_access_token(&user.id, &user.email, &state.config)?;
    let new_refresh = generate_refresh_token(&user.id, &user.email, &state.config)?;

    let new_hash = hash_token(&new_refresh);
    let expires_at = Utc::now()
        + Duration::seconds(crate::config::ServerConfig::REFRESH_TOKEN_EXPIRY_SECS as i64);
    state
        .db
        .create_session(&user.id, &new_hash, expires_at, None)
        .await?;

    Ok(Json(AuthResponse {
        access_token: new_access,
        refresh_token: new_refresh,
        token_type: "Bearer".to_string(),
        expires_in: crate::config::ServerConfig::TOKEN_EXPIRY_SECS,
        user_id: user.id,
        email: user.email,
    }))
}

/// POST /api/v1/auth/logout
pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<serde_json::Value>> {
    let token_hash = hash_token(&req.refresh_token);
    state.db.delete_session(&token_hash).await?;

    let ip = extract_ip(&headers);
    state
        .db
        .log_event(None, "logout", ip.as_deref(), None)
        .await?;

    Ok(Json(
        serde_json::json!({ "message": "Logged out successfully" }),
    ))
}

/// Extract client IP from headers (X-Forwarded-For or X-Real-IP)
fn extract_ip(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-forwarded-for")
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or(s).trim().to_string())
}
