use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

use crate::wallet_config::WalletConfigsError;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("configuration error: {0}")]
    Config(#[from] config::ConfigError),
    #[error("config path has no parent directory")]
    MissingConfigParent,
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    WalletConfigs(#[from] WalletConfigsError),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::WalletConfigs(WalletConfigsError::ReadLock(_)) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to read wallet configs".to_string(),
            ),
            AppError::WalletConfigs(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "wallet configs unavailable".to_string(),
            ),
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}
