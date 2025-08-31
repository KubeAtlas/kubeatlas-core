use axum::{
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn health_check() -> Result<Json<Value>, StatusCode> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let health_data = json!({
        "status": "healthy",
        "timestamp": timestamp,
        "service": "kubeatlas-backend",
        "version": env!("CARGO_PKG_VERSION")
    });

    Ok(Json(health_data))
}
