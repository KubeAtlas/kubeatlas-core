use anyhow::{anyhow, Result};
use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    response::Json,
};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm, TokenData, Header};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::config::Config;
use crate::models::{CreateUserRequest, UpdateUserRequest};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeycloakUser {
    pub sub: String,
    pub preferred_username: String,
    pub email: String,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub realm_access: Option<RealmAccess>,
    pub resource_access: Option<HashMap<String, ResourceAccess>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealmAccess {
    pub roles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAccess {
    pub roles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct KeycloakAccessTokenClaims {
    pub sub: String,
    pub preferred_username: String,
    pub email: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub iss: String,
    pub aud: serde_json::Value,
    pub exp: usize,
    pub iat: Option<usize>,
    pub nbf: Option<usize>,
    pub realm_access: Option<RealmAccess>,
    pub resource_access: Option<HashMap<String, ResourceAccess>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenValidationRequest {
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogoutRequest {
    pub refresh_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenValidationResponse {
    pub valid: bool,
    pub user: Option<KeycloakUser>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfoResponse {
    pub user: KeycloakUser,
    pub roles: Vec<String>,
}

#[derive(Clone)]
pub struct AuthService {
    config: Config,
    client: Client,
}

impl AuthService {
    pub fn new(config: &Config) -> Result<Self> {
        let client = Client::new();
        Ok(Self {
            config: config.clone(),
            client,
        })
    }

    pub async fn ensure_realm_exists(&self) -> Result<()> {
        let master_token = self.get_master_admin_token().await?;
        
        // Check if realm exists
        let realm_url = format!("{}/admin/realms/{}", self.config.keycloak_url, self.config.keycloak_realm);
        let resp = self.client.get(&realm_url).bearer_auth(&master_token).send().await?;
        
        if resp.status().is_success() {
            tracing::info!("Realm '{}' already exists", self.config.keycloak_realm);
            return Ok(());
        }
        
        if resp.status() != reqwest::StatusCode::NOT_FOUND {
            return Err(anyhow!("Failed to check realm existence: HTTP {}", resp.status()));
        }
        
        // Realm doesn't exist, create it
        tracing::info!("Creating realm '{}'", self.config.keycloak_realm);
        self.create_realm_with_token(&master_token).await?;
        
        Ok(())
    }

    async fn get_master_admin_token(&self) -> Result<String> {
        let (admin_user, admin_password) = match (&self.config.keycloak_admin_user, &self.config.keycloak_admin_password) {
            (Some(u), Some(p)) => (u.clone(), p.clone()),
            _ => return Err(anyhow!("Master admin credentials not configured")),
        };

        let params = [
            ("grant_type", "password"),
            ("client_id", "admin-cli"),
            ("username", admin_user.as_str()),
            ("password", admin_password.as_str()),
        ];
        
        let master_token_url = format!("{}/realms/master/protocol/openid-connect/token", self.config.keycloak_url);
        let resp = self.client.post(&master_token_url).form(&params).send().await?;
        
        if !resp.status().is_success() {
            return Err(anyhow!("Failed to get master admin token: HTTP {}", resp.status()));
        }
        
        let token: OAuthTokenResponse = resp.json().await?;
        Ok(token.access_token)
    }

    async fn create_realm_with_token(&self, master_token: &str) -> Result<()> {
        let admin_username = self.config.adm_user.as_deref().unwrap_or("admin-service");
        let admin_password = self.config.adm_password.as_deref().unwrap_or("AdminPassw0rd!");
        let service_account_username = format!("service-account-{}", self.config.keycloak_client_id);
        
        // Create basic realm configuration
        let mut realm_config = serde_json::Map::new();
        realm_config.insert("realm".to_string(), serde_json::Value::String(self.config.keycloak_realm.clone()));
        realm_config.insert("enabled".to_string(), serde_json::Value::Bool(true));
        realm_config.insert("sslRequired".to_string(), serde_json::Value::String("none".to_string()));
        realm_config.insert("registrationAllowed".to_string(), serde_json::Value::Bool(false));
        realm_config.insert("loginWithEmailAllowed".to_string(), serde_json::Value::Bool(true));
        realm_config.insert("duplicateEmailsAllowed".to_string(), serde_json::Value::Bool(false));
        realm_config.insert("resetPasswordAllowed".to_string(), serde_json::Value::Bool(true));
        realm_config.insert("editUsernameAllowed".to_string(), serde_json::Value::Bool(false));
        realm_config.insert("bruteForceProtected".to_string(), serde_json::Value::Bool(true));
        realm_config.insert("accessTokenLifespan".to_string(), serde_json::Value::Number(serde_json::Number::from(3600)));
        realm_config.insert("ssoSessionIdleTimeout".to_string(), serde_json::Value::Number(serde_json::Number::from(1800)));
        realm_config.insert("ssoSessionMaxLifespan".to_string(), serde_json::Value::Number(serde_json::Number::from(36000)));
        
        // Create client configuration
        let mut client = serde_json::Map::new();
        client.insert("clientId".to_string(), serde_json::Value::String(self.config.keycloak_client_id.clone()));
        client.insert("name".to_string(), serde_json::Value::String("KubeAtlas Backend".to_string()));
        client.insert("enabled".to_string(), serde_json::Value::Bool(true));
        client.insert("publicClient".to_string(), serde_json::Value::Bool(false));
        client.insert("serviceAccountsEnabled".to_string(), serde_json::Value::Bool(true));
        client.insert("directAccessGrantsEnabled".to_string(), serde_json::Value::Bool(true));
        client.insert("standardFlowEnabled".to_string(), serde_json::Value::Bool(true));
        client.insert("fullScopeAllowed".to_string(), serde_json::Value::Bool(true));
        client.insert("secret".to_string(), serde_json::Value::String(self.config.keycloak_client_secret.clone()));
        
        let default_scopes = vec!["roles", "profile", "email"];
        client.insert("defaultClientScopes".to_string(), serde_json::Value::Array(
            default_scopes.into_iter().map(|s| serde_json::Value::String(s.to_string())).collect()
        ));
        client.insert("redirectUris".to_string(), serde_json::Value::Array(vec![serde_json::Value::String("*".to_string())]));
        client.insert("webOrigins".to_string(), serde_json::Value::Array(vec![serde_json::Value::String("*".to_string())]));
        
        realm_config.insert("clients".to_string(), serde_json::Value::Array(vec![serde_json::Value::Object(client)]));
        
        // Create roles
        let mut roles = serde_json::Map::new();
        let realm_roles = vec![
            serde_json::json!({"name": "admin"}),
            serde_json::json!({"name": "user"}),
            serde_json::json!({"name": "guest"})
        ];
        roles.insert("realm".to_string(), serde_json::Value::Array(realm_roles));
        realm_config.insert("roles".to_string(), serde_json::Value::Object(roles));
        
        // Create users - only service account, admin user will be created separately
        let service_account = serde_json::json!({
            "username": service_account_username,
            "enabled": true,
            "serviceAccountClientId": self.config.keycloak_client_id,
            "clientRoles": {
                "realm-management": [
                    "manage-users",
                    "view-users",
                    "query-users",
                    "view-realm",
                    "manage-realm",
                    "view-clients",
                    "manage-clients"
                ]
            }
        });
        
        realm_config.insert("users".to_string(), serde_json::Value::Array(vec![service_account]));

        let realms_url = format!("{}/admin/realms", self.config.keycloak_url);
        let resp = self.client.post(&realms_url)
            .bearer_auth(master_token)
            .json(&serde_json::Value::Object(realm_config))
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to create realm: HTTP {} - {}", status, text));
        }

        tracing::info!("Realm '{}' created successfully", self.config.keycloak_realm);
        
        // Wait a bit for realm to be fully ready
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        
        Ok(())
    }

    pub async fn ensure_admin_user(&self) -> Result<()> {
        let username = match (&self.config.adm_user, &self.config.adm_password) {
            (Some(u), Some(_)) => u.clone(),
            _ => return Ok(()),
        };

        // Check if user exists
        // Retry, так как Keycloak может быть не готов сразу после старта
        let mut last_err: Option<anyhow::Error> = None;
        for _ in 0..10u8 {
            match self.get_admin_access_token().await {
                Ok(token) => {
                    // пытаемся выполнить проверку/создание
                    if let Err(e) = self.ensure_admin_user_with_token(&token, &username).await {
                        last_err = Some(e);
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                        continue;
                    }
                    return Ok(());
                }
                Err(e) => {
                    last_err = Some(e);
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                }
            }
        }
        Err(last_err.unwrap_or_else(|| anyhow!("Unknown error ensuring admin user")))
    }

    pub async fn wait_for_keycloak_ready(&self, max_wait_secs: u64) -> Result<()> {
        let start = Instant::now();
        let config_url = format!(
            "{}/realms/{}/.well-known/openid-configuration",
            self.config.keycloak_url, self.config.keycloak_realm
        );

        loop {
            // 1) Проверяем .well-known
            let ok_config = self
                .client
                .get(&config_url)
                .send()
                .await
                .map(|r| r.status().is_success())
                .unwrap_or(false);

            // 2) Проверяем JWKS
            let ok_jwks = self
                .client
                .get(self.config.keycloak_jwks_url())
                .send()
                .await
                .map(|r| r.status().is_success())
                .unwrap_or(false);

            // 3) Пробуем получить сервисный токен клиента (client_credentials)
            let ok_token = self.get_admin_access_token().await.is_ok();

            if ok_config && ok_jwks && ok_token {
                tracing::info!("Keycloak is ready");
                return Ok(());
            }

            if start.elapsed() > Duration::from_secs(max_wait_secs) {
                return Err(anyhow!("Keycloak not ready after {}s", max_wait_secs));
            }

            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }

    async fn ensure_admin_user_with_token(&self, admin_token: &str, username: &str) -> Result<()> {
        let query_url = format!(
            "{}/admin/realms/{}/users",
            self.config.keycloak_url, self.config.keycloak_realm
        );
        let resp = self
            .client
            .get(&query_url)
            .bearer_auth(admin_token)
            .query(&[("username", username)])
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(anyhow!("Failed to query users: HTTP {}", resp.status()));
        }
        let users: Vec<serde_json::Value> = resp.json().await?;
        if let Some(existing) = users.first() {
            let user_id = existing
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            if user_id.is_empty() {
                return Ok(());
            }
            // Ensure admin role is present
            self.assign_realm_roles(admin_token, &user_id, &vec!["admin".to_string()])
                .await?;
            return Ok(());
        }

        // Create new admin user
        let req = CreateUserRequest {
            username: username.to_string(),
            email: format!("{}@local", username),
            first_name: Some("Admin".to_string()),
            last_name: Some("User".to_string()),
            password: self.config.adm_password.clone().unwrap_or_else(|| "admin".to_string()),
            roles: vec!["admin".to_string()],
        };
        let _ = self.create_keycloak_user(req).await?;
        Ok(())
    }

    pub async fn ensure_realm_admin_role(&self) -> Result<()> {
        let admin_user = match (&self.config.keycloak_admin_user, &self.config.keycloak_admin_password) {
            (Some(u), Some(p)) => (u.clone(), p.clone()),
            _ => return Ok(()),
        };

        // Получаем токен администратора master-реалма (admin-cli)
        let params = [
            ("grant_type", "password"),
            ("client_id", "admin-cli"),
            ("username", admin_user.0.as_str()),
            ("password", admin_user.1.as_str()),
        ];
        let master_token_url = format!("{}/realms/master/protocol/openid-connect/token", self.config.keycloak_url);
        let resp = self.client.post(&master_token_url).form(&params).send().await?;
        if !resp.status().is_success() {
            return Err(anyhow!("Failed to get master admin token: HTTP {}", resp.status()));
        }
        let token: OAuthTokenResponse = resp.json().await?;

        // Находим userId admin-service
        let users_url = format!("{}/admin/realms/{}/users", self.config.keycloak_url, self.config.keycloak_realm);
        let list = self.client.get(&users_url)
            .bearer_auth(&token.access_token)
            .query(&[("username", self.config.adm_user.as_deref().unwrap_or("admin-service"))])
            .send().await?;
        if !list.status().is_success() { return Err(anyhow!("Failed to query users: HTTP {}", list.status())); }
        let users: Vec<serde_json::Value> = list.json().await?;
        let user_id = users.get(0).and_then(|u| u.get("id")).and_then(|v| v.as_str()).ok_or_else(|| anyhow!("admin-service user not found"))?;

        // Забираем описание роли admin
        let role_url = format!("{}/admin/realms/{}/roles/{}", self.config.keycloak_url, self.config.keycloak_realm, "admin");
        let role_resp = self.client.get(&role_url).bearer_auth(&token.access_token).send().await?;
        if !role_resp.status().is_success() { return Err(anyhow!("Failed to get role: HTTP {}", role_resp.status())); }
        let role_json: serde_json::Value = role_resp.json().await?;

        // Назначаем роль admin пользователю
        let map_url = format!("{}/admin/realms/{}/users/{}/role-mappings/realm", self.config.keycloak_url, self.config.keycloak_realm, user_id);
        let assign = self.client.post(&map_url)
            .bearer_auth(&token.access_token)
            .json(&vec![role_json])
            .send().await?;
        if !assign.status().is_success() {
            return Err(anyhow!("Failed to assign admin role: HTTP {}", assign.status()));
        }
        Ok(())
    }

    pub async fn validate_token(&self, token: &str) -> Result<TokenValidationResponse> {
        // 1) Попытка локальной проверки по JWKS
        if let Ok(user) = self.validate_with_jwks(token).await {
            return Ok(TokenValidationResponse { valid: true, user: Some(user), error: None });
        }

        // 2) Фоллбэк на userinfo (если JWKS не сработал)
        match self.validate_with_keycloak(token).await {
            Ok(user) => Ok(TokenValidationResponse { valid: true, user: Some(user), error: None }),
            Err(e) => {
                tracing::warn!("Token validation failed: {}", e);
                Ok(TokenValidationResponse { valid: false, user: None, error: Some(e.to_string()) })
            }
        }
    }

    async fn validate_with_keycloak(&self, token: &str) -> Result<KeycloakUser> {
        let userinfo_url = self.config.keycloak_userinfo_url();
        
        let response = self
            .client
            .get(&userinfo_url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Invalid token: HTTP {}", response.status()));
        }

        let user: KeycloakUser = response.json().await?;
        Ok(user)
    }

    async fn validate_with_jwks(&self, token: &str) -> Result<KeycloakUser> {
        // Получаем заголовок токена, чтобы извлечь kid
        let header = jsonwebtoken::decode_header(token)?;
        let kid = header.kid.ok_or_else(|| anyhow!("JWT header missing kid"))?;

        // Загружаем JWKS
        let jwks = self.client.get(self.config.keycloak_jwks_url()).send().await?.json::<serde_json::Value>().await?;
        let keys = jwks.get("keys").and_then(|k| k.as_array()).ok_or_else(|| anyhow!("JWKS keys missing"))?;
        let jwk = keys.iter().find(|k| k.get("kid").and_then(|v| v.as_str()) == Some(kid.as_str()))
            .ok_or_else(|| anyhow!("No matching JWK for kid"))?;

        let n = jwk.get("n").and_then(|v| v.as_str()).ok_or_else(|| anyhow!("JWK missing n"))?;
        let e = jwk.get("e").and_then(|v| v.as_str()).ok_or_else(|| anyhow!("JWK missing e"))?;

        let decoding_key = DecodingKey::from_rsa_components(n, e)?;

        let mut validation = Validation::new(Algorithm::RS256);
        // Не проверяем audience, т.к. в Keycloak aud может отличаться (строка/массив/"account")

        let token_data: TokenData<KeycloakAccessTokenClaims> = decode::<KeycloakAccessTokenClaims>(token, &decoding_key, &validation)?;

        let claims = token_data.claims;
        tracing::debug!("JWT claims validated: sub={}, preferred_username={}", claims.sub, claims.preferred_username);
        let user = KeycloakUser {
            sub: claims.sub,
            preferred_username: claims.preferred_username,
            email: claims.email.unwrap_or_default(),
            given_name: claims.given_name,
            family_name: claims.family_name,
            realm_access: claims.realm_access,
            resource_access: claims.resource_access,
        };

        Ok(user)
    }

    pub fn extract_token_from_headers(&self, headers: &HeaderMap) -> Result<String> {
        let auth_header = headers
            .get("Authorization")
            .ok_or_else(|| anyhow!("Missing Authorization header"))?
            .to_str()
            .map_err(|_| anyhow!("Invalid Authorization header"))?;

        if !auth_header.starts_with("Bearer ") {
            return Err(anyhow!("Invalid Authorization header format"));
        }

        Ok(auth_header[7..].to_string())
    }

    pub fn get_user_roles(&self, user: &KeycloakUser) -> Vec<String> {
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
        
        roles
    }

    pub fn has_role(&self, user: &KeycloakUser, role: &str) -> bool {
        self.get_user_roles(user).contains(&role.to_string())
    }

    pub fn is_admin(&self, user: &KeycloakUser) -> bool {
        self.has_role(user, "admin")
    }

    pub fn is_user(&self, user: &KeycloakUser) -> bool {
        self.has_role(user, "user")
    }

    pub fn is_guest(&self, user: &KeycloakUser) -> bool {
        self.has_role(user, "guest")
    }

    pub async fn refresh_token(&self, refresh_token: &str) -> Result<RefreshTokenResponse> {
        let params = [
            ("grant_type", "refresh_token"),
            ("client_id", self.config.keycloak_client_id.as_str()),
            ("client_secret", self.config.keycloak_client_secret.as_str()),
            ("refresh_token", refresh_token),
        ];

        let resp = self
            .client
            .post(&self.config.keycloak_token_url())
            .form(&params)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(anyhow!("Failed to refresh token: HTTP {}", resp.status()));
        }

        let token_resp: OAuthTokenResponse = resp.json().await?;
        Ok(RefreshTokenResponse {
            access_token: token_resp.access_token,
            refresh_token: token_resp.refresh_token.unwrap_or_default(),
            expires_in: token_resp.expires_in,
        })
    }

    pub async fn logout(&self, refresh_token: &str) -> Result<()> {
        let params = [
            ("client_id", self.config.keycloak_client_id.as_str()),
            ("client_secret", self.config.keycloak_client_secret.as_str()),
            ("refresh_token", refresh_token),
        ];

        let logout_url = self.config.keycloak_logout_url();

        let resp = self
            .client
            .post(&logout_url)
            .form(&params)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(anyhow!("Failed to logout: HTTP {}", resp.status()));
        }

        Ok(())
    }

    pub async fn get_active_sessions(&self, user_id: &str) -> Result<Vec<serde_json::Value>> {
        let admin_token = self.get_admin_access_token().await?;
        
        let sessions_url = format!(
            "{}/admin/realms/{}/users/{}/sessions",
            self.config.keycloak_url, self.config.keycloak_realm, user_id
        );

        let resp = self
            .client
            .get(&sessions_url)
            .bearer_auth(&admin_token)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(anyhow!("Failed to get sessions: HTTP {}", resp.status()));
        }

        let sessions: Vec<serde_json::Value> = resp.json().await?;
        Ok(sessions)
    }

    pub async fn revoke_user_sessions(&self, user_id: &str) -> Result<()> {
        let admin_token = self.get_admin_access_token().await?;
        
        let logout_url = format!(
            "{}/admin/realms/{}/users/{}/logout",
            self.config.keycloak_url, self.config.keycloak_realm, user_id
        );

        let resp = self
            .client
            .post(&logout_url)
            .bearer_auth(&admin_token)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(anyhow!("Failed to revoke sessions: HTTP {}", resp.status()));
        }

        Ok(())
    }

    pub async fn revoke_specific_session(&self, session_id: &str) -> Result<()> {
        let admin_token = self.get_admin_access_token().await?;
        
        let session_url = format!(
            "{}/admin/realms/{}/sessions/{}",
            self.config.keycloak_url, self.config.keycloak_realm, session_id
        );

        let resp = self
            .client
            .delete(&session_url)
            .bearer_auth(&admin_token)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(anyhow!("Failed to revoke specific session: HTTP {}", resp.status()));
        }

        Ok(())
    }

    pub async fn get_all_users(&self) -> Result<Vec<serde_json::Value>> {
        let admin_token = self.get_admin_access_token().await?;
        
        let users_url = format!(
            "{}/admin/realms/{}/users",
            self.config.keycloak_url, self.config.keycloak_realm
        );

        let resp = self
            .client
            .get(&users_url)
            .bearer_auth(&admin_token)
            .query(&[("max", "100")]) // Limit to 100 users
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(anyhow!("Failed to get users: HTTP {}", resp.status()));
        }

        let users: Vec<serde_json::Value> = resp.json().await?;
        Ok(users)
    }

    pub async fn get_user_by_id(&self, user_id: &str) -> Result<serde_json::Value> {
        let admin_token = self.get_admin_access_token().await?;
        
        let user_url = format!(
            "{}/admin/realms/{}/users/{}",
            self.config.keycloak_url, self.config.keycloak_realm, user_id
        );

        let resp = self
            .client
            .get(&user_url)
            .bearer_auth(&admin_token)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(anyhow!("Failed to get user: HTTP {}", resp.status()));
        }

        let user: serde_json::Value = resp.json().await?;
        Ok(user)
    }

    pub async fn get_user_roles_by_id(&self, user_id: &str) -> Result<Vec<serde_json::Value>> {
        let admin_token = self.get_admin_access_token().await?;
        
        let roles_url = format!(
            "{}/admin/realms/{}/users/{}/role-mappings/realm",
            self.config.keycloak_url, self.config.keycloak_realm, user_id
        );

        let resp = self
            .client
            .get(&roles_url)
            .bearer_auth(&admin_token)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(anyhow!("Failed to get user roles: HTTP {}", resp.status()));
        }

        let roles: Vec<serde_json::Value> = resp.json().await?;
        Ok(roles)
    }
}

// --------------------------
// Keycloak Admin API helpers
// --------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OAuthTokenResponse {
    access_token: String,
    token_type: String,
    expires_in: i64,
    refresh_token: Option<String>,
    #[serde(alias = "refresh_expires_in")]
    refresh_expires_in: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RoleRepresentation {
    id: String,
    name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserRepresentation<'a> {
    username: &'a str,
    email: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    firstName: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    lastName: Option<&'a str>,
    enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CredentialRepresentation<'a> {
    r#type: &'a str,
    value: &'a str,
    temporary: bool,
}

impl AuthService {
    pub async fn get_admin_access_token(&self) -> Result<String> {
        let params = [
            ("grant_type", "client_credentials"),
            ("client_id", self.config.keycloak_client_id.as_str()),
            ("client_secret", self.config.keycloak_client_secret.as_str()),
        ];

        let resp = self
            .client
            .post(&self.config.keycloak_token_url())
            .form(&params)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(anyhow!("Failed to get admin token: HTTP {}", resp.status()));
        }

        let token: OAuthTokenResponse = resp.json().await?;
        Ok(token.access_token)
    }

    pub async fn create_keycloak_user(&self, req: CreateUserRequest) -> Result<String> {
        let admin_token = self.get_admin_access_token().await?;

        let user_rep = UserRepresentation {
            username: &req.username,
            email: &req.email,
            firstName: req.first_name.as_deref(),
            lastName: req.last_name.as_deref(),
            enabled: true,
        };

        // Create user
        let users_url = format!(
            "{}/admin/realms/{}/users",
            self.config.keycloak_url, self.config.keycloak_realm
        );

        let resp = self
            .client
            .post(&users_url)
            .bearer_auth(&admin_token)
            .json(&user_rep)
            .send()
            .await?;

        if resp.status() != reqwest::StatusCode::CREATED {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to create user: HTTP {} - {}", status, text));
        }

        // Extract user id from Location header
        let location = resp
            .headers()
            .get(reqwest::header::LOCATION)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| anyhow!("Location header missing in create user response"))?;

        let user_id = location
            .rsplit('/')
            .next()
            .ok_or_else(|| anyhow!("Cannot parse user id from Location header"))?
            .to_string();

        // Set password
        let cred = CredentialRepresentation {
            r#type: "password",
            value: &req.password,
            temporary: false,
        };

        let pwd_url = format!(
            "{}/admin/realms/{}/users/{}/reset-password",
            self.config.keycloak_url, self.config.keycloak_realm, user_id
        );

        let resp = self
            .client
            .put(&pwd_url)
            .bearer_auth(&admin_token)
            .json(&cred)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(anyhow!(
                "Failed to set password: HTTP {}",
                resp.status()
            ));
        }

        // Assign realm roles if provided
        if !req.roles.is_empty() {
            self.assign_realm_roles(&admin_token, &user_id, &req.roles).await?;
        }

        Ok(user_id)
    }

    pub async fn update_keycloak_user(
        &self,
        user_id: &str,
        req: UpdateUserRequest,
    ) -> Result<()> {
        let admin_token = self.get_admin_access_token().await?;

        // Update basic fields
        let user_url = format!(
            "{}/admin/realms/{}/users/{}",
            self.config.keycloak_url, self.config.keycloak_realm, user_id
        );

        // We must send a full representation for fields we want to change
        // Fetch current user to avoid wiping fields
        let current_resp = self
            .client
            .get(&user_url)
            .bearer_auth(&admin_token)
            .send()
            .await?;
        if !current_resp.status().is_success() {
            return Err(anyhow!("Failed to fetch user: HTTP {}", current_resp.status()));
        }
        let mut current: serde_json::Value = current_resp.json().await?;

        if let Some(email) = req.email {
            current["email"] = serde_json::Value::String(email);
        }
        if let Some(first_name) = req.first_name {
            current["firstName"] = serde_json::Value::String(first_name);
        }
        if let Some(last_name) = req.last_name {
            current["lastName"] = serde_json::Value::String(last_name);
        }

        let resp = self
            .client
            .put(&user_url)
            .bearer_auth(&admin_token)
            .json(&current)
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(anyhow!("Failed to update user: HTTP {}", resp.status()));
        }

        // Update roles if provided
        if let Some(roles) = req.roles {
            self.replace_realm_roles(&admin_token, user_id, &roles).await?;
        }

        Ok(())
    }

    async fn assign_realm_roles(
        &self,
        admin_token: &str,
        user_id: &str,
        roles: &Vec<String>,
    ) -> Result<()> {
        let mut role_reps: Vec<RoleRepresentation> = Vec::new();
        for role in roles {
            if let Some(rep) = self.get_realm_role_representation(admin_token, role).await? {
                role_reps.push(rep);
            } else {
                return Err(anyhow!("Role '{}' not found", role));
            }
        }

        let url = format!(
            "{}/admin/realms/{}/users/{}/role-mappings/realm",
            self.config.keycloak_url, self.config.keycloak_realm, user_id
        );

        let resp = self
            .client
            .post(&url)
            .bearer_auth(admin_token)
            .json(&role_reps)
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(anyhow!(
                "Failed to assign roles: HTTP {}",
                resp.status()
            ));
        }
        Ok(())
    }

    async fn replace_realm_roles(
        &self,
        admin_token: &str,
        user_id: &str,
        roles: &Vec<String>,
    ) -> Result<()> {
        // Fetch current roles
        let current_roles_url = format!(
            "{}/admin/realms/{}/users/{}/role-mappings/realm",
            self.config.keycloak_url, self.config.keycloak_realm, user_id
        );
        let current_resp = self
            .client
            .get(&current_roles_url)
            .bearer_auth(admin_token)
            .send()
            .await?;
        if !current_resp.status().is_success() {
            return Err(anyhow!(
                "Failed to get current roles: HTTP {}",
                current_resp.status()
            ));
        }
        let current: Vec<RoleRepresentation> = current_resp.json().await?;

        // Remove all current roles
        if !current.is_empty() {
            let del_resp = self
                .client
                .delete(&current_roles_url)
                .bearer_auth(admin_token)
                .json(&current)
                .send()
                .await?;
            if !del_resp.status().is_success() {
                return Err(anyhow!(
                    "Failed to remove current roles: HTTP {}",
                    del_resp.status()
                ));
            }
        }

        // Assign provided roles
        self.assign_realm_roles(admin_token, user_id, roles).await
    }

    pub async fn delete_keycloak_user(&self, user_id: &str) -> Result<()> {
        let admin_token = self.get_admin_access_token().await?;
        
        let user_url = format!(
            "{}/admin/realms/{}/users/{}",
            self.config.keycloak_url, self.config.keycloak_realm, user_id
        );

        let resp = self
            .client
            .delete(&user_url)
            .bearer_auth(&admin_token)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(anyhow!(
                "Failed to delete user: HTTP {}",
                resp.status()
            ));
        }

        Ok(())
    }

    async fn get_realm_role_representation(
        &self,
        admin_token: &str,
        role_name: &str,
    ) -> Result<Option<RoleRepresentation>> {
        let url = format!(
            "{}/admin/realms/{}/roles/{}",
            self.config.keycloak_url, self.config.keycloak_realm, role_name
        );
        let resp = self
            .client
            .get(&url)
            .bearer_auth(admin_token)
            .send()
            .await?;
        match resp.status() {
            reqwest::StatusCode::OK => {
                let rep: RoleRepresentation = resp.json().await?;
                Ok(Some(rep))
            }
            reqwest::StatusCode::NOT_FOUND => Ok(None),
            s => Err(anyhow!("Failed to get role '{}': HTTP {}", role_name, s)),
        }
    }

    /// Get total number of users in the realm
    pub async fn get_total_users_count(&self) -> Result<u64> {
        let admin_token = self.get_admin_access_token().await?;
        let url = format!(
            "{}/admin/realms/{}/users/count",
            self.config.keycloak_url, self.config.keycloak_realm
        );

        let resp = self
            .client
            .get(&url)
            .bearer_auth(&admin_token)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(anyhow!("Failed to get users count: HTTP {}", resp.status()));
        }

        let count: u64 = resp.json().await?;
        Ok(count)
    }

    /// Get number of active sessions (approximation based on recent user activity)
    pub async fn get_active_sessions_count(&self) -> Result<u64> {
        let admin_token = self.get_admin_access_token().await?;
        
        // Get all users first
        let users_url = format!(
            "{}/admin/realms/{}/users",
            self.config.keycloak_url, self.config.keycloak_realm
        );

        let resp = self
            .client
            .get(&users_url)
            .bearer_auth(&admin_token)
            .query(&[("max", "1000")]) // Limit to reasonable number
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(anyhow!("Failed to get users: HTTP {}", resp.status()));
        }

        let users: Vec<serde_json::Value> = resp.json().await?;
        let mut active_sessions = 0u64;

        // For each user, check if they have active sessions
        for user in users.iter().take(100) { // Limit to avoid too many requests
            if let Some(user_id) = user.get("id").and_then(|v| v.as_str()) {
                let sessions_url = format!(
                    "{}/admin/realms/{}/users/{}/sessions",
                    self.config.keycloak_url, self.config.keycloak_realm, user_id
                );

                if let Ok(sessions_resp) = self
                    .client
                    .get(&sessions_url)
                    .bearer_auth(&admin_token)
                    .send()
                    .await
                {
                    if sessions_resp.status().is_success() {
                        if let Ok(sessions) = sessions_resp.json::<Vec<serde_json::Value>>().await {
                            active_sessions += sessions.len() as u64;
                        }
                    }
                }
            }
        }

        Ok(active_sessions)
    }

    /// Health check for Keycloak connectivity
    pub async fn health_check(&self) -> Result<()> {
        let config_url = format!(
            "{}/realms/{}/.well-known/openid-configuration",
            self.config.keycloak_url, self.config.keycloak_realm
        );

        let resp = self.client.get(&config_url).send().await?;
        
        if resp.status().is_success() {
            Ok(())
        } else {
            Err(anyhow!("Keycloak health check failed: HTTP {}", resp.status()))
        }
    }
}
