//! OpenAPI documentation configuration

use utoipa::OpenApi;

use crate::handlers;
use crate::models::*;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "LIS API",
        version = "2026.1.16",
        description = "REST API for LIS (Language for Intelligent Systems) - Compile, execute, and format LIS code",
        license(name = "AGPL-3.0", url = "https://www.gnu.org/licenses/agpl-3.0.html"),
        contact(name = "SIL Contributors", url = "https://github.com/silvanoneto/SIL")
    ),
    servers(
        (url = "http://localhost:3000", description = "Local development server")
    ),
    tags(
        (name = "compilation", description = "Compile and execute LIS code"),
        (name = "formatting", description = "Format LIS source code"),
        (name = "introspection", description = "Language information and intrinsics"),
        (name = "health", description = "Health check endpoints")
    ),
    paths(
        handlers::compile_handler,
        handlers::execute_handler,
        handlers::format_handler,
        handlers::check_handler,
        handlers::intrinsics_handler,
        handlers::info_handler,
        handlers::health_handler,
    ),
    components(
        schemas(
            CompileRequest,
            FormatRequest,
            FormatOptions,
            ApiError,
            SourceLocation,
            CompileResult,
            ExecuteResult,
            ExecutionState,
            LayerValue,
            FormatResult,
            CheckResult,
            IntrinsicsResult,
            IntrinsicCategory,
            IntrinsicFunction,
            LangInfo,
            HealthCheck,
        )
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

/// Security addon for API key authentication
struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "api_key",
                utoipa::openapi::security::SecurityScheme::ApiKey(
                    utoipa::openapi::security::ApiKey::Header(
                        utoipa::openapi::security::ApiKeyValue::new("X-API-Key"),
                    ),
                ),
            );
        }
    }
}
