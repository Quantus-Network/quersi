use axum::{extract::State, response::IntoResponse, Json};
use serde::Serialize;
use serde_json::Value;

use crate::{error::AppError, AppState};

#[derive(Debug, Serialize)]
pub struct SuccessResponse<T> {
    pub data: T,
}

impl<T> SuccessResponse<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }
}

pub async fn handle_get_wallet_configs(
    State(state): State<AppState>,
) -> Result<Json<SuccessResponse<Value>>, AppError> {
    let flags = state.wallet_config_service.get_wallet_configs()?;
    Ok(Json(SuccessResponse::new(flags)))
}

pub async fn handle_health() -> impl IntoResponse {
    Json(serde_json::json!({
        "healthy": true,
        "service": "RemoteConfig".to_string(),
        "version": env!("CARGO_PKG_VERSION").to_string(),
    }))
}
