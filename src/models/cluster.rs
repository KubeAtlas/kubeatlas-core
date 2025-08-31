use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cluster {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub endpoint: String,
    pub status: ClusterStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClusterStatus {
    Active,
    Inactive,
    Error,
    Pending,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateClusterRequest {
    pub name: String,
    pub description: Option<String>,
    pub endpoint: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateClusterRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub endpoint: Option<String>,
    pub status: Option<ClusterStatus>,
}
