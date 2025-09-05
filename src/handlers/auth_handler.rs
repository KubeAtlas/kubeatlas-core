use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use tracing::{info, warn};

use crate::{auth::{TokenValidationRequest, TokenValidationResponse, UserInfoResponse, RefreshTokenRequest, RefreshTokenResponse, LogoutRequest}, AppState};

pub async fn validate_token(
    State(state): State<AppState>,
    Json(payload): Json<TokenValidationRequest>,
) -> Result<Json<TokenValidationResponse>, StatusCode> {
    info!("Validating token...");
    
    match state.auth_service.validate_token(&payload.token).await {
        Ok(response) => {
            if response.valid {
                info!("Token validation successful");
            } else {
                warn!("Token validation failed: {:?}", response.error);
            }
            Ok(Json(response))
        }
        Err(e) => {
            warn!("Token validation error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_user_info(
    State(state): State<AppState>,
    Json(payload): Json<TokenValidationRequest>,
) -> Result<Json<UserInfoResponse>, StatusCode> {
    info!("Getting user info...");
    
    match state.auth_service.validate_token(&payload.token).await {
        Ok(response) => {
            if response.valid {
                if let Some(user) = response.user {
                    let roles = state.auth_service.get_user_roles(&user);
                    let user_info = UserInfoResponse { user, roles };
                    info!("User info retrieved successfully");
                    Ok(Json(user_info))
                } else {
                    warn!("No user data in valid token");
                    Err(StatusCode::UNAUTHORIZED)
                }
            } else {
                warn!("Invalid token for user info request");
                Err(StatusCode::UNAUTHORIZED)
            }
        }
        Err(e) => {
            warn!("User info error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn refresh_token(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<Json<RefreshTokenResponse>, (StatusCode, Json<Value>)> {
    info!("Refreshing token...");
    
    match state.auth_service.refresh_token(&payload.refresh_token).await {
        Ok(response) => {
            info!("Token refresh successful");
            Ok(Json(response))
        }
        Err(e) => {
            warn!("Token refresh error: {}", e);
            let error_response = json!({
                "error": "invalid_token",
                "error_description": "Refresh token is invalid or expired"
            });
            Err((StatusCode::UNAUTHORIZED, Json(error_response)))
        }
    }
}

pub async fn logout(
    State(state): State<AppState>,
    Json(payload): Json<LogoutRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    info!("Logging out user...");
    
    match state.auth_service.logout(&payload.refresh_token).await {
        Ok(()) => {
            info!("Logout successful");
            let response = json!({
                "message": "Successfully logged out"
            });
            Ok(Json(response))
        }
        Err(e) => {
            warn!("Logout error: {}", e);
            let error_response = json!({
                "error": "logout_failed",
                "error_description": "Failed to logout"
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}
