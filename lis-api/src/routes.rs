//! Route configuration for the LIS API

use axum::{
    routing::{get, post},
    Router,
};

use crate::handlers;

/// Create the main application router
pub fn create_router() -> Router {
    Router::new()
        // Compilation endpoints
        .route("/api/compile", post(handlers::compile_handler))
        .route("/api/execute", post(handlers::execute_handler))

        // Code quality endpoints
        .route("/api/format", post(handlers::format_handler))
        .route("/api/check", post(handlers::check_handler))

        // Information endpoints
        .route("/api/intrinsics", get(handlers::intrinsics_handler))
        .route("/api/info", get(handlers::info_handler))

        // Health check
        .route("/health", get(handlers::health_handler))
}
