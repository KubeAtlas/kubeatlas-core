use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::env;
use rand::{distributions::Alphanumeric, Rng};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server_address: String,
    pub database_url: String,
    pub keycloak_url: String,
    pub keycloak_realm: String,
    pub keycloak_client_id: String,
    pub keycloak_client_secret: String,
    pub jwt_secret: String,
    pub log_level: String,
    pub adm_user: Option<String>,
    pub adm_password: Option<String>,
    pub keycloak_admin_user: Option<String>,
    pub keycloak_admin_password: Option<String>,
}

impl Config {
    pub fn load() -> Result<Self> {
        // Опциональная загрузка .env только если явно указано USE_DOTENV=true
        if env::var("USE_DOTENV").ok().as_deref() == Some("true") {
            dotenv::dotenv().ok();
        }

        let config = Config {
            server_address: env::var("SERVER_ADDRESS")
                .unwrap_or_else(|_| "0.0.0.0:3001".to_string()),
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://keycloak:keycloak123@localhost:5432/kubeatlas".to_string()),
            keycloak_url: env::var("KEYCLOAK_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            keycloak_realm: env::var("KEYCLOAK_REALM")
                .unwrap_or_else(|_| "kubeatlas".to_string()),
            keycloak_client_id: env::var("KEYCLOAK_CLIENT_ID")
                .unwrap_or_else(|_| "kubeatlas-backend".to_string()),
            keycloak_client_secret: env::var("KEYCLOAK_CLIENT_SECRET")
                .unwrap_or_else(|_| "backend-secret-key".to_string()),
            jwt_secret: env::var("JWT_SECRET").unwrap_or_else(|_| {
                // Генерируем случайный секрет 64 символа
                rand::thread_rng()
                    .sample_iter(&Alphanumeric)
                    .take(64)
                    .map(char::from)
                    .collect::<String>()
            }),
            log_level: env::var("RUST_LOG")
                .unwrap_or_else(|_| "info".to_string()),
            adm_user: env::var("ADM_USER").ok(),
            adm_password: env::var("ADM_PASSWORD").ok(),
            keycloak_admin_user: env::var("KEYCLOAK_ADMIN_USER").ok(),
            keycloak_admin_password: env::var("KEYCLOAK_ADMIN_PASSWORD").ok(),
        };

        Ok(config)
    }

    pub fn keycloak_issuer_url(&self) -> String {
        format!("{}/realms/{}", self.keycloak_url, self.keycloak_realm)
    }

    pub fn keycloak_token_url(&self) -> String {
        format!("{}/realms/{}/protocol/openid-connect/token", self.keycloak_url, self.keycloak_realm)
    }

    pub fn keycloak_userinfo_url(&self) -> String {
        format!("{}/realms/{}/protocol/openid-connect/userinfo", self.keycloak_url, self.keycloak_realm)
    }

    pub fn keycloak_jwks_url(&self) -> String {
        format!("{}/realms/{}/protocol/openid-connect/certs", self.keycloak_url, self.keycloak_realm)
    }

    pub fn keycloak_logout_url(&self) -> String {
        format!("{}/realms/{}/protocol/openid-connect/logout", self.keycloak_url, self.keycloak_realm)
    }
}
