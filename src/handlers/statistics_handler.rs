use axum::{
    extract::{Request, State},
    http::StatusCode,
    response::Json,
};
use tracing::{info, warn, error};

use crate::{
    models::{ApiResponse, StatisticsResponse, StatItem, SystemStatus, ServiceStatus},
    AppState,
};

/// GET /api/v1/statistics
/// Retrieves dashboard statistics including total users, active sessions, and system status
/// 
/// Returns statistics similar to the frontend cards:
/// - Total users with growth percentage
/// - Active sessions with recent change
/// - System status with service health details
pub async fn get_statistics(
    State(state): State<AppState>,
    request: Request,
) -> Result<Json<ApiResponse<StatisticsResponse>>, StatusCode> {
    // Extract user from request extensions (set by auth middleware)
    let user = request
        .extensions()
        .get::<crate::auth::KeycloakUser>()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    info!("Getting statistics for user: {}", user.preferred_username);
    match get_statistics_data(&state).await {
        Ok(stats) => {
            info!("Successfully retrieved statistics for user: {}", user.preferred_username);
            Ok(Json(ApiResponse::success(stats)))
        },
        Err(e) => {
            error!("Failed to get statistics for user {}: {}", user.preferred_username, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_statistics_data(state: &AppState) -> Result<StatisticsResponse, anyhow::Error> {
    // Get total users count from Keycloak
    let total_users = state.auth_service.get_total_users_count().await?;
    
    // Get active sessions count
    let active_sessions = state.auth_service.get_active_sessions_count().await?;
    
    // Calculate system status based on various health checks
    let system_status = calculate_system_status(state).await?;

    let stats = StatisticsResponse {
        total_users: StatItem {
            value: total_users,
            change_percent: 12.0, // Placeholder - could be calculated from historical data
            change_period: "с прошлого месяца".to_string(),
        },
        active_sessions: StatItem {
            value: active_sessions,
            change_percent: 5.0, // Placeholder - could be calculated from recent data
            change_period: "с прошлого часа".to_string(),
        },
        system_status,
    };

    Ok(stats)
}

async fn calculate_system_status(state: &AppState) -> Result<SystemStatus, anyhow::Error> {
    let mut services = Vec::new();
    
    // Check Keycloak connectivity
    let keycloak_status = match state.auth_service.health_check().await {
        Ok(_) => ServiceStatus {
            name: "Keycloak".to_string(),
            status: "operational".to_string(),
            uptime_percentage: 99.9,
        },
        Err(_) => ServiceStatus {
            name: "Keycloak".to_string(),
            status: "degraded".to_string(),
            uptime_percentage: 95.0,
        }
    };
    services.push(keycloak_status);

    // Check database connectivity (if applicable)
    // This is a placeholder - you would implement actual database health check
    let database_status = ServiceStatus {
        name: "Database".to_string(),
        status: "operational".to_string(),
        uptime_percentage: 99.5,
    };
    services.push(database_status);

    // Check API server status (always operational if we're responding)
    let api_status = ServiceStatus {
        name: "API Server".to_string(),
        status: "operational".to_string(),
        uptime_percentage: 99.8,
    };
    services.push(api_status);

    // Calculate overall system health percentage
    let average_uptime = services.iter()
        .map(|s| s.uptime_percentage)
        .sum::<f64>() / services.len() as f64;

    let overall_status = if average_uptime > 99.0 {
        "Все системы работают".to_string()
    } else if average_uptime > 95.0 {
        "Незначительные проблемы".to_string()
    } else {
        "Системные проблемы".to_string()
    };

    Ok(SystemStatus {
        percentage: average_uptime,
        status: overall_status,
        details: services,
    })
}