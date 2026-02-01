//! Server setup and configuration.
//!
//! Provides the main server struct for running the API.

use std::net::SocketAddr;

use axum::Router;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::routes::create_router;
use crate::state::AppState;

/// API server configuration.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Host to bind to.
    pub host: String,
    /// Port to listen on.
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
        }
    }
}

impl ServerConfig {
    /// Creates a new server configuration.
    #[must_use]
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
        }
    }

    /// Returns the socket address.
    #[must_use]
    pub fn socket_addr(&self) -> SocketAddr {
        format!("{}:{}", self.host, self.port)
            .parse()
            .expect("Invalid socket address")
    }
}

/// API server.
pub struct Server {
    /// Server configuration.
    config: ServerConfig,
    /// Application state.
    state: AppState,
}

impl Server {
    /// Creates a new server with the given configuration and state.
    #[must_use]
    pub fn new(config: ServerConfig, state: AppState) -> Self {
        Self { config, state }
    }

    /// Creates a new server with default configuration.
    #[must_use]
    pub fn with_default_config(state: AppState) -> Self {
        Self::new(ServerConfig::default(), state)
    }

    /// Returns the server configuration.
    #[must_use]
    pub const fn config(&self) -> &ServerConfig {
        &self.config
    }

    /// Creates the router with middleware.
    pub fn router(&self) -> Router {
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        create_router(self.state.clone())
            .layer(TraceLayer::new_for_http())
            .layer(cors)
    }

    /// Runs the server.
    ///
    /// # Errors
    ///
    /// Returns an error if the server fails to start.
    pub async fn run(self) -> Result<(), std::io::Error> {
        let addr = self.config.socket_addr();
        let listener = TcpListener::bind(addr).await?;

        info!("API server listening on {}", addr);

        axum::serve(listener, self.router()).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config_default() {
        let config = ServerConfig::default();
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 8080);
    }

    #[test]
    fn test_server_config_new() {
        let config = ServerConfig::new("127.0.0.1", 3000);
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 3000);
    }

    #[test]
    fn test_server_config_socket_addr() {
        let config = ServerConfig::new("127.0.0.1", 8080);
        let addr = config.socket_addr();
        assert_eq!(addr.to_string(), "127.0.0.1:8080");
    }

    #[test]
    fn test_server_new() {
        let config = ServerConfig::default();
        let state = AppState::default();
        let server = Server::new(config.clone(), state);
        assert_eq!(server.config().port, config.port);
    }

    #[test]
    fn test_server_router() {
        let state = AppState::default();
        let server = Server::with_default_config(state);
        let _router = server.router();
    }
}
