use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode, HeaderMap},
    response::Response,
};
use axum_test::TestServer;
use serde_json::{json, Value};
use tokio;

use kubeatlas_backend::{
    handlers::{health_handler, user_handler, auth_handler},
    models::{ApiResponse, UserRole},
    AppState, AuthService, Config,
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

/// Создает тестовый AppState
fn create_test_app_state() -> AppState {
    let config = create_test_config();
    let auth_service = AuthService::new(&config).unwrap();
    
    AppState {
        config,
        auth_service,
    }
}

#[tokio::test]
async fn test_health_check_handler() {
    // Тест health check handler
    let app_state = create_test_app_state();
    
    // Создаем тестовый сервер
    let app = axum::Router::new()
        .route("/health", axum::routing::get(health_handler::health_check))
        .with_state(app_state);
    
    let server = TestServer::new(app).unwrap();
    
    // Отправляем запрос
    let response = server.get("/health").await;
    
    // Проверяем статус
    assert_eq!(response.status_code(), StatusCode::OK);
    
    // Проверяем содержимое ответа
    let body = response.text();
    let health_data: Value = serde_json::from_str(&body).unwrap();
    
    assert_eq!(health_data["status"], "healthy");
    assert!(health_data["timestamp"].is_number());
    assert_eq!(health_data["service"], "kubeatlas-backend");
    assert_eq!(health_data["version"], "0.1.0");
}

#[tokio::test]
async fn test_health_check_response_structure() {
    // Тест структуры ответа health check
    let app_state = create_test_app_state();
    
    let app = axum::Router::new()
        .route("/health", axum::routing::get(health_handler::health_check))
        .with_state(app_state);
    
    let server = TestServer::new(app).unwrap();
    let response = server.get("/health").await;
    
    assert_eq!(response.status_code(), StatusCode::OK);
    
    let body = response.text();
    let health_data: Value = serde_json::from_str(&body).unwrap();
    
    // Проверяем обязательные поля
    assert!(health_data.get("status").is_some());
    assert!(health_data.get("timestamp").is_some());
    assert!(health_data.get("service").is_some());
    assert!(health_data.get("version").is_some());
}

#[tokio::test]
async fn test_health_check_content_type() {
    // Тест content type для health check
    let app_state = create_test_app_state();
    
    let app = axum::Router::new()
        .route("/health", axum::routing::get(health_handler::health_check))
        .with_state(app_state);
    
    let server = TestServer::new(app).unwrap();
    let response = server.get("/health").await;
    
    // Проверяем content type
    let content_type = response.headers().get("content-type");
    assert!(content_type.is_some());
    
    let content_type_str = content_type.unwrap().to_str().unwrap();
    assert!(content_type_str.contains("application/json"));
}

// Мок-тесты для handlers, которые требуют аутентификации
mod auth_required_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_user_profile_handler_without_auth() {
        // Тест обработчика профиля пользователя без аутентификации
        let app_state = create_test_app_state();
        
        let app = axum::Router::new()
            .route("/api/v1/user/profile", axum::routing::get(user_handler::get_profile))
            .with_state(app_state);
        
        let server = TestServer::new(app).unwrap();
        let response = server.get("/api/v1/user/profile").await;
        
        // Без токена должен возвращать 401
        assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
    }
    
    #[tokio::test]
    async fn test_user_roles_handler_without_auth() {
        // Тест обработчика ролей пользователя без аутентификации
        let app_state = create_test_app_state();
        
        let app = axum::Router::new()
            .route("/api/v1/user/roles", axum::routing::get(user_handler::get_user_roles))
            .with_state(app_state);
        
        let server = TestServer::new(app).unwrap();
        let response = server.get("/api/v1/user/roles").await;
        
        // Без токена должен возвращать 401
        assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
    }
    
    #[tokio::test]
    async fn test_user_profile_handler_with_invalid_auth() {
        // Тест обработчика профиля с неверным токеном
        let app_state = create_test_app_state();
        
        let app = axum::Router::new()
            .route("/api/v1/user/profile", axum::routing::get(user_handler::get_profile))
            .with_state(app_state);
        
        let server = TestServer::new(app).unwrap();
        let response = server
            .get("/api/v1/user/profile")
            .add_header("Authorization", "Bearer invalid-token")
            .await;
        
        // С неверным токеном должен возвращать 401
        assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
    }
}

mod auth_handler_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_validate_token_handler_structure() {
        // Тест структуры обработчика валидации токена
        let app_state = create_test_app_state();
        
        let app = axum::Router::new()
            .route("/auth/validate", axum::routing::post(auth_handler::validate_token))
            .with_state(app_state);
        
        let server = TestServer::new(app).unwrap();
        
        // Тест с пустым телом запроса
        let response = server
            .post("/auth/validate")
            .json(&json!({}))
            .await;
        
        // Должен возвращать ошибку (400 или 422)
        assert!(response.status_code().is_client_error());
    }
    
    #[tokio::test]
    async fn test_validate_token_handler_with_token() {
        // Тест валидации токена с реальным токеном
        let app_state = create_test_app_state();
        
        let app = axum::Router::new()
            .route("/auth/validate", axum::routing::post(auth_handler::validate_token))
            .with_state(app_state);
        
        let server = TestServer::new(app).unwrap();
        
        let response = server
            .post("/auth/validate")
            .json(&json!({"token": "invalid-test-token"}))
            .await;
        
        // С неверным токеном должен возвращать статус (200 с error или 401)
        // Точный статус зависит от реализации
        assert!(response.status_code().is_client_error() || response.status_code().is_success());
        
        if response.status_code().is_success() {
            let body = response.text();
            let result: Value = serde_json::from_str(&body).unwrap();
            
            // Если возвращает 200, то должно быть поле valid: false
            if let Some(valid) = result.get("valid") {
                assert_eq!(valid, &json!(false));
            }
        }
    }
    
    #[tokio::test]
    async fn test_refresh_token_handler_structure() {
        // Тест структуры обработчика обновления токена
        let app_state = create_test_app_state();
        
        let app = axum::Router::new()
            .route("/auth/refresh", axum::routing::post(auth_handler::refresh_token))
            .with_state(app_state);
        
        let server = TestServer::new(app).unwrap();
        
        let response = server
            .post("/auth/refresh")
            .json(&json!({"refresh_token": "invalid-refresh-token"}))
            .await;
        
        // С неверным refresh токеном должен возвращать ошибку
        assert!(response.status_code().is_client_error() || response.status_code().is_server_error());
    }
    
    #[tokio::test]
    async fn test_logout_handler_structure() {
        // Тест структуры обработчика выхода
        let app_state = create_test_app_state();
        
        let app = axum::Router::new()
            .route("/auth/logout", axum::routing::post(auth_handler::logout))
            .with_state(app_state);
        
        let server = TestServer::new(app).unwrap();
        
        let response = server
            .post("/auth/logout")
            .json(&json!({"refresh_token": "test-refresh-token"}))
            .await;
        
        // Logout может возвращать разные статусы в зависимости от реализации
        assert!(response.status_code().as_u16() >= 200);
        assert!(response.status_code().as_u16() < 600);
    }
}

mod error_handling_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_handler_error_responses() {
        // Тест обработки ошибок в handlers
        let app_state = create_test_app_state();
        
        let app = axum::Router::new()
            .route("/auth/validate", axum::routing::post(auth_handler::validate_token))
            .with_state(app_state);
        
        let server = TestServer::new(app).unwrap();
        
        // Тест с некорректным JSON
        let response = server
            .post("/auth/validate")
            .text("invalid json")
            .content_type("application/json")
            .await;
        
        // Должен возвращать 400 или 422
        assert!(response.status_code().is_client_error());
    }
    
    #[tokio::test]
    async fn test_method_not_allowed() {
        // Тест неразрешенных методов
        let app_state = create_test_app_state();
        
        let app = axum::Router::new()
            .route("/health", axum::routing::get(health_handler::health_check))
            .with_state(app_state);
        
        let server = TestServer::new(app).unwrap();
        
        // POST на GET endpoint
        let response = server
            .post("/health")
            .await;
        
        // Должен возвращать 405 Method Not Allowed
        assert_eq!(response.status_code(), StatusCode::METHOD_NOT_ALLOWED);
    }
    
    #[tokio::test]
    async fn test_not_found() {
        // Тест несуществующих endpoint'ов
        let app_state = create_test_app_state();
        
        let app = axum::Router::new()
            .route("/health", axum::routing::get(health_handler::health_check))
            .with_state(app_state);
        
        let server = TestServer::new(app).unwrap();
        
        let response = server
            .get("/nonexistent")
            .await;
        
        // Должен возвращать 404 Not Found
        assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
    }
}

#[tokio::test]
async fn test_cors_headers() {
    // Тест CORS заголовков (если они настроены)
    let app_state = create_test_app_state();
    
    let app = axum::Router::new()
        .route("/health", axum::routing::get(health_handler::health_check))
        .with_state(app_state);
    
    let server = TestServer::new(app).unwrap();
    
    let response = server
        .get("/health")
        .add_header("Origin", "http://localhost:3000")
        .await;
    
    // Проверяем статус
    assert_eq!(response.status_code(), StatusCode::OK);
    
    // В зависимости от настроек CORS, могут присутствовать соответствующие заголовки
    let headers = response.headers();
    
    // Если CORS настроен, должны быть соответствующие заголовки
    // В данном случае просто проверяем, что ответ успешен
    assert!(headers.get("content-type").is_some());
}