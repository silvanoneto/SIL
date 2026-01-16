//! Middleware for authentication and rate limiting

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};
use std::{num::NonZeroU32, sync::Arc};

use crate::models::{ApiError, ApiResponse};

// ============================================================================
// API Key Authentication
// ============================================================================

/// API key configuration
#[derive(Clone)]
pub struct ApiKeyConfig {
    /// Valid API keys (empty = no authentication required)
    pub keys: Vec<String>,
    /// Whether authentication is enabled
    pub enabled: bool,
}

impl Default for ApiKeyConfig {
    fn default() -> Self {
        Self {
            keys: Vec::new(),
            enabled: false,
        }
    }
}

impl ApiKeyConfig {
    /// Create config from environment variable
    pub fn from_env() -> Self {
        let keys_str = std::env::var("LIS_API_KEYS").unwrap_or_default();
        let keys: Vec<String> = keys_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Self {
            enabled: !keys.is_empty(),
            keys,
        }
    }

    /// Check if a key is valid
    pub fn is_valid(&self, key: &str) -> bool {
        if !self.enabled {
            return true;
        }
        self.keys.iter().any(|k| k == key)
    }
}

/// Authentication middleware
pub async fn auth_middleware(
    State(config): State<Arc<ApiKeyConfig>>,
    request: Request<Body>,
    next: Next,
) -> Response {
    // Skip auth for health check and docs
    let path = request.uri().path();
    if path == "/health" || path.starts_with("/docs") || path.starts_with("/swagger-ui") {
        return next.run(request).await;
    }

    // If auth is disabled, proceed
    if !config.enabled {
        return next.run(request).await;
    }

    // Check for API key in header
    let api_key = request
        .headers()
        .get("X-API-Key")
        .and_then(|v| v.to_str().ok());

    match api_key {
        Some(key) if config.is_valid(key) => next.run(request).await,
        Some(_) => {
            let error = ApiResponse::<()>::error(ApiError {
                code: "UNAUTHORIZED".to_string(),
                message: "Invalid API key".to_string(),
                location: None,
                help: Some("Provide a valid API key in the X-API-Key header".to_string()),
            });
            (StatusCode::UNAUTHORIZED, Json(error)).into_response()
        }
        None => {
            let error = ApiResponse::<()>::error(ApiError {
                code: "UNAUTHORIZED".to_string(),
                message: "API key required".to_string(),
                location: None,
                help: Some("Provide an API key in the X-API-Key header".to_string()),
            });
            (StatusCode::UNAUTHORIZED, Json(error)).into_response()
        }
    }
}

// ============================================================================
// Rate Limiting
// ============================================================================

/// Rate limiter type alias
pub type AppRateLimiter = RateLimiter<NotKeyed, InMemoryState, DefaultClock>;

/// Rate limiter configuration
#[derive(Clone)]
pub struct RateLimitConfig {
    /// Requests per second
    pub requests_per_second: u32,
    /// Burst size
    pub burst_size: u32,
    /// Whether rate limiting is enabled
    pub enabled: bool,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_second: 10,
            burst_size: 20,
            enabled: true,
        }
    }
}

impl RateLimitConfig {
    /// Create config from environment variables
    pub fn from_env() -> Self {
        let rps = std::env::var("LIS_API_RATE_LIMIT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10);

        let burst = std::env::var("LIS_API_RATE_BURST")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(20);

        let enabled = std::env::var("LIS_API_RATE_ENABLED")
            .map(|s| s != "false" && s != "0")
            .unwrap_or(true);

        Self {
            requests_per_second: rps,
            burst_size: burst,
            enabled,
        }
    }

    /// Create a rate limiter from this config
    pub fn create_limiter(&self) -> Arc<AppRateLimiter> {
        let quota = Quota::per_second(NonZeroU32::new(self.requests_per_second).unwrap())
            .allow_burst(NonZeroU32::new(self.burst_size).unwrap());

        Arc::new(RateLimiter::direct(quota))
    }
}

/// Rate limiting middleware
pub async fn rate_limit_middleware(
    State((limiter, config)): State<(Arc<AppRateLimiter>, RateLimitConfig)>,
    request: Request<Body>,
    next: Next,
) -> Response {
    // Skip rate limiting for health check
    if request.uri().path() == "/health" {
        return next.run(request).await;
    }

    // If rate limiting is disabled, proceed
    if !config.enabled {
        return next.run(request).await;
    }

    // Check rate limit
    match limiter.check() {
        Ok(_) => next.run(request).await,
        Err(_) => {
            let error = ApiResponse::<()>::error(ApiError {
                code: "RATE_LIMITED".to_string(),
                message: "Too many requests".to_string(),
                location: None,
                help: Some(format!(
                    "Rate limit: {} requests/second, burst: {}",
                    config.requests_per_second, config.burst_size
                )),
            });
            (StatusCode::TOO_MANY_REQUESTS, Json(error)).into_response()
        }
    }
}
