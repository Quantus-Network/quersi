use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

use crate::services::{exchange_rate::ExchangeRateError, wallet_config::WalletConfigsError};

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
    #[error(transparent)]
    ExchangeRate(#[from] ExchangeRateError),
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
            AppError::ExchangeRate(err) => map_exchange_rate_error(err),
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}

fn map_exchange_rate_error(err: &ExchangeRateError) -> (StatusCode, String) {
    match err {
        ExchangeRateError::Api(detail) => {
            tracing::error!("Exchange rate API error: {detail}");
            (
                StatusCode::BAD_GATEWAY,
                "Failed to fetch exchange rates".to_string(),
            )
        }
        ExchangeRateError::Http(e) => {
            tracing::error!("Exchange rate HTTP error: {e}");
            (
                StatusCode::BAD_GATEWAY,
                "Failed to fetch exchange rates".to_string(),
            )
        }
        ExchangeRateError::Json(e) => {
            tracing::error!("Exchange rate JSON parse error: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to parse exchange rate response".to_string(),
            )
        }
        ExchangeRateError::Cache(detail) => {
            tracing::error!("Exchange rate cache error: {detail}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An internal server error occurred".to_string(),
            )
        }
    }
}
