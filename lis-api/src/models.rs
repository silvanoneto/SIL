//! Data models for API requests and responses

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// ============================================================================
// Request Models
// ============================================================================

/// Request body for compilation endpoints
#[derive(Debug, Deserialize, ToSchema)]
pub struct CompileRequest {
    /// LIS source code to compile
    #[schema(example = "fn main() { let x = 42; return x; }")]
    pub source: String,
}

/// Request body for format endpoint
#[derive(Debug, Deserialize, ToSchema)]
pub struct FormatRequest {
    /// LIS source code to format
    #[schema(example = "fn main(){let x=42;}")]
    pub source: String,

    /// Formatting options (optional)
    #[serde(default)]
    pub options: FormatOptions,
}

/// Formatting configuration options
#[derive(Debug, Deserialize, Default, ToSchema)]
pub struct FormatOptions {
    /// Indent with tabs or spaces (default: spaces)
    #[serde(default)]
    #[schema(example = "spaces")]
    pub indent_style: Option<String>,

    /// Number of spaces per indent level (default: 4)
    #[serde(default)]
    #[schema(example = 4)]
    pub indent_size: Option<usize>,

    /// Maximum line width (default: 100)
    #[serde(default)]
    #[schema(example = 100)]
    pub max_width: Option<usize>,
}

// ============================================================================
// Response Models
// ============================================================================

/// Generic API response wrapper
#[derive(Debug, Serialize, ToSchema)]
pub struct ApiResponse<T: Serialize> {
    /// Whether the operation succeeded
    pub success: bool,

    /// Response data (present on success)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,

    /// Error information (present on failure)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiError>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(error: ApiError) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
        }
    }
}

/// Error information in API responses
#[derive(Debug, Serialize, ToSchema)]
pub struct ApiError {
    /// Error code (e.g., "PARSE_ERROR", "TYPE_MISMATCH")
    #[schema(example = "PARSE_ERROR")]
    pub code: String,

    /// Human-readable error message
    #[schema(example = "Unexpected token at line 1")]
    pub message: String,

    /// Source location (for compilation errors)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<SourceLocation>,

    /// Helpful suggestion for fixing the error
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "Check syntax near the error location")]
    pub help: Option<String>,
}

/// Source code location
#[derive(Debug, Serialize, ToSchema)]
pub struct SourceLocation {
    #[schema(example = 1)]
    pub line: usize,
    #[schema(example = 10)]
    pub column: usize,
}

/// Compilation result
#[derive(Debug, Serialize, ToSchema)]
pub struct CompileResult {
    /// Generated VSP assembly code
    #[schema(example = ".mode SIL-128\n\n.code\n\nmain:\n    MOVI R0, 42\n    HLT")]
    pub assembly: String,

    /// Number of instructions generated
    #[schema(example = 3)]
    pub instruction_count: usize,
}

/// Execution result
#[derive(Debug, Serialize, ToSchema)]
pub struct ExecuteResult {
    /// Generated VSP assembly (intermediate)
    pub assembly: String,

    /// Bytecode size in bytes
    #[schema(example = 52)]
    pub bytecode_size: usize,

    /// Final 16-layer state
    pub state: ExecutionState,

    /// Execution completed successfully
    pub completed: bool,
}

/// 16-layer SIL state representation
#[derive(Debug, Serialize, ToSchema)]
pub struct ExecutionState {
    /// Layer values (L0-LF as hex strings)
    pub layers: Vec<LayerValue>,
}

/// Individual layer value
#[derive(Debug, Serialize, ToSchema)]
pub struct LayerValue {
    /// Layer index (0-15)
    #[schema(example = 0)]
    pub index: u8,

    /// Layer name (e.g., "Photonic", "Acoustic")
    #[schema(example = "Photonic")]
    pub name: &'static str,

    /// Layer value as hex string
    #[schema(example = "2A")]
    pub value: String,
}

impl ExecutionState {
    /// Layer names for L0-LF
    pub const LAYER_NAMES: [&'static str; 16] = [
        "Photonic",
        "Acoustic",
        "Olfactory",
        "Gustatory",
        "Dermic",
        "Electronic",
        "Psychomotor",
        "Environmental",
        "Cybernetic",
        "Geopolitical",
        "Cosmopolitan",
        "Synergic",
        "Quantum",
        "Superposition",
        "Entanglement",
        "Collapse",
    ];
}

/// Format result
#[derive(Debug, Serialize, ToSchema)]
pub struct FormatResult {
    /// Formatted source code
    #[schema(example = "fn main() {\n    let x = 42;\n}\n")]
    pub formatted: String,

    /// Whether the code was already formatted
    pub was_formatted: bool,
}

/// Check/validation result
#[derive(Debug, Serialize, ToSchema)]
pub struct CheckResult {
    /// Syntax is valid
    pub valid: bool,

    /// Informational messages
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub messages: Vec<String>,
}

/// Intrinsics listing
#[derive(Debug, Serialize, ToSchema)]
pub struct IntrinsicsResult {
    /// Total count of intrinsics
    #[schema(example = 123)]
    pub total: usize,

    /// Intrinsics grouped by category
    pub categories: Vec<IntrinsicCategory>,
}

/// Category of intrinsic functions
#[derive(Debug, Serialize, ToSchema)]
pub struct IntrinsicCategory {
    /// Category name
    #[schema(example = "ByteSil Operations")]
    pub name: String,

    /// Functions in this category
    pub functions: Vec<IntrinsicFunction>,
}

/// Individual intrinsic function
#[derive(Debug, Serialize, ToSchema)]
pub struct IntrinsicFunction {
    /// Function name
    #[schema(example = "bytesil_new")]
    pub name: String,

    /// Brief description
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "Create ByteSil from magnitude and phase")]
    pub description: Option<String>,
}

/// Language information
#[derive(Debug, Serialize, ToSchema)]
pub struct LangInfo {
    /// Language name
    #[schema(example = "LIS")]
    pub name: &'static str,

    /// Version
    #[schema(example = "2026.1.16")]
    pub version: &'static str,

    /// Full name
    #[schema(example = "Language for Intelligent Systems")]
    pub full_name: &'static str,

    /// Description
    pub description: &'static str,

    /// List of features
    pub features: Vec<&'static str>,

    /// Compilation target
    #[schema(example = "SIL VSP (Virtual Sil Processor)")]
    pub target: &'static str,
}

/// Health check response
#[derive(Debug, Serialize, ToSchema)]
pub struct HealthCheck {
    /// Service status
    #[schema(example = "healthy")]
    pub status: &'static str,

    /// Service version
    #[schema(example = "2026.1.16")]
    pub version: &'static str,
}
