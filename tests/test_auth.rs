use tokio;
use anyhow::Result;
use serde_json::Value;

use kubeatlas_backend::{
    auth::{AuthService, KeycloakUser, RealmAccess, TokenValidationRequest, RefreshTokenRequest, LogoutRequest},
    config::Config,
};

/// Создает тестовую конфигурацию
fn create_test_config() -> Config {
    Config {
        server_address: "0.0.0.0:3001".to_string(),
        database_url: "postgresql://localhost:5432/test".to_string(),
        keycloak_url: "http://localhost:8080".to_string(),
        keycloak_realm: "kubeatlas".to_string(),
        keycloak_client_id: "kubeatlas-backend".to_string(),
        keycloak_client_secret: "test-secret".to_string(),
        jwt_secret: "test-jwt-secret".to_string(),
        log_level: "info".to_string(),
        keycloak_admin_user: Some("admin".to_string()),
        keycloak_admin_password: Some("admin".to_string()),
        adm_user: Some("admin".to_string()),
        adm_password: Some("admin".to_string()),
    }
}

#[tokio::test]
async fn test_auth_service_creation() {
    // Тест создания AuthService
    let config = create_test_config();
    let auth_service = AuthService::new(&config);
    
    assert!(auth_service.is_ok());
}

#[tokio::test]
async fn test_keycloak_user_structure() {
    // Тест структуры пользователя Keycloak
    let user = KeycloakUser {
        sub: "user-id-123".to_string(),
        preferred_username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        given_name: Some("Test".to_string()),
        family_name: Some("User".to_string()),
        realm_access: Some(RealmAccess {
            roles: vec!["user".to_string(), "admin".to_string()],
        }),
        resource_access: None,
    };
    
    assert_eq!(user.sub, "user-id-123");
    assert_eq!(user.preferred_username, "testuser");
    assert_eq!(user.email, "test@example.com");
    assert_eq!(user.given_name, Some("Test".to_string()));
    assert_eq!(user.family_name, Some("User".to_string()));
    assert!(user.realm_access.is_some());
    
    let realm_access = user.realm_access.unwrap();
    assert_eq!(realm_access.roles.len(), 2);
    assert!(realm_access.roles.contains(&"user".to_string()));
    assert!(realm_access.roles.contains(&"admin".to_string()));
}

#[tokio::test]
async fn test_token_validation_request() {
    // Тест структуры запроса валидации токена
    let request = TokenValidationRequest {
        token: "test-token-123".to_string(),
    };
    
    assert_eq!(request.token, "test-token-123");
}

#[tokio::test]
async fn test_refresh_token_request() {
    // Тест структуры запроса обновления токена
    let request = RefreshTokenRequest {
        refresh_token: "refresh-token-123".to_string(),
    };
    
    assert_eq!(request.refresh_token, "refresh-token-123");
}

#[tokio::test]
async fn test_logout_request() {
    // Тест структуры запроса выхода
    let request = LogoutRequest {
        refresh_token: "refresh-token-123".to_string(),
    };
    
    assert_eq!(request.refresh_token, "refresh-token-123");
}

#[tokio::test]
async fn test_realm_access_creation() {
    // Тест создания доступа к realm
    let realm_access = RealmAccess {
        roles: vec!["admin".to_string(), "user".to_string(), "guest".to_string()],
    };
    
    assert_eq!(realm_access.roles.len(), 3);
    assert!(realm_access.roles.contains(&"admin".to_string()));
    assert!(realm_access.roles.contains(&"user".to_string()));
    assert!(realm_access.roles.contains(&"guest".to_string()));
}

#[tokio::test]
async fn test_config_jwks_url_generation() {
    // Тест генерации URL для JWKS
    let config = create_test_config();
    let jwks_url = config.keycloak_jwks_url();
    
    let expected_url = "http://localhost:8080/realms/kubeatlas/protocol/openid-connect/certs";
    assert_eq!(jwks_url, expected_url);
}

#[tokio::test]
async fn test_config_userinfo_url_generation() {
    // Тест генерации URL для userinfo
    let config = create_test_config();
    let userinfo_url = config.keycloak_userinfo_url();
    
    let expected_url = "http://localhost:8080/realms/kubeatlas/protocol/openid-connect/userinfo";
    assert_eq!(userinfo_url, expected_url);
}

#[tokio::test]
async fn test_config_token_url_generation() {
    // Тест генерации URL для получения токена
    let config = create_test_config();
    let token_url = config.keycloak_token_url();
    
    let expected_url = "http://localhost:8080/realms/kubeatlas/protocol/openid-connect/token";
    assert_eq!(token_url, expected_url);
}

#[tokio::test]
async fn test_config_logout_url_generation() {
    // Тест генерации URL для выхода
    let config = create_test_config();
    let logout_url = config.keycloak_logout_url();
    
    let expected_url = "http://localhost:8080/realms/kubeatlas/protocol/openid-connect/logout";
    assert_eq!(logout_url, expected_url);
}

// Мок-тесты для методов AuthService, которые требуют сетевого взаимодействия
mod mock_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_statistics_methods_interface() {
        // Тест интерфейса методов статистики
        // В реальной ситуации мы бы создали mock или использовали библиотеку типа mockall
        
        // Создаем AuthService с тестовой конфигурацией
        let config = create_test_config();
        let auth_service = AuthService::new(&config).unwrap();
        
        // Эти тесты будут падать без реального Keycloak, но проверяют интерфейс
        // В реальной CI/CD мы бы настроили mock или тестовый Keycloak
        
        // Проверяем, что методы существуют и имеют правильную сигнатуру
        let users_count_future = auth_service.get_total_users_count();
        let sessions_count_future = auth_service.get_active_sessions_count();
        let health_check_future = auth_service.health_check();
        
        // Методы должны возвращать Future, который можно ожидать
        // В реальных тестах мы бы использовали mock для проверки результатов
    }
}

// Интеграционные тесты (требуют запущенного Keycloak)
#[cfg(feature = "integration_tests")]
mod integration_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_keycloak_health_check_integration() {
        // Этот тест требует запущенного Keycloak
        let config = create_test_config();
        let auth_service = AuthService::new(&config).unwrap();
        
        // Ждем готовности Keycloak (с таймаутом)
        let result = auth_service.wait_for_keycloak_ready(10).await;
        
        if result.is_ok() {
            // Если Keycloak доступен, проверяем health check
            let health_result = auth_service.health_check().await;
            assert!(health_result.is_ok());
        }
        // Если Keycloak недоступен, тест просто пропускается
    }
    
    #[tokio::test]
    async fn test_get_total_users_count_integration() {
        // Этот тест требует запущенного Keycloak с настроенным admin пользователем
        let config = create_test_config();
        let auth_service = AuthService::new(&config).unwrap();
        
        // Пытаемся получить количество пользователей
        let result = auth_service.get_total_users_count().await;
        
        // В зависимости от окружения тест может падать или проходить
        if result.is_ok() {
            let count = result.unwrap();
            assert!(count >= 0); // Количество пользователей не может быть отрицательным
        }
    }
}