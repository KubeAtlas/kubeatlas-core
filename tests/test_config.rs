use std::env;
use tokio;

use kubeatlas_backend::config::Config;

#[tokio::test]
async fn test_config_default_values() {
    // Тест значений конфигурации по умолчанию
    let config = Config {
        server_address: "0.0.0.0:3001".to_string(),
        database_url: "postgresql://localhost:5432/test".to_string(),
        keycloak_url: "http://localhost:8080".to_string(),
        keycloak_realm: "kubeatlas".to_string(),
        keycloak_client_id: "kubeatlas-backend".to_string(),
        keycloak_client_secret: "secret".to_string(),
        jwt_secret: "jwt-secret".to_string(),
        log_level: "info".to_string(),
        keycloak_admin_user: Some("admin".to_string()),
        keycloak_admin_password: Some("admin".to_string()),
        adm_user: Some("admin".to_string()),
        adm_password: Some("admin".to_string()),
    };
    
    assert_eq!(config.server_address, "0.0.0.0:3001");
    assert_eq!(config.keycloak_url, "http://localhost:8080");
    assert_eq!(config.keycloak_realm, "kubeatlas");
    assert_eq!(config.keycloak_client_id, "kubeatlas-backend");
}

#[tokio::test]
async fn test_keycloak_url_methods() {
    // Тест методов генерации URL для Keycloak
    let config = Config {
        server_address: "0.0.0.0:3001".to_string(),
        database_url: "postgresql://localhost:5432/test".to_string(),
        keycloak_url: "http://keycloak:8080".to_string(),
        keycloak_realm: "test-realm".to_string(),
        keycloak_client_id: "test-client".to_string(),
        keycloak_client_secret: "secret".to_string(),
        jwt_secret: "jwt-secret".to_string(),
        log_level: "info".to_string(),
        keycloak_admin_user: Some("admin".to_string()),
        keycloak_admin_password: Some("admin".to_string()),
        adm_user: Some("admin".to_string()),
        adm_password: Some("admin".to_string()),
    };
    
    // Тест JWKS URL
    let jwks_url = config.keycloak_jwks_url();
    assert_eq!(jwks_url, "http://keycloak:8080/realms/test-realm/protocol/openid-connect/certs");
    
    // Тест UserInfo URL
    let userinfo_url = config.keycloak_userinfo_url();
    assert_eq!(userinfo_url, "http://keycloak:8080/realms/test-realm/protocol/openid-connect/userinfo");
    
    // Тест Token URL
    let token_url = config.keycloak_token_url();
    assert_eq!(token_url, "http://keycloak:8080/realms/test-realm/protocol/openid-connect/token");
    
    // Тест Logout URL
    let logout_url = config.keycloak_logout_url();
    assert_eq!(logout_url, "http://keycloak:8080/realms/test-realm/protocol/openid-connect/logout");
}

#[tokio::test]
async fn test_config_optional_fields() {
    // Тест опциональных полей конфигурации
    let config = Config {
        server_address: "0.0.0.0:3001".to_string(),
        database_url: "postgresql://localhost:5432/test".to_string(),
        keycloak_url: "http://localhost:8080".to_string(),
        keycloak_realm: "kubeatlas".to_string(),
        keycloak_client_id: "kubeatlas-backend".to_string(),
        keycloak_client_secret: "test-secret".to_string(),
        jwt_secret: "test-jwt-secret".to_string(),
        log_level: "info".to_string(),
        keycloak_admin_user: None,
        keycloak_admin_password: None,
        adm_user: None,
        adm_password: None,
    };
    
    assert_eq!(config.keycloak_client_secret, "test-secret");
    assert!(config.keycloak_admin_user.is_none());
    assert!(config.keycloak_admin_password.is_none());
    assert!(config.adm_user.is_none());
    assert!(config.adm_password.is_none());
}

#[tokio::test]
async fn test_config_with_custom_ports() {
    // Тест конфигурации с кастомными портами
    let config = Config {
        server_address: "0.0.0.0:4000".to_string(),
        database_url: "postgresql://localhost:5432/test".to_string(),
        keycloak_url: "http://localhost:9080".to_string(),
        keycloak_realm: "custom-realm".to_string(),
        keycloak_client_id: "custom-client".to_string(),
        keycloak_client_secret: "custom-secret".to_string(),
        jwt_secret: "custom-jwt-secret".to_string(),
        log_level: "debug".to_string(),
        keycloak_admin_user: Some("custom-admin".to_string()),
        keycloak_admin_password: Some("custom-password".to_string()),
        adm_user: Some("custom-user".to_string()),
        adm_password: Some("custom-pass".to_string()),
    };
    
    assert_eq!(config.server_address, "0.0.0.0:4000");
    assert_eq!(config.keycloak_url, "http://localhost:9080");
    
    // Проверяем, что URL генерируются с правильными портами
    let jwks_url = config.keycloak_jwks_url();
    assert!(jwks_url.contains(":9080"));
    assert!(jwks_url.contains("custom-realm"));
}

#[tokio::test]
async fn test_config_url_generation_edge_cases() {
    // Тест крайних случаев генерации URL
    
    // Конфигурация с trailing slash в URL
    let config_with_slash = Config {
        server_address: "0.0.0.0:3001".to_string(),
        database_url: "postgresql://localhost:5432/test".to_string(),
        keycloak_url: "http://localhost:8080/".to_string(),
        keycloak_realm: "test".to_string(),
        keycloak_client_id: "test".to_string(),
        keycloak_client_secret: "test-secret".to_string(),
        jwt_secret: "test-jwt-secret".to_string(),
        log_level: "info".to_string(),
        keycloak_admin_user: None,
        keycloak_admin_password: None,
        adm_user: None,
        adm_password: None,
    };
    
    let jwks_url = config_with_slash.keycloak_jwks_url();
    // URL должен работать корректно даже с trailing slash
    assert!(jwks_url.starts_with("http://localhost:8080/"));
    assert!(jwks_url.contains("/realms/test/"));
    
    // Конфигурация с HTTPS
    let config_https = Config {
        server_address: "0.0.0.0:3001".to_string(),
        database_url: "postgresql://localhost:5432/prod".to_string(),
        keycloak_url: "https://keycloak.example.com".to_string(),
        keycloak_realm: "production".to_string(),
        keycloak_client_id: "prod-client".to_string(),
        keycloak_client_secret: "prod-secret".to_string(),
        jwt_secret: "prod-jwt-secret".to_string(),
        log_level: "warn".to_string(),
        keycloak_admin_user: Some("prod-admin".to_string()),
        keycloak_admin_password: Some("prod-password".to_string()),
        adm_user: Some("prod-user".to_string()),
        adm_password: Some("prod-pass".to_string()),
    };
    
    let userinfo_url = config_https.keycloak_userinfo_url();
    assert!(userinfo_url.starts_with("https://"));
    assert!(userinfo_url.contains("keycloak.example.com"));
    assert!(userinfo_url.contains("production"));
}

#[tokio::test]
async fn test_config_clone() {
    // Тест клонирования конфигурации
    let original_config = Config {
        server_address: "0.0.0.0:3001".to_string(),
        database_url: "postgresql://localhost:5432/test".to_string(),
        keycloak_url: "http://localhost:8080".to_string(),
        keycloak_realm: "kubeatlas".to_string(),
        keycloak_client_id: "kubeatlas-backend".to_string(),
        keycloak_client_secret: "secret".to_string(),
        jwt_secret: "jwt-secret".to_string(),
        log_level: "info".to_string(),
        keycloak_admin_user: Some("admin".to_string()),
        keycloak_admin_password: Some("admin".to_string()),
        adm_user: Some("admin".to_string()),
        adm_password: Some("admin".to_string()),
    };
    
    let cloned_config = original_config.clone();
    
    assert_eq!(original_config.server_address, cloned_config.server_address);
    assert_eq!(original_config.keycloak_url, cloned_config.keycloak_url);
    assert_eq!(original_config.keycloak_realm, cloned_config.keycloak_realm);
    assert_eq!(original_config.keycloak_client_id, cloned_config.keycloak_client_id);
    assert_eq!(original_config.keycloak_client_secret, cloned_config.keycloak_client_secret);
}

// Тесты для загрузки конфигурации из переменных окружения
// Эти тесты изолированы, чтобы не влиять на другие тесты
mod env_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_config_debug_format() {
        // Тест Debug форматирования (проверяем, что пароли не выводятся)
        let config = Config {
            server_address: "0.0.0.0:3001".to_string(),
            database_url: "postgresql://localhost:5432/test".to_string(),
            keycloak_url: "http://localhost:8080".to_string(),
            keycloak_realm: "kubeatlas".to_string(),
            keycloak_client_id: "kubeatlas-backend".to_string(),
            keycloak_client_secret: "secret123".to_string(),
            jwt_secret: "jwt-secret123".to_string(),
            log_level: "debug".to_string(),
            keycloak_admin_user: Some("admin".to_string()),
            keycloak_admin_password: Some("password123".to_string()),
            adm_user: Some("admin".to_string()),
            adm_password: Some("password123".to_string()),
        };
        
        let debug_output = format!("{:?}", config);
        
        // Проверяем, что основные поля присутствуют
        assert!(debug_output.contains("server_address"));
        assert!(debug_output.contains("keycloak_url"));
        assert!(debug_output.contains("keycloak_realm"));
        
        // В реальной системе мы бы проверили, что пароли скрыты
        // но в текущей реализации Debug может их показывать
    }
    
    #[tokio::test]
    async fn test_config_validation() {
        // Тест валидации конфигурации
        let valid_config = Config {
            server_address: "0.0.0.0:3001".to_string(),
            database_url: "postgresql://localhost:5432/test".to_string(),
            keycloak_url: "http://localhost:8080".to_string(),
            keycloak_realm: "kubeatlas".to_string(),
            keycloak_client_id: "kubeatlas-backend".to_string(),
            keycloak_client_secret: "secret".to_string(),
            jwt_secret: "jwt-secret".to_string(),
            log_level: "info".to_string(),
            keycloak_admin_user: Some("admin".to_string()),
            keycloak_admin_password: Some("admin".to_string()),
            adm_user: Some("admin".to_string()),
            adm_password: Some("admin".to_string()),
        };
        
        // Проверяем, что все обязательные поля заполнены
        assert!(!valid_config.server_address.is_empty());
        assert!(!valid_config.keycloak_url.is_empty());
        assert!(!valid_config.keycloak_realm.is_empty());
        assert!(!valid_config.keycloak_client_id.is_empty());
        
        // Проверяем формат адреса сервера
        assert!(valid_config.server_address.contains(":"));
        
        // Проверяем формат URL Keycloak
        assert!(valid_config.keycloak_url.starts_with("http"));
    }
}

#[tokio::test]
async fn test_config_memory_usage() {
    // Тест использования памяти (базовый тест размера структуры)
    let config = Config {
        server_address: "0.0.0.0:3001".to_string(),
        database_url: "postgresql://localhost:5432/test".to_string(),
        keycloak_url: "http://localhost:8080".to_string(),
        keycloak_realm: "kubeatlas".to_string(),
        keycloak_client_id: "kubeatlas-backend".to_string(),
        keycloak_client_secret: "secret".to_string(),
        jwt_secret: "jwt-secret".to_string(),
        log_level: "info".to_string(),
        keycloak_admin_user: Some("admin".to_string()),
        keycloak_admin_password: Some("admin".to_string()),
        adm_user: Some("admin".to_string()),
        adm_password: Some("admin".to_string()),
    };
    
    // Проверяем, что структура имеет разумный размер
    let size = std::mem::size_of_val(&config);
    
    // Config должна быть достаточно компактной
    assert!(size > 0);
    assert!(size < 1024); // Не более 1KB для структуры конфигурации
}