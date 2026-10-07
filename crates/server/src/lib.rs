pub mod routes;

use std::net::SocketAddr;
use forgex_core::ForgeConfig;
use forgex_storage::ForgeDb;

pub use routes::{create_router, AppState};

pub async fn start_server(
    db: ForgeDb,
    config: ForgeConfig,
    pairing_token: String,
    host: &str,
    port: u16,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let state = AppState {
        db,
        config,
        pairing_token,
    };

    let router = create_router(state);
    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;

    println!("ForgeX Sync Daemon listening on http://{}", addr);
    axum::serve(listener, router).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_server_health_check() {
        let db = ForgeDb::open_in_memory().unwrap();
        let state = AppState {
            db,
            config: ForgeConfig::default(),
            pairing_token: "test_token_123".into(),
        };

        let app = create_router(state);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/health")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::OK);
    }
}
