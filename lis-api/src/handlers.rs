//! Request handlers for API endpoints

use axum::{http::StatusCode, Json};
use lis_core::{compile, parse, Error as LisError, TypeErrorKind};
use lis_format::{format as format_lis, format_with_config, FormatConfig, IndentStyle};
use sil_core::vsp::{assemble, Vsp, VspConfig};

use crate::models::*;

/// Convert LIS error to API error
fn lis_error_to_api(err: LisError) -> ApiError {
    match err {
        LisError::LexError { message, line, col } => ApiError {
            code: "LEX_ERROR".to_string(),
            message,
            location: Some(SourceLocation { line, column: col }),
            help: None,
        },
        LisError::ParseError { message, line, col } => ApiError {
            code: "PARSE_ERROR".to_string(),
            message,
            location: Some(SourceLocation { line, column: col }),
            help: None,
        },
        LisError::SemanticError { message } => ApiError {
            code: "SEMANTIC_ERROR".to_string(),
            message,
            location: None,
            help: None,
        },
        LisError::TypeError {
            kind,
            line,
            col,
            help,
        } => {
            let (code, message) = type_error_details(&kind);
            ApiError {
                code,
                message,
                location: Some(SourceLocation { line, column: col }),
                help,
            }
        }
        LisError::CodeGenError { message } => ApiError {
            code: "CODEGEN_ERROR".to_string(),
            message,
            location: None,
            help: None,
        },
        LisError::IoError { message } => ApiError {
            code: "IO_ERROR".to_string(),
            message,
            location: None,
            help: None,
        },
        LisError::Manifest(message) => ApiError {
            code: "MANIFEST_ERROR".to_string(),
            message,
            location: None,
            help: Some("Check your lis.toml file".to_string()),
        },
        LisError::ModuleError { message, path } => ApiError {
            code: "MODULE_ERROR".to_string(),
            message: if let Some(p) = path {
                format!("{} (in {})", message, p)
            } else {
                message
            },
            location: None,
            help: Some("Check module path and dependencies".to_string()),
        },
    }
}

/// Extract details from type error kind
fn type_error_details(kind: &TypeErrorKind) -> (String, String) {
    match kind {
        TypeErrorKind::Mismatch {
            expected,
            found,
            context,
        } => (
            "TYPE_MISMATCH".to_string(),
            format!(
                "Expected type '{}', found '{}'{}",
                expected,
                found,
                if context.is_empty() {
                    String::new()
                } else {
                    format!(" in {}", context)
                }
            ),
        ),
        TypeErrorKind::UndefinedVariable { name } => (
            "UNDEFINED_VARIABLE".to_string(),
            format!("Undefined variable '{}'", name),
        ),
        TypeErrorKind::InvalidLayerAccess { layer } => (
            "INVALID_LAYER".to_string(),
            format!("Invalid layer access L{:X} (valid range: L0-LF)", layer),
        ),
        TypeErrorKind::HardwareConflict { required, found } => (
            "HARDWARE_CONFLICT".to_string(),
            format!("Hardware conflict: required {}, found {}", required, found),
        ),
        TypeErrorKind::InfiniteType { var } => (
            "INFINITE_TYPE".to_string(),
            format!("Infinite type detected (type variable {})", var),
        ),
        TypeErrorKind::ArgumentCountMismatch {
            expected,
            found,
            function,
        } => (
            "ARGUMENT_COUNT".to_string(),
            format!(
                "Function '{}' expects {} arguments, got {}",
                function, expected, found
            ),
        ),
        TypeErrorKind::InvalidOperation { op, left, right } => {
            let msg = if let Some(right) = right {
                format!("Cannot apply '{}' to types {} and {}", op, left, right)
            } else {
                format!("Cannot apply '{}' to type {}", op, left)
            };
            ("INVALID_OPERATION".to_string(), msg)
        }
    }
}

// ============================================================================
// Handler Functions
// ============================================================================

/// Compile LIS source code to VSP assembly
#[utoipa::path(
    post,
    path = "/api/compile",
    tag = "compilation",
    request_body = CompileRequest,
    responses(
        (status = 200, description = "Compilation successful", body = ApiResponse<CompileResult>),
        (status = 400, description = "Compilation error", body = ApiResponse<()>),
    ),
    security(("api_key" = []))
)]
pub async fn compile_handler(
    Json(req): Json<CompileRequest>,
) -> (StatusCode, Json<ApiResponse<CompileResult>>) {
    match compile(&req.source) {
        Ok(assembly) => {
            let instruction_count = assembly.lines().filter(|l| !l.trim().is_empty() && !l.starts_with(';') && !l.starts_with('.')).count();
            (
                StatusCode::OK,
                Json(ApiResponse::success(CompileResult {
                    assembly,
                    instruction_count,
                })),
            )
        }
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(lis_error_to_api(err))),
        ),
    }
}

/// Compile and execute LIS code, returning the final 16-layer state
#[utoipa::path(
    post,
    path = "/api/execute",
    tag = "compilation",
    request_body = CompileRequest,
    responses(
        (status = 200, description = "Execution successful", body = ApiResponse<ExecuteResult>),
        (status = 400, description = "Compilation error", body = ApiResponse<()>),
        (status = 500, description = "Execution error", body = ApiResponse<()>),
    ),
    security(("api_key" = []))
)]
pub async fn execute_handler(
    Json(req): Json<CompileRequest>,
) -> (StatusCode, Json<ApiResponse<ExecuteResult>>) {
    // Step 1: Compile to assembly
    let assembly = match compile(&req.source) {
        Ok(asm) => asm,
        Err(err) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::error(lis_error_to_api(err))),
            );
        }
    };

    // Step 2: Assemble to bytecode
    let silc = match assemble(&assembly) {
        Ok(s) => s,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(ApiError {
                    code: "ASSEMBLE_ERROR".to_string(),
                    message: format!("Failed to assemble: {}", err),
                    location: None,
                    help: None,
                })),
            );
        }
    };

    let bytecode = silc.to_bytes();
    let bytecode_size = bytecode.len();

    // Step 3: Create VSP and execute
    let mut vsp = match Vsp::new(VspConfig::default()) {
        Ok(v) => v,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(ApiError {
                    code: "VSP_INIT_ERROR".to_string(),
                    message: format!("Failed to initialize VSP: {}", err),
                    location: None,
                    help: None,
                })),
            );
        }
    };

    if let Err(err) = vsp.load(&bytecode) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(ApiError {
                code: "VSP_LOAD_ERROR".to_string(),
                message: format!("Failed to load bytecode: {}", err),
                location: None,
                help: None,
            })),
        );
    }

    let result = match vsp.run() {
        Ok(r) => r,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(ApiError {
                    code: "EXECUTION_ERROR".to_string(),
                    message: format!("Execution failed: {}", err),
                    location: None,
                    help: None,
                })),
            );
        }
    };

    // Build state representation
    let layers: Vec<LayerValue> = (0..16)
        .map(|i| {
            let layer = result.layer(i);
            LayerValue {
                index: i as u8,
                name: ExecutionState::LAYER_NAMES[i],
                value: format!("{:02X}", layer.to_u8()),
            }
        })
        .collect();

    (
        StatusCode::OK,
        Json(ApiResponse::success(ExecuteResult {
            assembly,
            bytecode_size,
            state: ExecutionState { layers },
            completed: true,
        })),
    )
}

/// Format LIS source code with configurable options
#[utoipa::path(
    post,
    path = "/api/format",
    tag = "formatting",
    request_body = FormatRequest,
    responses(
        (status = 200, description = "Formatting successful", body = ApiResponse<FormatResult>),
        (status = 400, description = "Parse error", body = ApiResponse<()>),
    ),
    security(("api_key" = []))
)]
pub async fn format_handler(
    Json(req): Json<FormatRequest>,
) -> (StatusCode, Json<ApiResponse<FormatResult>>) {
    // Build config from options
    let indent_size = req.options.indent_size.unwrap_or(4);
    let config = FormatConfig {
        indent_style: req
            .options
            .indent_style
            .as_deref()
            .map(|s| {
                if s == "tabs" {
                    IndentStyle::Tabs
                } else {
                    IndentStyle::Spaces(indent_size)
                }
            })
            .unwrap_or(IndentStyle::Spaces(indent_size)),
        max_width: req.options.max_width.unwrap_or(100),
        ..Default::default()
    };

    // Check if already formatted
    let was_formatted = format_lis(&req.source)
        .map(|f| f == req.source)
        .unwrap_or(false);

    match format_with_config(&req.source, &config) {
        Ok(formatted) => (
            StatusCode::OK,
            Json(ApiResponse::success(FormatResult {
                formatted,
                was_formatted,
            })),
        ),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(ApiError {
                code: "FORMAT_ERROR".to_string(),
                message: format!("{}", err),
                location: None,
                help: Some("Ensure the source code is syntactically valid".to_string()),
            })),
        ),
    }
}

/// Validate LIS syntax and types without execution
#[utoipa::path(
    post,
    path = "/api/check",
    tag = "compilation",
    request_body = CompileRequest,
    responses(
        (status = 200, description = "Validation successful", body = ApiResponse<CheckResult>),
        (status = 400, description = "Validation error", body = ApiResponse<()>),
    ),
    security(("api_key" = []))
)]
pub async fn check_handler(
    Json(req): Json<CompileRequest>,
) -> (StatusCode, Json<ApiResponse<CheckResult>>) {
    // Try to parse and compile (validates syntax and semantics)
    match parse(&req.source) {
        Ok(_ast) => {
            // Also try full compilation to catch codegen issues
            match compile(&req.source) {
                Ok(_) => (
                    StatusCode::OK,
                    Json(ApiResponse::success(CheckResult {
                        valid: true,
                        messages: vec!["Syntax and types are valid".to_string()],
                    })),
                ),
                Err(err) => (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::error(lis_error_to_api(err))),
                ),
            }
        }
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(lis_error_to_api(err))),
        ),
    }
}

/// List all available stdlib intrinsic functions grouped by category
#[utoipa::path(
    get,
    path = "/api/intrinsics",
    tag = "introspection",
    responses(
        (status = 200, description = "Intrinsics listing", body = ApiResponse<IntrinsicsResult>),
    ),
    security(("api_key" = []))
)]
pub async fn intrinsics_handler() -> Json<ApiResponse<IntrinsicsResult>> {
    let categories = vec![
        IntrinsicCategory {
            name: "ByteSil Operations".to_string(),
            functions: vec![
                ("bytesil_new", "Create ByteSil from magnitude and phase"),
                ("bytesil_from_complex", "Convert complex to ByteSil"),
                ("bytesil_to_complex", "Convert ByteSil to complex"),
                ("bytesil_null", "Zero/null ByteSil"),
                ("bytesil_one", "Unit magnitude ByteSil (1+0i)"),
                ("bytesil_i", "Imaginary unit (0+1i)"),
                ("bytesil_neg_one", "Negative unit (-1+0i)"),
                ("bytesil_neg_i", "Negative imaginary (0-1i)"),
                ("bytesil_max", "Maximum magnitude ByteSil"),
                ("bytesil_mul", "Multiply two ByteSils"),
                ("bytesil_div", "Divide two ByteSils"),
                ("bytesil_pow", "Power operation"),
                ("bytesil_root", "Root operation"),
                ("bytesil_inv", "Multiplicative inverse"),
                ("bytesil_conj", "Complex conjugate"),
                ("bytesil_xor", "XOR operation"),
                ("bytesil_mix", "Mix two ByteSils"),
                ("bytesil_rho", "Get magnitude (rho)"),
                ("bytesil_theta", "Get phase (theta)"),
                ("bytesil_magnitude", "Get magnitude as float"),
                ("bytesil_phase_degrees", "Get phase in degrees"),
                ("bytesil_phase_radians", "Get phase in radians"),
                ("bytesil_is_null", "Check if null"),
                ("bytesil_is_real", "Check if purely real"),
                ("bytesil_is_imaginary", "Check if purely imaginary"),
            ]
            .into_iter()
            .map(|(name, desc)| IntrinsicFunction {
                name: name.to_string(),
                description: Some(desc.to_string()),
            })
            .collect(),
        },
        IntrinsicCategory {
            name: "Math Functions".to_string(),
            functions: vec![
                ("sin", "Sine"),
                ("cos", "Cosine"),
                ("tan", "Tangent"),
                ("asin", "Arc sine"),
                ("acos", "Arc cosine"),
                ("atan", "Arc tangent"),
                ("atan2", "Two-argument arc tangent"),
                ("pi", "Pi constant"),
                ("tau", "Tau constant (2*pi)"),
                ("e", "Euler's number"),
                ("phi", "Golden ratio"),
                ("sqrt", "Square root"),
                ("pow_float", "Power (float)"),
                ("exp", "Exponential (e^x)"),
                ("ln", "Natural logarithm"),
                ("log10", "Base-10 logarithm"),
                ("log2", "Base-2 logarithm"),
                ("abs_int", "Absolute value (int)"),
                ("abs_float", "Absolute value (float)"),
                ("min_int", "Minimum (int)"),
                ("max_int", "Maximum (int)"),
                ("clamp_int", "Clamp value (int)"),
                ("clamp_float", "Clamp value (float)"),
                ("floor", "Floor"),
                ("ceil", "Ceiling"),
                ("round", "Round"),
                ("degrees_to_radians", "Convert degrees to radians"),
                ("radians_to_degrees", "Convert radians to degrees"),
            ]
            .into_iter()
            .map(|(name, desc)| IntrinsicFunction {
                name: name.to_string(),
                description: Some(desc.to_string()),
            })
            .collect(),
        },
        IntrinsicCategory {
            name: "State Operations".to_string(),
            functions: vec![
                ("state_vacuum", "Create vacuum state (all zeros)"),
                ("state_neutral", "Create neutral state"),
                ("state_maximum", "Create maximum state"),
                ("state_from_bytes", "Create state from 16 bytes"),
                ("state_from_layers", "Create state from layer values"),
                ("state_get_layer", "Get specific layer value"),
                ("state_set_layer", "Set specific layer value"),
                ("state_tensor", "Tensor product of states"),
                ("state_xor", "XOR two states"),
                ("state_project", "Project state onto subspace"),
                ("state_collapse_xor", "Collapse via XOR"),
                ("state_collapse_sum", "Collapse via sum"),
                ("state_equals", "Compare states"),
                ("state_is_vacuum", "Check if vacuum state"),
                ("perception_mask", "Get perception layer mask"),
                ("processing_mask", "Get processing layer mask"),
                ("interaction_mask", "Get interaction layer mask"),
                ("emergence_mask", "Get emergence layer mask"),
                ("meta_mask", "Get meta layer mask"),
            ]
            .into_iter()
            .map(|(name, desc)| IntrinsicFunction {
                name: name.to_string(),
                description: Some(desc.to_string()),
            })
            .collect(),
        },
        IntrinsicCategory {
            name: "Console I/O".to_string(),
            functions: vec![
                ("print_int", "Print integer"),
                ("print_float", "Print float"),
                ("print_string", "Print string"),
                ("print_bool", "Print boolean"),
                ("print_bytesil", "Print ByteSil"),
                ("print_state", "Print SIL state"),
                ("println", "Print with newline"),
                ("read_line", "Read line from stdin"),
                ("read_int", "Read integer from stdin"),
                ("read_float", "Read float from stdin"),
            ]
            .into_iter()
            .map(|(name, desc)| IntrinsicFunction {
                name: name.to_string(),
                description: Some(desc.to_string()),
            })
            .collect(),
        },
        IntrinsicCategory {
            name: "String Operations".to_string(),
            functions: vec![
                ("string_length", "Get string length"),
                ("string_concat", "Concatenate strings"),
                ("string_slice", "Extract substring"),
                ("string_to_upper", "Convert to uppercase"),
                ("string_to_lower", "Convert to lowercase"),
                ("string_contains", "Check if contains substring"),
                ("string_starts_with", "Check prefix"),
                ("string_ends_with", "Check suffix"),
                ("string_equals", "Compare strings"),
                ("int_to_string", "Convert int to string"),
                ("float_to_string", "Convert float to string"),
                ("string_to_int", "Parse string to int"),
                ("string_to_float", "Parse string to float"),
                ("string_trim", "Trim whitespace"),
                ("string_replace", "Replace substring"),
            ]
            .into_iter()
            .map(|(name, desc)| IntrinsicFunction {
                name: name.to_string(),
                description: Some(desc.to_string()),
            })
            .collect(),
        },
        IntrinsicCategory {
            name: "Layer Operations".to_string(),
            functions: vec![
                ("fuse_vision_audio", "Fuse vision and audio layers"),
                ("fuse_multimodal", "Fuse multiple modalities"),
                ("normalize_perception", "Normalize perception layers"),
                ("shift_layers_up", "Shift layers up"),
                ("shift_layers_down", "Shift layers down"),
                ("rotate_layers", "Rotate layers"),
                ("spread_to_group", "Spread value to layer group"),
            ]
            .into_iter()
            .map(|(name, desc)| IntrinsicFunction {
                name: name.to_string(),
                description: Some(desc.to_string()),
            })
            .collect(),
        },
        IntrinsicCategory {
            name: "Transform Operations".to_string(),
            functions: vec![
                ("transform_phase_shift", "Apply phase shift"),
                ("transform_magnitude_scale", "Scale magnitude"),
                ("transform_layer_swap", "Swap layers"),
                ("transform_xor_layers", "XOR specific layers"),
                ("transform_identity", "Identity transform"),
                ("apply_feedback", "Apply feedback loop"),
                ("detect_emergence", "Detect emergent patterns"),
                ("emergence_pattern", "Get emergence pattern"),
                ("autopoietic_loop", "Self-organizing loop"),
            ]
            .into_iter()
            .map(|(name, desc)| IntrinsicFunction {
                name: name.to_string(),
                description: Some(desc.to_string()),
            })
            .collect(),
        },
        IntrinsicCategory {
            name: "Debug Utilities".to_string(),
            functions: vec![
                ("assert", "Assert condition"),
                ("assert_eq_int", "Assert int equality"),
                ("assert_eq_bytesil", "Assert ByteSil equality"),
                ("assert_eq_state", "Assert state equality"),
                ("debug_print", "Debug print"),
                ("trace_state", "Trace state execution"),
                ("timestamp_millis", "Get timestamp (ms)"),
                ("timestamp_micros", "Get timestamp (us)"),
                ("sleep_millis", "Sleep for milliseconds"),
                ("memory_used", "Get memory usage"),
            ]
            .into_iter()
            .map(|(name, desc)| IntrinsicFunction {
                name: name.to_string(),
                description: Some(desc.to_string()),
            })
            .collect(),
        },
        IntrinsicCategory {
            name: "HTTP Client".to_string(),
            functions: vec![
                ("http_get", "HTTP GET request, returns body"),
                ("http_get_with_status", "HTTP GET, returns (status, body)"),
                ("http_post", "HTTP POST with text body"),
                ("http_post_json", "HTTP POST with JSON body"),
                ("http_put", "HTTP PUT with text body"),
                ("http_put_json", "HTTP PUT with JSON body"),
                ("http_delete", "HTTP DELETE request"),
                ("http_patch", "HTTP PATCH with text body"),
                ("http_patch_json", "HTTP PATCH with JSON body"),
                ("http_head", "HTTP HEAD, returns status code"),
                ("http_get_with_header", "HTTP GET with custom header"),
                ("http_get_auth", "HTTP GET with Bearer token"),
                ("http_post_auth", "HTTP POST with Bearer token"),
                ("url_encode", "URL-encode a string"),
                ("url_decode", "URL-decode a string"),
                ("http_status_ok", "Check if status is 2xx"),
                ("http_status_text", "Get status code description"),
            ]
            .into_iter()
            .map(|(name, desc)| IntrinsicFunction {
                name: name.to_string(),
                description: Some(desc.to_string()),
            })
            .collect(),
        },
    ];

    let total: usize = categories.iter().map(|c| c.functions.len()).sum();

    Json(ApiResponse::success(IntrinsicsResult { total, categories }))
}

/// Get LIS language information and features
#[utoipa::path(
    get,
    path = "/api/info",
    tag = "introspection",
    responses(
        (status = 200, description = "Language information", body = ApiResponse<LangInfo>),
    ),
    security(("api_key" = []))
)]
pub async fn info_handler() -> Json<ApiResponse<LangInfo>> {
    Json(ApiResponse::success(LangInfo {
        name: "LIS",
        version: "2026.1.16",
        full_name: "Language for Intelligent Systems",
        description: "A programming language for modeling non-linear systems that compiles to SIL VSP bytecode",
        features: vec![
            "Native feedback loops and causal cycles",
            "Topology and continuous transformations",
            "Emergence and self-organization",
            "Hardware-aware compilation (CPU/GPU/NPU)",
            "16-layer state model (L0-LF)",
            "ByteSil log-polar complex arithmetic",
            "Reflexive metaprogramming",
        ],
        target: "SIL VSP (Virtual Sil Processor)",
    }))
}

/// Health check endpoint for monitoring
#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthCheck),
    )
)]
pub async fn health_handler() -> Json<HealthCheck> {
    Json(HealthCheck {
        status: "healthy",
        version: "2026.1.16",
    })
}
