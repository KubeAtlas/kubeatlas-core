use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use redis::{AsyncCommands};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::models::{
    ConnectedService, CreateInstallTokenRequest, CreateInstallTokenResponse,
    InstallTokenData, ServiceRegistrationRequest, ServiceRegistrationResponse, ServiceType
};
use crate::services::TokenService;

#[derive(Clone)]
pub struct InstallService {
    token_service: TokenService,
}

impl InstallService {
    pub fn new(token_service: TokenService) -> Self {
        Self { token_service }
    }

    pub async fn create_install_token(
        &self,
        request: CreateInstallTokenRequest,
        created_by: &str,
    ) -> Result<CreateInstallTokenResponse> {
        let expires_in_hours = request.expires_in_hours.unwrap_or(24);
        let created_at = Utc::now();
        let expires_at = created_at + Duration::hours(expires_in_hours);

        let token_data = InstallTokenData {
            service_name: request.service_name,
            service_type: request.service_type,
            controller_name: request.controller_name,
            created_by: created_by.to_string(),
            created_at,
            expires_at,
        };

        let install_token = self.token_service
            .generate_install_token(token_data, expires_in_hours)
            .await
            .context("Failed to generate install token")?;

        Ok(CreateInstallTokenResponse {
            install_token,
            expires_at,
        })
    }

    pub async fn register_service(
        &self,
        request: ServiceRegistrationRequest,
    ) -> Result<ServiceRegistrationResponse> {
        let token_data = self.token_service
            .consume_install_token(&request.install_token)
            .await
            .context("Invalid or expired install token")?;

        let cert_serial = self.extract_cert_serial(&request.client_cert_pem)
            .context("Failed to extract certificate serial")?;
        let cert_fingerprint = self.calculate_cert_fingerprint(&request.client_cert_pem);

        let service_id = Uuid::new_v4();
        let service = ConnectedService {
            id: service_id,
            service_type: token_data.service_type.to_string(),
            service_name: token_data.service_name.clone(),
            controller_name: token_data.controller_name.clone(),
            client_cert_serial: cert_serial,
            client_cert_fingerprint: cert_fingerprint,
            connected_at: Utc::now(),
            last_seen: Utc::now(),
            metadata: request.metadata.unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new())),
            status: "active".to_string(),
        };

        // Store service in Redis
        let service_key = format!("service:{}", service_id);
        let service_json = serde_json::to_string(&service)
            .context("Failed to serialize service data")?;

        let mut conn = self.token_service.redis.get_async_connection()
            .await
            .context("Failed to connect to Redis")?;

        conn.set(&service_key, service_json)
            .await
            .context("Failed to store service in Redis")?;

        // Add to service list
        conn.sadd("services", service_id.to_string())
            .await
            .context("Failed to add service to list")?;

        Ok(ServiceRegistrationResponse {
            service_id,
            message: format!(
                "{} '{}' successfully registered",
                token_data.service_type.to_string(),
                token_data.service_name
            ),
        })
    }

    pub async fn get_connected_services(&self, service_type: Option<ServiceType>) -> Result<Vec<ConnectedService>> {
        let mut conn = self.token_service.redis.get_async_connection()
            .await
            .context("Failed to connect to Redis")?;

        // Get all service IDs
        let service_ids: Vec<String> = conn.smembers("services")
            .await
            .context("Failed to get service list from Redis")?;

        let mut services = Vec::new();
        for service_id in service_ids {
            let service_key = format!("service:{}", service_id);
            let service_json: Option<String> = conn.get(&service_key)
                .await
                .context("Failed to get service from Redis")?;

            if let Some(json) = service_json {
                match serde_json::from_str::<ConnectedService>(&json) {
                    Ok(service) => {
                        // Filter by service type if specified
                        if let Some(ref stype) = service_type {
                            if service.service_type == stype.to_string() {
                                services.push(service);
                            }
                        } else {
                            services.push(service);
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to deserialize service {}: {}", service_id, e);
                    }
                }
            }
        }

        // Sort by connected_at in descending order
        services.sort_by(|a, b| b.connected_at.cmp(&a.connected_at));

        Ok(services)
    }

    pub async fn update_service_heartbeat(&self, cert_serial: &str) -> Result<()> {
        let mut conn = self.token_service.redis.get_async_connection()
            .await
            .context("Failed to connect to Redis")?;

        // Get all service IDs to find the one with matching cert serial
        let service_ids: Vec<String> = conn.smembers("services")
            .await
            .context("Failed to get service list from Redis")?;

        for service_id in service_ids {
            let service_key = format!("service:{}", service_id);
            let service_json: Option<String> = conn.get(&service_key)
                .await
                .context("Failed to get service from Redis")?;

            if let Some(json) = service_json {
                if let Ok(mut service) = serde_json::from_str::<ConnectedService>(&json) {
                    if service.client_cert_serial == cert_serial {
                        // Update last_seen
                        service.last_seen = Utc::now();
                        
                        let updated_json = serde_json::to_string(&service)
                            .context("Failed to serialize updated service")?;
                        
                        conn.set(&service_key, updated_json)
                            .await
                            .context("Failed to update service heartbeat in Redis")?;
                        
                        return Ok(());
                    }
                }
            }
        }

        Err(anyhow::anyhow!("Service with cert serial {} not found", cert_serial))
    }

    fn extract_cert_serial(&self, cert_pem: &str) -> Result<String> {
        use x509_parser::prelude::*;
        
        let (_, pem) = parse_x509_pem(cert_pem.as_bytes())
            .map_err(|e| anyhow::anyhow!("Failed to parse PEM: {}", e))?;
        
        let cert = pem.parse_x509()
            .map_err(|e| anyhow::anyhow!("Failed to parse certificate: {}", e))?;
        
        Ok(cert.serial.to_string())
    }

    fn calculate_cert_fingerprint(&self, cert_pem: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(cert_pem.trim().as_bytes());
        format!("{:x}", hasher.finalize())
    }
}