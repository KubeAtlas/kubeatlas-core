use axum::{
    middleware::from_fn_with_state,
    routing::{get, post, put},
    Router,
};
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::{info, Level};
use tracing_subscriber;

mod auth;
mod config;
mod handlers;
mod middleware;
mod models;

use auth::AuthService;
use config::Config;
use handlers::{auth_handler, health_handler, user_handler, user_admin_handler};
use crate::middleware::{auth_middleware, require_admin_middleware};

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub auth_service: AuthService,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing with more verbose output
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .init();

    info!("🚀 Starting KubeAtlas Backend...");

    // Load configuration
    let config = match Config::load() {
        Ok(config) => {
            info!("✅ Configuration loaded");
            config
        }
        Err(e) => {
            eprintln!("❌ Failed to load configuration: {}", e);
            return Err(e.into());
        }
    };

    // Initialize auth service
    let auth_service = match AuthService::new(&config) {
        Ok(auth_service) => {
            info!("✅ Auth service initialized");
            auth_service
        }
        Err(e) => {
            eprintln!("❌ Failed to initialize auth service: {}", e);
            return Err(e.into());
        }
    };

    // Wait for Keycloak readiness
    if let Err(e) = auth_service.wait_for_keycloak_ready(120).await {
        eprintln!("⚠️ Keycloak not ready: {}", e);
    }

    // Ensure admin user if configured
    if let Err(e) = auth_service.ensure_admin_user().await {
        eprintln!("⚠️ Failed to ensure admin user: {}", e);
    }
    // Ensure admin-service has realm role admin if master creds provided
    if let Err(e) = auth_service.ensure_realm_admin_role().await {
        eprintln!("⚠️ Failed to ensure realm admin role: {}", e);
    }

    // Create app state
    let app_state = AppState {
        config: config.clone(),
        auth_service,
    };

    // Protected user routes
    let protected = Router::new()
        .route("/api/v1/user/profile", get(user_handler::get_profile))
        .route("/api/v1/user/roles", get(user_handler::get_user_roles))
        .layer(from_fn_with_state(app_state.clone(), auth_middleware));

    // Admin routes (RBAC only for this subrouter)
    let admin = Router::new()
        .route("/api/v1/admin/users", post(user_admin_handler::create_user))
        .route("/api/v1/admin/users/:id", put(user_admin_handler::update_user))
        // Порядок важен: внешний слой выполняется первым, поэтому сначала auth, потом require_admin
        .route_layer(from_fn_with_state(app_state.clone(), require_admin_middleware))
        .route_layer(from_fn_with_state(app_state.clone(), auth_middleware));

    // Build the application router
    let app = Router::new()
        // Health check (no auth required)
        .route("/health", get(health_handler::health_check))
        
        // Auth routes (no auth required)
        .route("/auth/validate", post(auth_handler::validate_token))
        .route("/auth/user", get(auth_handler::get_user_info))
        .route("/auth/refresh", post(auth_handler::refresh_token))
        
        // Merge subrouters
        .merge(protected)
        .merge(admin)
        
        // Add CORS and tracing middleware
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(
                    CorsLayer::new()
                        .allow_origin(Any)
                        .allow_methods(Any)
                        .allow_headers(Any),
                ),
        )
        .with_state(app_state.clone());

    // Start the server
    let listener = match tokio::net::TcpListener::bind(&config.server_address).await {
        Ok(listener) => {
            info!("🌐 Server listening on {}", config.server_address);
            listener
        }
        Err(e) => {
            eprintln!("❌ Failed to bind to {}: {}", config.server_address, e);
            return Err(e.into());
        }
    };

    info!("🚀 Starting server...");
    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("❌ Server error: {}", e);
        return Err(e.into());
    }

    info!("✅ Server stopped gracefully");
    Ok(())
}
