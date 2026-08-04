use std::sync::Arc;

use axum::{routing::get, Router};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::{
    config::Config,
    handlers::{handle_get_wallet_configs, handle_health},
    wallet_config::WalletConfigService,
};

#[derive(Clone)]
pub struct AppState {
    pub wallet_config_service: Arc<WalletConfigService>,
}

pub fn build_router(config: &Config, wallet_config_service: Arc<WalletConfigService>) -> Router {
    let state = AppState {
        wallet_config_service,
    };

    let origins = config.get_cors_allowed_origins();
    let cors = if origins.is_empty() {
        CorsLayer::new()
    } else {
        CorsLayer::new().allow_origin(origins)
    };

    Router::new()
        .route("/health", get(handle_health))
        .route("/api/configs/wallet", get(handle_get_wallet_configs))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt;
    use uuid::Uuid;

    const VALID_FLAGS: &str = r#"{
  "enableTestButtons": false,
  "enableKeystoneHardwareWallet": false,
  "enableHighSecurity": true,
  "enableRemoteNotifications": true,
  "enableSwap": true
}"#;

    fn test_config() -> Config {
        Config {
            server: crate::config::ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 0,
                cors_allowed_origins: vec![],
            },
            logging: crate::config::LoggingConfig {
                level: "info".to_string(),
            },
            remote_configs: crate::config::RemoteConfigsConfig {
                wallet_configs_file: String::new(),
            },
        }
    }

    async fn setup_app() -> (Router, std::path::PathBuf) {
        let dir = std::env::temp_dir().join(format!("remote-config-http-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let config_path = dir.join("flags.json");
        std::fs::write(&config_path, VALID_FLAGS).unwrap();

        let service = Arc::new(WalletConfigService::new(config_path).expect("service"));
        let app = build_router(&test_config(), service);
        (app, dir)
    }

    #[tokio::test]
    async fn get_wallet_configs_returns_data_envelope() {
        let (app, dir) = setup_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/configs/wallet")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["data"]["enableSwap"], true);
        assert_eq!(json["data"]["enableTestButtons"], false);

        std::fs::remove_dir_all(dir).ok();
    }

    #[tokio::test]
    async fn health_returns_ok_when_config_loaded() {
        let (app, dir) = setup_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        std::fs::remove_dir_all(dir).ok();
    }
}
