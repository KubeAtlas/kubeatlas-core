use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use rand::Rng;
use redis::{AsyncCommands, Client};
use base64::{engine::general_purpose, Engine as _};

use crate::models::InstallTokenData;

#[derive(Clone)]
pub struct TokenService {
    pub redis: Client,
}

impl TokenService {
    pub fn new(redis_url: &str) -> Result<Self> {
        let redis = Client::open(redis_url)
            .context("Failed to create Redis client")?;
        
        Ok(Self { redis })
    }

    pub async fn generate_install_token(
        &self,
        token_data: InstallTokenData,
        expires_in_hours: i64,
    ) -> Result<String> {
        let raw_token = self.generate_secure_token();

        let token_key = format!("install_token:{}", raw_token);
        let token_json = serde_json::to_string(&token_data)
            .context("Failed to serialize token data")?;

        let mut conn = self.redis.get_async_connection()
            .await
            .context("Failed to connect to Redis")?;

        conn.set_ex(&token_key, token_json, expires_in_hours as u64 * 3600)
            .await
            .context("Failed to store token in Redis")?;

        Ok(raw_token)
    }

    pub async fn validate_install_token(&self, token: &str) -> Result<InstallTokenData> {
        let token_key = format!("install_token:{}", token);

        let mut conn = self.redis.get_async_connection()
            .await
            .context("Failed to connect to Redis")?;

        let token_json: Option<String> = conn.get(&token_key)
            .await
            .context("Failed to get token from Redis")?;

        let token_json = token_json
            .ok_or_else(|| anyhow::anyhow!("Invalid or expired install token"))?;

        let token_data: InstallTokenData = serde_json::from_str(&token_json)
            .context("Failed to deserialize token data")?;

        if token_data.expires_at < Utc::now() {
            let _: () = conn.del(&token_key).await.ok().unwrap_or(());
            return Err(anyhow::anyhow!("Token has expired"));
        }

        Ok(token_data)
    }

    pub async fn consume_install_token(&self, token: &str) -> Result<InstallTokenData> {
        let token_data = self.validate_install_token(token).await?;
        
        let token_key = format!("install_token:{}", token);
        let mut conn = self.redis.get_async_connection()
            .await
            .context("Failed to connect to Redis")?;

        let _: () = conn.del(&token_key)
            .await
            .context("Failed to delete token from Redis")?;

        Ok(token_data)
    }

    pub async fn cleanup_expired_tokens(&self) -> Result<()> {
        Ok(())
    }

    fn generate_secure_token(&self) -> String {
        let mut rng = rand::thread_rng();
        let token_bytes: [u8; 32] = rng.gen();
        general_purpose::URL_SAFE_NO_PAD.encode(token_bytes)
    }
}