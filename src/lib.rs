#![recursion_limit = "256"]

// KubeAtlas Backend Library
// Экспортируем публичные модули для использования в тестах

pub mod auth;
pub mod config;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod services;

// Экспортируем основные типы
pub use auth::AuthService;
pub use config::Config;
pub use services::{TokenService, InstallService};

// Основная структура состояния приложения
#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub auth_service: AuthService,
    pub install_service: InstallService,
}