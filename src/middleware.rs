use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{Json, Response},
};
use serde_json::json;
use tracing::{info, warn};

use crate::AppState;

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    // Extract token from Authorization header
    let headers = request.headers().clone();
    let token = match state.auth_service.extract_token_from_headers(&headers) {
        Ok(token) => token,
        Err(e) => {
            warn!("Failed to extract token: {}", e);
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "error": "Unauthorized",
                    "message": "Missing or invalid authorization header"
                })),
            ));
        }
    };

    // Validate token with Keycloak
    let validation_result = state.auth_service.validate_token(&token).await;
    
    match validation_result {
        Ok(validation_response) => {
            if !validation_response.valid {
                warn!("Token validation failed: {:?}", validation_response.error);
                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(json!({
                        "error": "Unauthorized",
                        "message": "Invalid token"
                    })),
                ));
            }

            if let Some(user) = validation_response.user {
                info!("User authenticated: {} (sub: {})", user.preferred_username, user.sub);
                info!("User realm_access: {:?}", user.realm_access);
                info!("User resource_access: {:?}", user.resource_access);
                
                // Add user info to request extensions for use in handlers
                request.extensions_mut().insert(user);
                
                Ok(next.run(request).await)
            } else {
                warn!("Token validation succeeded but no user data returned");
                Err((
                    StatusCode::UNAUTHORIZED,
                    Json(json!({
                        "error": "Unauthorized",
                        "message": "No user data in token"
                    })),
                ))
            }
        }
        Err(e) => {
            warn!("Token validation error: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Internal Server Error",
                    "message": "Token validation failed"
                })),
            ))
        }
    }
}

pub async fn require_admin_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let user = request.extensions().get::<crate::auth::KeycloakUser>().ok_or_else(|| {
        warn!("User not found in request extensions (require_admin)");
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": "Unauthorized",
                "message": "Authentication required"
            })),
        )
    })?;

    // Debug: log user info and roles
    let all_roles = state.auth_service.get_user_roles(user);
    info!("Admin check for user '{}': roles={:?}", user.preferred_username, all_roles);
    info!("Realm access: {:?}", user.realm_access);
    info!("Resource access: {:?}", user.resource_access);
    
    let is_admin = state.auth_service.is_admin(user);
    info!("Is admin check result: {}", is_admin);

    if !is_admin {
        warn!("Access denied: user '{}' is not admin. Available roles: {:?}", user.preferred_username, all_roles);
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "Forbidden",
                "message": "Admin role required"
            })),
        ));
    }

    info!("Admin access granted for user: {}", user.preferred_username);
    Ok(next.run(request).await)
}
