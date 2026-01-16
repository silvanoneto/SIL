//! LIS REST API Server
//!
//! Exposes LIS compilation, execution, and formatting via HTTP endpoints.
//!
//! ## Features
//! - API Key Authentication (optional, via X-API-Key header)
//! - Rate Limiting (configurable requests per second)
//! - OpenAPI/Swagger documentation at /docs
//!
//! ## Environment Variables
//! - `LIS_API_HOST`: Host to bind to (default: 127.0.0.1)
//! - `LIS_API_PORT`: Port to listen on (default: 3000)
//! - `LIS_API_KEYS`: Comma-separated list of valid API keys (empty = no auth)
//! - `LIS_API_RATE_LIMIT`: Requests per second (default: 10)
//! - `LIS_API_RATE_BURST`: Burst size (default: 20)
//! - `LIS_API_RATE_ENABLED`: Enable rate limiting (default: true)

pub mod handlers;
pub mod middleware;
pub mod models;
mod openapi;
mod routes;

use axum::middleware as axum_middleware;
use clap::Parser;
use std::{net::SocketAddr, sync::Arc};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use middleware::{
    auth_middleware, rate_limit_middleware, ApiKeyConfig, RateLimitConfig,
};
use openapi::ApiDoc;

#[derive(Parser)]
#[command(name = "lis-api")]
#[command(author = "SIL Contributors")]
#[command(version = "2026.1.16")]
#[command(about = "REST API server for LIS language", long_about = None)]
struct Args {
    /// Host to bind to
    #[arg(short = 'H', long, default_value = "127.0.0.1", env = "LIS_API_HOST")]
    host: String,

    /// Port to listen on
    #[arg(short, long, default_value_t = 3000, env = "LIS_API_PORT")]
    port: u16,

    /// Enable CORS for all origins
    #[arg(long, default_value_t = false)]
    cors: bool,

    /// Disable rate limiting
    #[arg(long, default_value_t = false)]
    no_rate_limit: bool,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "lis_api=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();

    // Load configurations from environment
    let auth_config = Arc::new(ApiKeyConfig::from_env());
    let mut rate_config = RateLimitConfig::from_env();

    if args.no_rate_limit {
        rate_config.enabled = false;
    }

    let rate_limiter = rate_config.create_limiter();

    // Build router with API routes
    let mut app = routes::create_router();

    // Add Swagger UI
    app = app.merge(
        SwaggerUi::new("/docs")
            .url("/api-docs/openapi.json", ApiDoc::openapi()),
    );

    // Add rate limiting middleware
    app = app.layer(axum_middleware::from_fn_with_state(
        (rate_limiter.clone(), rate_config.clone()),
        rate_limit_middleware,
    ));

    // Add authentication middleware
    app = app.layer(axum_middleware::from_fn_with_state(
        auth_config.clone(),
        auth_middleware,
    ));

    // Add CORS if enabled
    if args.cors {
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);
        app = app.layer(cors);
    }

    // Add tracing layer
    app = app.layer(TraceLayer::new_for_http());

    // Parse address
    let addr: SocketAddr = format!("{}:{}", args.host, args.port)
        .parse()
        .expect("Invalid address");

    tracing::info!("LIS API server starting on http://{}", addr);
    tracing::info!("Swagger UI available at http://{}/docs", addr);
    tracing::info!("");
    tracing::info!("Endpoints:");
    tracing::info!("  POST /api/compile    - Compile LIS to VSP assembly");
    tracing::info!("  POST /api/execute    - Compile and execute LIS code");
    tracing::info!("  POST /api/format     - Format LIS source code");
    tracing::info!("  POST /api/check      - Validate LIS syntax and types");
    tracing::info!("  GET  /api/intrinsics - List available stdlib functions");
    tracing::info!("  GET  /api/info       - Language information");
    tracing::info!("  GET  /health         - Health check");
    tracing::info!("  GET  /docs           - Swagger UI");
    tracing::info!("");
    tracing::info!("Configuration:");
    tracing::info!(
        "  Authentication: {}",
        if auth_config.enabled {
            format!("{} API keys configured", auth_config.keys.len())
        } else {
            "disabled".to_string()
        }
    );
    tracing::info!(
        "  Rate limiting: {}",
        if rate_config.enabled {
            format!(
                "{} req/s (burst: {})",
                rate_config.requests_per_second, rate_config.burst_size
            )
        } else {
            "disabled".to_string()
        }
    );

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
