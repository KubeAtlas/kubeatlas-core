use axum::{
    extract::{Extension, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;

use crate::{
    models::{
        CreateInstallTokenRequest, CreateInstallTokenResponse,
        ServiceRegistrationRequest, ServiceRegistrationResponse,
        ConnectedService, ServiceType, ErrorResponse
    },
    AppState,
};

#[derive(Debug, Deserialize)]
pub struct ListServicesQuery {
    service_type: Option<String>,
}

pub async fn create_install_token(
    State(state): State<AppState>,
    Extension(user): Extension<crate::auth::KeycloakUser>,
    Json(request): Json<CreateInstallTokenRequest>,
) -> Result<Json<CreateInstallTokenResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.install_service.create_install_token(request, &user.preferred_username).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            tracing::error!("Failed to create install token: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to create install token".to_string(),
                    message: Some(e.to_string()),
                }),
            ))
        }
    }
}

pub async fn register_service(
    State(state): State<AppState>,
    Json(request): Json<ServiceRegistrationRequest>,
) -> Result<Json<ServiceRegistrationResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.install_service.register_service(request).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            tracing::error!("Failed to register service: {}", e);
            Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "Failed to register service".to_string(),
                    message: Some(e.to_string()),
                }),
            ))
        }
    }
}

pub async fn list_connected_services(
    State(state): State<AppState>,
    Query(query): Query<ListServicesQuery>,
) -> Result<Json<Vec<ConnectedService>>, (StatusCode, Json<ErrorResponse>)> {
    let service_type = match query.service_type {
        Some(ref type_str) => match type_str.as_str() {
            "controller" => Some(ServiceType::Controller),
            "agent" => Some(ServiceType::Agent),
            _ => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: "Invalid service type".to_string(),
                        message: Some("Valid types are: controller, agent".to_string()),
                    }),
                ));
            }
        },
        None => None,
    };

    match state.install_service.get_connected_services(service_type).await {
        Ok(services) => Ok(Json(services)),
        Err(e) => {
            tracing::error!("Failed to list connected services: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to list connected services".to_string(),
                    message: Some(e.to_string()),
                }),
            ))
        }
    }
}