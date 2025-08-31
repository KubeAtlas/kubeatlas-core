use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use tracing::{info, warn};

use crate::{auth::{TokenValidationRequest, TokenValidationResponse, UserInfoResponse, KeycloakUser}, AppState};

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
    State(_state): State<AppState>,
    Json(_payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: Implement token refresh logic
    // For now, return a placeholder response
    let response = json!({
        "message": "Token refresh not implemented yet",
        "status": "placeholder"
    });
    
    Ok(Json(response))
}
