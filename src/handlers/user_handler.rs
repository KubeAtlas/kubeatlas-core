use axum::{
    extract::Request,
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use tracing::info;

use crate::auth::KeycloakUser;

pub async fn get_profile(request: Request) -> Result<Json<Value>, StatusCode> {
    // Extract user from request extensions (set by auth middleware)
    let user = request
        .extensions()
        .get::<KeycloakUser>()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    info!("Getting profile for user: {}", user.preferred_username);

    let profile = json!({
        "id": user.sub,
        "username": user.preferred_username,
        "email": user.email,
        "firstName": user.given_name,
        "lastName": user.family_name,
        "realmAccess": user.realm_access,
        "resourceAccess": user.resource_access
    });

    Ok(Json(profile))
}

pub async fn get_user_roles(request: Request) -> Result<Json<Value>, StatusCode> {
    // Extract user from request extensions (set by auth middleware)
    let user = request
        .extensions()
        .get::<KeycloakUser>()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    info!("Getting roles for user: {}", user.preferred_username);

    let mut roles = Vec::new();
    
    // Add realm roles
    if let Some(realm_access) = &user.realm_access {
        roles.extend(realm_access.roles.clone());
    }
    
    // Add client roles
    if let Some(resource_access) = &user.resource_access {
        for (_, access) in resource_access {
            roles.extend(access.roles.clone());
        }
    }

    let response = json!({
        "username": user.preferred_username,
        "roles": roles,
        "isAdmin": roles.contains(&"admin".to_string()),
        "isUser": roles.contains(&"user".to_string()),
        "isGuest": roles.contains(&"guest".to_string())
    });

    Ok(Json(response))
}
