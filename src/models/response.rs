use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatisticsResponse {
    pub total_users: StatItem,
    pub active_sessions: StatItem,
    pub system_status: SystemStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatItem {
    pub value: u64,
    pub change_percent: f64,
    pub change_period: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemStatus {
    pub percentage: f64,
    pub status: String,
    pub details: Vec<ServiceStatus>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub name: String,
    pub status: String,
    pub uptime_percentage: f64,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            message: None,
        }
    }

    #[allow(dead_code)]
    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
            message: None,
        }
    }

    #[allow(dead_code)]
    pub fn message(message: String) -> Self {
        Self {
            success: true,
            data: None,
            error: None,
            message: Some(message),
        }
    }
}
