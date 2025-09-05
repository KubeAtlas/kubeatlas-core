// Integration tests for KubeAtlas Backend
// 
// Этот файл содержит интеграционные тесты, которые проверяют
// взаимодействие между компонентами системы

mod test_auth;
mod test_config;
mod test_handlers;
mod test_models;
mod test_statistics_handler;

use kubeatlas_backend::{AppState, AuthService, Config};
use std::sync::Once;
use tokio;
use tracing_subscriber;

static INIT: Once = Once::new();

/// Инициализация логгирования для тестов (вызывается один раз)
pub fn init_test_logging() {
    INIT.call_once(|| {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .with_test_writer()
            .init();
    });
}

/// Создает тестовую конфигурацию для интеграционных тестов
pub fn create_integration_test_config() -> Config {
    Config {
        server_address: "0.0.0.0:3001".to_string(),
        database_url: "postgresql://localhost:5432/test".to_string(),
        keycloak_url: "http://localhost:8080".to_string(),
        keycloak_realm: "kubeatlas".to_string(),
        keycloak_client_id: "kubeatlas-backend".to_string(),
        keycloak_client_secret: "integration-test-secret".to_string(),
        jwt_secret: "integration-test-jwt-secret".to_string(),
        log_level: "info".to_string(),
        keycloak_admin_user: Some("admin".to_string()),
        keycloak_admin_password: Some("admin".to_string()),
        adm_user: Some("admin".to_string()),
        adm_password: Some("admin".to_string()),
    }
}

/// Создает тестовый AppState для интеграционных тестов
pub fn create_integration_test_app_state() -> AppState {
    let config = create_integration_test_config();
    let auth_service = AuthService::new(&config).unwrap();
    
    AppState {
        config,
        auth_service,
    }
}

#[tokio::test]
async fn test_integration_basic_setup() {
    // Базовый интеграционный тест настройки системы
    init_test_logging();
    
    let config = create_integration_test_config();
    let auth_service_result = AuthService::new(&config);
    
    // Проверяем, что AuthService создается без ошибок
    assert!(auth_service_result.is_ok());
    
    let app_state = create_integration_test_app_state();
    
    // Проверяем основные компоненты AppState
    assert_eq!(app_state.config.server_address, "0.0.0.0:3001");
    assert_eq!(app_state.config.keycloak_realm, "kubeatlas");
}

#[tokio::test]
async fn test_integration_config_urls() {
    // Интеграционный тест генерации URL конфигурации
    let config = create_integration_test_config();
    
    let jwks_url = config.keycloak_jwks_url();
    let userinfo_url = config.keycloak_userinfo_url();
    let token_url = config.keycloak_token_url();
    let logout_url = config.keycloak_logout_url();
    
    // Проверяем, что все URL генерируются корректно
    assert!(jwks_url.contains("localhost:8080"));
    assert!(jwks_url.contains("kubeatlas"));
    assert!(jwks_url.contains("certs"));
    
    assert!(userinfo_url.contains("userinfo"));
    assert!(token_url.contains("token"));
    assert!(logout_url.contains("logout"));
}

// Интеграционные тесты, которые требуют реального окружения
#[cfg(feature = "integration_tests")]
mod full_integration_tests {
    use super::*;
    use axum_test::TestServer;
    use kubeatlas_backend::{handlers, middleware};
    
    #[tokio::test]
    async fn test_full_server_integration() {
        // Полный интеграционный тест запуска сервера
        init_test_logging();
        
        let app_state = create_integration_test_app_state();
        
        // Создаем полное приложение как в main.rs
        let app = axum::Router::new()
            .route("/health", axum::routing::get(handlers::health_handler::health_check))
            .route("/auth/validate", axum::routing::post(handlers::auth_handler::validate_token))
            .route("/auth/refresh", axum::routing::post(handlers::auth_handler::refresh_token))
            .route("/auth/logout", axum::routing::post(handlers::auth_handler::logout))
            .with_state(app_state);
        
        let server = TestServer::new(app).unwrap();
        
        // Тест health endpoint
        let health_response = server.get("/health").await;
        assert_eq!(health_response.status_code(), axum::http::StatusCode::OK);
        
        // Тест auth endpoints (они должны возвращать ошибки без валидных данных)
        let validate_response = server
            .post("/auth/validate")
            .json(&serde_json::json!({"token": "invalid"}))
            .await;
        
        // Валидация неверного токена должна вернуть ошибку или false
        assert!(validate_response.status_code().is_client_error() || validate_response.status_code().is_success());
    }
    
    #[tokio::test]
    async fn test_statistics_endpoint_integration() {
        // Интеграционный тест statistics endpoint
        init_test_logging();
        
        let app_state = create_integration_test_app_state();
        
        let app = axum::Router::new()
            .route("/api/v1/statistics", axum::routing::get(handlers::statistics_handler::get_statistics))
            .with_state(app_state);
        
        let server = TestServer::new(app).unwrap();
        
        // Тест без аутентификации должен вернуть 401
        let response = server.get("/api/v1/statistics").await;
        assert_eq!(response.status_code(), axum::http::StatusCode::UNAUTHORIZED);
    }
    
    #[tokio::test]
    async fn test_keycloak_connectivity() {
        // Тест подключения к Keycloak (только если он доступен)
        init_test_logging();
        
        let config = create_integration_test_config();
        let auth_service = AuthService::new(&config).unwrap();
        
        // Пытаемся подключиться к Keycloak с таймаутом
        let connectivity_result = auth_service.wait_for_keycloak_ready(5).await;
        
        if connectivity_result.is_ok() {
            println!("✅ Keycloak доступен для интеграционных тестов");
            
            // Если Keycloak доступен, тестируем health check
            let health_result = auth_service.health_check().await;
            assert!(health_result.is_ok());
        } else {
            println!("⚠️ Keycloak недоступен, пропускаем интеграционные тесты");
        }
    }
    
    #[tokio::test]
    async fn test_middleware_integration() {
        // Интеграционный тест middleware
        init_test_logging();
        
        let app_state = create_integration_test_app_state();
        
        let protected_routes = axum::Router::new()
            .route("/api/v1/user/profile", axum::routing::get(handlers::user_handler::get_profile))
            .route("/api/v1/user/roles", axum::routing::get(handlers::user_handler::get_user_roles))
            .layer(axum::middleware::from_fn_with_state(
                app_state.clone(),
                middleware::auth_middleware
            ));
        
        let app = axum::Router::new()
            .route("/health", axum::routing::get(handlers::health_handler::health_check))
            .merge(protected_routes)
            .with_state(app_state);
        
        let server = TestServer::new(app).unwrap();
        
        // Тест незащищенного endpoint
        let health_response = server.get("/health").await;
        assert_eq!(health_response.status_code(), axum::http::StatusCode::OK);
        
        // Тест защищенного endpoint без токена
        let profile_response = server.get("/api/v1/user/profile").await;
        assert_eq!(profile_response.status_code(), axum::http::StatusCode::UNAUTHORIZED);
        
        // Тест защищенного endpoint с неверным токеном
        let profile_with_token_response = server
            .get("/api/v1/user/profile")
            .add_header("Authorization", "Bearer invalid-token")
            .await;
        assert_eq!(profile_with_token_response.status_code(), axum::http::StatusCode::UNAUTHORIZED);
    }
}

// Тесты производительности
#[cfg(feature = "performance_tests")]
mod performance_tests {
    use super::*;
    use std::time::Instant;
    
    #[tokio::test]
    async fn test_config_creation_performance() {
        // Тест производительности создания конфигурации
        let start = Instant::now();
        
        for _ in 0..1000 {
            let _config = create_integration_test_config();
        }
        
        let duration = start.elapsed();
        
        // Создание 1000 конфигураций должно занимать менее 100ms
        assert!(duration.as_millis() < 100);
        println!("Создание 1000 конфигураций заняло: {:?}", duration);
    }
    
    #[tokio::test]
    async fn test_auth_service_creation_performance() {
        // Тест производительности создания AuthService
        let config = create_integration_test_config();
        
        let start = Instant::now();
        
        for _ in 0..100 {
            let _auth_service = AuthService::new(&config).unwrap();
        }
        
        let duration = start.elapsed();
        
        // Создание 100 AuthService должно занимать менее 1 секунды
        assert!(duration.as_secs() < 1);
        println!("Создание 100 AuthService заняло: {:?}", duration);
    }
}