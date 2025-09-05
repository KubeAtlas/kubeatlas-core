use serde_json;
use tokio;

use kubeatlas_backend::models::{
    ApiResponse, StatisticsResponse, StatItem, SystemStatus, ServiceStatus,
    CreateUserRequest, UpdateUserRequest, UserRole,
};

#[tokio::test]
async fn test_api_response_serialization() {
    // Тест сериализации ApiResponse
    let response = ApiResponse::success("test data");
    let json = serde_json::to_string(&response).unwrap();
    
    assert!(json.contains("\"success\":true"));
    assert!(json.contains("\"data\":\"test data\""));
    assert!(json.contains("\"error\":null"));
}

#[tokio::test]
async fn test_api_response_deserialization() {
    // Тест десериализации ApiResponse
    let json = r#"{"success":true,"data":"test","error":null,"message":null}"#;
    let response: ApiResponse<String> = serde_json::from_str(json).unwrap();
    
    assert!(response.success);
    assert_eq!(response.data, Some("test".to_string()));
    assert!(response.error.is_none());
}

#[tokio::test]
async fn test_stat_item_serialization() {
    // Тест сериализации StatItem
    let stat_item = StatItem {
        value: 1234,
        change_percent: 12.5,
        change_period: "за месяц".to_string(),
    };
    
    let json = serde_json::to_string(&stat_item).unwrap();
    
    assert!(json.contains("\"value\":1234"));
    assert!(json.contains("\"change_percent\":12.5"));
    assert!(json.contains("\"change_period\":\"за месяц\""));
}

#[tokio::test]
async fn test_stat_item_deserialization() {
    // Тест десериализации StatItem
    let json = r#"{"value":1234,"change_percent":12.5,"change_period":"за месяц"}"#;
    let stat_item: StatItem = serde_json::from_str(json).unwrap();
    
    assert_eq!(stat_item.value, 1234);
    assert_eq!(stat_item.change_percent, 12.5);
    assert_eq!(stat_item.change_period, "за месяц");
}

#[tokio::test]
async fn test_service_status_serialization() {
    // Тест сериализации ServiceStatus
    let service_status = ServiceStatus {
        name: "Keycloak".to_string(),
        status: "operational".to_string(),
        uptime_percentage: 99.9,
    };
    
    let json = serde_json::to_string(&service_status).unwrap();
    
    assert!(json.contains("\"name\":\"Keycloak\""));
    assert!(json.contains("\"status\":\"operational\""));
    assert!(json.contains("\"uptime_percentage\":99.9"));
}

#[tokio::test]
async fn test_system_status_serialization() {
    // Тест сериализации SystemStatus
    let system_status = SystemStatus {
        percentage: 98.5,
        status: "Все системы работают".to_string(),
        details: vec![
            ServiceStatus {
                name: "API".to_string(),
                status: "operational".to_string(),
                uptime_percentage: 99.0,
            },
        ],
    };
    
    let json = serde_json::to_string(&system_status).unwrap();
    
    assert!(json.contains("\"percentage\":98.5"));
    assert!(json.contains("\"status\":\"Все системы работают\""));
    assert!(json.contains("\"details\":["));
}

#[tokio::test]
async fn test_statistics_response_full_serialization() {
    // Тест полной сериализации StatisticsResponse
    let stats_response = StatisticsResponse {
        total_users: StatItem {
            value: 1234,
            change_percent: 12.0,
            change_period: "с прошлого месяца".to_string(),
        },
        active_sessions: StatItem {
            value: 89,
            change_percent: 5.0,
            change_period: "с прошлого часа".to_string(),
        },
        system_status: SystemStatus {
            percentage: 98.5,
            status: "Все системы работают".to_string(),
            details: vec![
                ServiceStatus {
                    name: "Keycloak".to_string(),
                    status: "operational".to_string(),
                    uptime_percentage: 99.9,
                },
                ServiceStatus {
                    name: "Database".to_string(),
                    status: "operational".to_string(),
                    uptime_percentage: 99.5,
                },
            ],
        },
    };
    
    let json = serde_json::to_string(&stats_response).unwrap();
    
    // Проверяем основные поля
    assert!(json.contains("\"total_users\""));
    assert!(json.contains("\"active_sessions\""));
    assert!(json.contains("\"system_status\""));
    
    // Проверяем вложенные значения
    assert!(json.contains("1234"));
    assert!(json.contains("89"));
    assert!(json.contains("98.5"));
}

#[tokio::test]
async fn test_statistics_response_deserialization() {
    // Тест десериализации StatisticsResponse
    let json = r#"{
        "total_users": {
            "value": 1234,
            "change_percent": 12.0,
            "change_period": "с прошлого месяца"
        },
        "active_sessions": {
            "value": 89,
            "change_percent": 5.0,
            "change_period": "с прошлого часа"
        },
        "system_status": {
            "percentage": 98.5,
            "status": "Все системы работают",
            "details": [
                {
                    "name": "Keycloak",
                    "status": "operational",
                    "uptime_percentage": 99.9
                }
            ]
        }
    }"#;
    
    let stats_response: StatisticsResponse = serde_json::from_str(json).unwrap();
    
    assert_eq!(stats_response.total_users.value, 1234);
    assert_eq!(stats_response.active_sessions.value, 89);
    assert_eq!(stats_response.system_status.percentage, 98.5);
    assert_eq!(stats_response.system_status.details.len(), 1);
}

#[tokio::test]
async fn test_create_user_request_serialization() {
    // Тест сериализации CreateUserRequest
    let create_request = CreateUserRequest {
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        first_name: Some("Test".to_string()),
        last_name: Some("User".to_string()),
        password: "password123".to_string(),
        roles: vec!["user".to_string(), "admin".to_string()],
    };
    
    let json = serde_json::to_string(&create_request).unwrap();
    
    assert!(json.contains("\"username\":\"testuser\""));
    assert!(json.contains("\"email\":\"test@example.com\""));
    assert!(json.contains("\"first_name\":\"Test\""));
    assert!(json.contains("\"last_name\":\"User\""));
    assert!(json.contains("\"password\":\"password123\""));
    assert!(json.contains("\"roles\":[\"user\",\"admin\"]"));
}

#[tokio::test]
async fn test_update_user_request_serialization() {
    // Тест сериализации UpdateUserRequest
    let update_request = UpdateUserRequest {
        email: Some("newemail@example.com".to_string()),
        first_name: Some("NewName".to_string()),
        last_name: Some("NewLastName".to_string()),
        roles: Some(vec!["admin".to_string()]),
    };
    
    let json = serde_json::to_string(&update_request).unwrap();
    
    assert!(json.contains("\"email\":\"newemail@example.com\""));
    assert!(json.contains("\"roles\":[\"admin\"]"));
}

#[tokio::test]
async fn test_user_role_serialization() {
    // Тест сериализации UserRole
    let user_role = UserRole {
        username: "testuser".to_string(),
        roles: vec!["user".to_string(), "admin".to_string()],
        is_admin: true,
        is_user: true,
        is_guest: false,
    };
    
    let json = serde_json::to_string(&user_role).unwrap();
    
    assert!(json.contains("\"username\":\"testuser\""));
    assert!(json.contains("\"roles\":[\"user\",\"admin\"]"));
    assert!(json.contains("\"is_admin\":true"));
    assert!(json.contains("\"is_user\":true"));
    assert!(json.contains("\"is_guest\":false"));
}

#[tokio::test]
async fn test_api_response_error_creation() {
    // Тест создания ошибки API
    let error_response: ApiResponse<String> = ApiResponse::error("Test error message".to_string());
    
    assert!(!error_response.success);
    assert!(error_response.data.is_none());
    assert_eq!(error_response.error, Some("Test error message".to_string()));
    assert!(error_response.message.is_none());
}

#[tokio::test]
async fn test_api_response_message_creation() {
    // Тест создания сообщения API
    let message_response: ApiResponse<String> = ApiResponse::message("Operation completed".to_string());
    
    assert!(message_response.success);
    assert!(message_response.data.is_none());
    assert!(message_response.error.is_none());
    assert_eq!(message_response.message, Some("Operation completed".to_string()));
}

#[tokio::test]
async fn test_edge_cases() {
    // Тест крайних случаев
    
    // Пустые строки
    let empty_stat = StatItem {
        value: 0,
        change_percent: 0.0,
        change_period: "".to_string(),
    };
    let json = serde_json::to_string(&empty_stat).unwrap();
    assert!(json.contains("\"value\":0"));
    assert!(json.contains("\"change_percent\":0.0"));
    
    // Отрицательные проценты
    let negative_stat = StatItem {
        value: 100,
        change_percent: -5.5,
        change_period: "снижение".to_string(),
    };
    let json = serde_json::to_string(&negative_stat).unwrap();
    assert!(json.contains("\"change_percent\":-5.5"));
    
    // Очень большие числа
    let large_stat = StatItem {
        value: u64::MAX,
        change_percent: f64::MAX,
        change_period: "максимум".to_string(),
    };
    let json = serde_json::to_string(&large_stat).unwrap();
    assert!(json.contains(&format!("\"value\":{}", u64::MAX)));
}