use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json::json;
use tracing::{info, warn, error};

use crate::{models::{CreateUserRequest, UpdateUserRequest}, AppState};

pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let username = payload.username.clone();
    info!("Admin: create user '{}': roles={:?}", username, payload.roles);
    match state.auth_service.create_keycloak_user(payload).await {
        Ok(user_id) => {
            info!("Successfully created user with id: {}", user_id);
            Ok(Json(json!({ "id": user_id })))
        },
        Err(e) => {
            error!("Create user '{}' failed: {}", username, e);
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

pub async fn revoke_specific_user_session(
    State(state): State<AppState>,
    Path((user_id, session_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Admin: revoke specific session '{}' for user '{}'", session_id, user_id);
    match state.auth_service.revoke_specific_session(&session_id).await {
        Ok(()) => Ok(Json(json!({ "message": format!("Session {} revoked", session_id) }))),
        Err(e) => {
            warn!("Revoke specific session failed: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

pub async fn get_all_users(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Admin: get all users");
    match state.auth_service.get_all_users().await {
        Ok(users) => {
            info!("Successfully retrieved {} users", users.len());
            Ok(Json(json!({ "users": users })))
        },
        Err(e) => {
            error!("Get all users failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_user_by_id(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Admin: get user by id '{}'", user_id);
    match state.auth_service.get_user_by_id(&user_id).await {
        Ok(user) => Ok(Json(user)),
        Err(e) => {
            warn!("Get user by id failed: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

pub async fn get_user_roles(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Admin: get roles for user '{}'", user_id);
    match state.auth_service.get_user_roles_by_id(&user_id).await {
        Ok(roles) => {
            info!("Successfully retrieved {} roles for user {}", roles.len(), user_id);
            Ok(Json(json!({ "roles": roles })))
        },
        Err(e) => {
            error!("Get user roles failed for user {}: {}", user_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
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


