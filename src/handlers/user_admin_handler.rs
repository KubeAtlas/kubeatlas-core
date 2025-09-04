use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json::json;
use tracing::{info, warn};

use crate::{models::{CreateUserRequest, UpdateUserRequest}, AppState};

pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Admin: create user '{}': roles={:?}", payload.username, payload.roles);
    match state.auth_service.create_keycloak_user(payload).await {
        Ok(user_id) => Ok(Json(json!({ "id": user_id }))),
        Err(e) => {
            warn!("Create user failed: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

pub async fn update_user(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Admin: update user '{}': payload", user_id);
    match state
        .auth_service
        .update_keycloak_user(&user_id, payload)
        .await
    {
        Ok(_) => Ok(Json(json!({ "id": user_id }))),
        Err(e) => {
            warn!("Update user failed: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

pub async fn get_user_sessions(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Admin: get sessions for user '{}'", user_id);
    match state.auth_service.get_active_sessions(&user_id).await {
        Ok(sessions) => Ok(Json(json!({ "sessions": sessions }))),
        Err(e) => {
            warn!("Get user sessions failed: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

pub async fn revoke_user_sessions(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Admin: revoke sessions for user '{}'", user_id);
    match state.auth_service.revoke_user_sessions(&user_id).await {
        Ok(()) => Ok(Json(json!({ "message": "All sessions revoked" }))),
        Err(e) => {
            warn!("Revoke user sessions failed: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

pub async fn delete_user(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Admin: delete user '{}'", user_id);
    match state.auth_service.delete_keycloak_user(&user_id).await {
        Ok(()) => Ok(Json(json!({ 
            "message": "User deleted successfully",
            "id": user_id 
        }))),
        Err(e) => {
            warn!("Delete user failed: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}


