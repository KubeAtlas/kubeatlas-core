use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Deserialize)]
pub struct CreateInstallTokenRequest {
    pub service_name: String,
    pub service_type: ServiceType,
    pub controller_name: Option<String>, // для агентов - указывает к какому контроллеру они принадлежат
    pub expires_in_hours: Option<i64>, // по умолчанию 24 часа
}

#[derive(Debug, Serialize)]
pub struct CreateInstallTokenResponse {
    pub install_token: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallTokenData {
    pub service_name: String,
    pub service_type: ServiceType,
    pub controller_name: Option<String>,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServiceType {
    Controller,
    Agent,
}

impl ToString for ServiceType {
    fn to_string(&self) -> String {
        match self {
            ServiceType::Controller => "controller".to_string(),
            ServiceType::Agent => "agent".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ConnectedService {
    pub id: Uuid,
    pub service_type: String,
    pub service_name: String,
    pub controller_name: Option<String>,
    pub client_cert_serial: String,
    pub client_cert_fingerprint: String,
    pub connected_at: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub metadata: serde_json::Value,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct ServiceRegistrationRequest {
    pub install_token: String,
    pub client_cert_pem: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct ServiceRegistrationResponse {
    pub service_id: Uuid,
    pub message: String,
}