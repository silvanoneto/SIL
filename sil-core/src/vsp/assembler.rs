//! Assembler SIL (silasm)
//!
//! Compila código assembly SIL (.sil) para bytecode (.silc)
//!
//! # Sintaxe
//!
//! ```text
//! ; Comentário
//! .mode SIL-128          ; Diretiva de modo
//! .data                   ; Seção de dados
//!     state1: [FF, 00, ...]
//! .code                   ; Seção de código
//! start:                  ; Label
//!     LSTATE R0, state1   ; Instrução
//!     MUL R0, R1
//!     JMP start
//! ```

use std::collections::HashMap;

use super::{
    Opcode, InstructionFormat, SilcBuilder, SilcFile, VspError, VspResult,
    bytecode::SymbolInfo,
    state::SilMode,
};

// ═══════════════════════════════════════════════════════════════════════════════
// STDLIB INTRINSICS
// ═══════════════════════════════════════════════════════════════════════════════

/// IDs for stdlib intrinsic functions (used with SYSCALL opcode)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum StdlibIntrinsic {
    // ═══════════════════════════════════════════════════════════════════════════
    // I/O Functions (0x00-0x0F)
    // ═══════════════════════════════════════════════════════════════════════════
    Println = 0x00,
    PrintString = 0x01,
    PrintInt = 0x02,
    PrintFloat = 0x03,
    PrintBool = 0x04,
    PrintBytesil = 0x05,
    PrintState = 0x06,
    ReadLine = 0x07,
    ReadInt = 0x08,
    ReadFloat = 0x09,

    // ═══════════════════════════════════════════════════════════════════════════
    // ByteSil Functions (0x10-0x2F)
    // ═══════════════════════════════════════════════════════════════════════════
    BytesilNew = 0x10,
    BytesilFromComplex = 0x11,
    BytesilToComplex = 0x12,
    BytesilNull = 0x13,
    BytesilOne = 0x14,
    BytesilI = 0x15,
    BytesilNegOne = 0x16,
    BytesilNegI = 0x17,
    BytesilMax = 0x18,
    BytesilMul = 0x19,
    BytesilDiv = 0x1A,
    BytesilPow = 0x1B,
    BytesilRoot = 0x1C,
    BytesilInv = 0x1D,
    BytesilConj = 0x1E,
    BytesilXor = 0x1F,
    BytesilMix = 0x20,
    BytesilRho = 0x21,
    BytesilTheta = 0x22,
    BytesilMagnitude = 0x23,
    BytesilPhaseDegrees = 0x24,
    BytesilPhaseRadians = 0x25,
    BytesilIsNull = 0x26,
    BytesilIsReal = 0x27,
    BytesilIsImaginary = 0x28,
    BytesilNorm = 0x29,
    BytesilFromU8 = 0x2A,
    BytesilToU8 = 0x2B,
    BytesilAdd = 0x2C,
    BytesilSub = 0x2D,
    BytesilScale = 0x2E,
    BytesilRotate = 0x2F,

    // ═══════════════════════════════════════════════════════════════════════════
    // State Functions (0x30-0x4F)
    // ═══════════════════════════════════════════════════════════════════════════
    StateVacuum = 0x30,
    StateNeutral = 0x31,
    StateMax = 0x32,
    StateFromBytes = 0x33,
    StateToBytes = 0x34,
    StateGetLayer = 0x35,
    StateSetLayer = 0x36,
    StateXor = 0x37,
    StateFold = 0x38,
    StateRotate = 0x39,
    StateCollapse = 0x3A,
    StateCountActiveLayers = 0x3B,
    StateIsVacuum = 0x3C,
    StateHash = 0x3D,
    StateCompare = 0x3E,
    StateIsNeutral = 0x3F,
    StateEquals = 0x40,
    StateCountNullLayers = 0x41,
    StateTensor = 0x42,
    StateProject = 0x43,
    StateNormalize = 0x44,
    StateCollapseMag = 0x45,
    StateCollapseSum = 0x46,
    StateCollapseFirst = 0x47,
    StateCollapseLast = 0x48,

    // ═══════════════════════════════════════════════════════════════════════════
    // Math Functions (0x50-0x6F)
    // ═══════════════════════════════════════════════════════════════════════════
    MathSin = 0x50,
    MathCos = 0x51,
    MathTan = 0x52,
    MathSqrt = 0x53,
    MathPow = 0x54,
    MathLog = 0x55,
    MathExp = 0x56,
    MathAbs = 0x57,
    MathFloor = 0x58,
    MathCeil = 0x59,
    MathRound = 0x5A,
    MathMin = 0x5B,
    MathMax = 0x5C,
    // Math constants
    MathPi = 0x5D,
    MathTau = 0x5E,
    MathE = 0x5F,
    MathPhi = 0x60,
    // More math functions
    MathAsin = 0x61,
    MathAcos = 0x62,
    MathAtan = 0x63,
    MathAtan2 = 0x64,
    MathLn = 0x65,
    MathLog10 = 0x66,
    MathLog2 = 0x67,
    MathAbsInt = 0x68,
    MathAbsFloat = 0x69,
    MathMinInt = 0x6A,
    MathMaxInt = 0x6B,
    MathClampInt = 0x6C,
    MathPowFloat = 0x6D,

    // ═══════════════════════════════════════════════════════════════════════════
    // String Functions (0x70-0x8F)
    // ═══════════════════════════════════════════════════════════════════════════
    StringLen = 0x70,
    StringConcat = 0x71,
    StringSubstr = 0x72,
    StringContains = 0x73,
    StringReplace = 0x74,
    StringSplit = 0x75,
    StringTrim = 0x76,
    StringToUpper = 0x77,
    StringToLower = 0x78,
    StringSlice = 0x79,
    StringStartsWith = 0x7A,
    StringEndsWith = 0x7B,
    IntToString = 0x7C,
    FloatToString = 0x7D,
    StringToInt = 0x7E,
    StringToFloat = 0x7F,

    // ═══════════════════════════════════════════════════════════════════════════
    // Type Conversions (0x80-0x8F)
    // ═══════════════════════════════════════════════════════════════════════════
    FloatFromInt = 0x80,
    IntFromFloat = 0x81,
    FloatToInt = 0x82,
    IntToFloat = 0x83,

    // ═══════════════════════════════════════════════════════════════════════════
    // Transform Functions (0x90-0x9F)
    // ═══════════════════════════════════════════════════════════════════════════
    ApplyFeedback = 0x90,
    DetectEmergence = 0x91,
    TransformPhaseShift = 0x92,
    TransformMagnitudeScale = 0x93,
    TransformIdentity = 0x94,
    TransformLayerSwap = 0x95,
    TransformXorLayers = 0x96,
    ShiftLayersUp = 0x97,
    ShiftLayersDown = 0x98,
    NormalizePerception = 0x99,

    // ═══════════════════════════════════════════════════════════════════════════
    // HTTP Functions (0xA0-0xBF)
    // ═══════════════════════════════════════════════════════════════════════════
    HttpGet = 0xA0,
    HttpGetWithStatus = 0xA1,
    HttpPost = 0xA2,
    HttpPostJson = 0xA3,
    HttpPut = 0xA4,
    HttpPutJson = 0xA5,
    HttpDelete = 0xA6,
    HttpPatch = 0xA7,
    HttpPatchJson = 0xA8,
    HttpHead = 0xA9,
    HttpGetWithHeader = 0xAA,
    HttpGetAuth = 0xAB,
    HttpPostAuth = 0xAC,
    UrlEncode = 0xAD,
    UrlDecode = 0xAE,
    HttpStatusOk = 0xAF,
    HttpStatusText = 0xB0,

    // ═══════════════════════════════════════════════════════════════════════════
    // Layer Functions (0xC0-0xCF)
    // ═══════════════════════════════════════════════════════════════════════════
    FuseVisionAudio = 0xC0,
    FuseMultimodal = 0xC1,
    RotateLayers = 0xC2,
    SpreadToGroup = 0xC3,
    EmergencePattern = 0xC4,
    AutopoieticLoop = 0xC5,

    // ═══════════════════════════════════════════════════════════════════════════
    // Complex Math (0xD0-0xDF)
    // ═══════════════════════════════════════════════════════════════════════════
    ComplexAdd = 0xD0,
    ComplexSub = 0xD1,
    ComplexScale = 0xD2,
    ComplexRotate = 0xD3,
    ComplexLerp = 0xD4,
    DegreesToRadians = 0xD5,
    RadiansToDegrees = 0xD6,
    ClampFloat = 0xD7,
    MinFloat = 0xD8,
    MaxFloat = 0xD9,
    SignFloat = 0xDA,
    SignInt = 0xDB,

    // ═══════════════════════════════════════════════════════════════════════════
    // Energy & Benchmark Metrics (0xE0-0xEF)
    // ═══════════════════════════════════════════════════════════════════════════
    EnergyBegin = 0xE0,              // Inicia medição de energia
    EnergyEndJoules = 0xE1,          // Termina e retorna Joules
    EnergyEndWatts = 0xE2,           // Termina e retorna Watts
    EnergyEndSamplesPerJoule = 0xE3, // Retorna samples/J
    CarbonFootprintGrams = 0xE4,     // Retorna gCO2e (baseado em região)
    TimestampNanosLow = 0xE5,        // Timestamp ns (32-bit low)
    TimestampNanosHigh = 0xE6,       // Timestamp ns (32-bit high)
    BenchmarkStart = 0xE7,           // Inicia benchmark
    BenchmarkEnd = 0xE8,             // Termina benchmark e retorna métricas
    FlopCount = 0xE9,                // Contador de FLOPs
    MacCount = 0xEA,                 // Contador de MACs
    ThroughputSamplesPerSec = 0xEB,  // Samples/segundo atual
    LatencyMicros = 0xEC,            // Latência média em microssegundos
    MemoryPeakBytes = 0xED,          // Pico de memória em bytes
    AccuracyPerWatt = 0xEE,          // Accuracy por Watt
    EnergyDelayProduct = 0xEF,       // Energy-Delay Product (EDP)

    // ═══════════════════════════════════════════════════════════════════════════
    // Debug Functions (0xF0-0xFF)
    // ═══════════════════════════════════════════════════════════════════════════
    DebugPrint = 0xF0,
    AssertEq = 0xF1,
    AssertTrue = 0xF2,
    AssertFalse = 0xF3,
    TraceState = 0xF4,
    TimestampMillis = 0xF5,
    TimestampMicros = 0xF6,
    SleepMillis = 0xF7,
    MemoryUsed = 0xF8,
    AssertEqInt = 0xF9,
    AssertEqBytesil = 0xFA,
    AssertEqState = 0xFB,
    TimestampNanosState = 0xFC,      // Timestamp em ns (State para 128-bit)
    EnergyTotalJoules = 0xFD,        // Energia total acumulada
    EnergyEfficiency = 0xFE,         // Eficiência energética (ops/J)
    BenchmarkReset = 0xFF,           // Reset de métricas de benchmark
}

impl StdlibIntrinsic {
    /// Get intrinsic ID from name
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            // ═══════════════════════════════════════════════════════════════════
            // I/O
            // ═══════════════════════════════════════════════════════════════════
            "println" => Self::Println,
            "print_string" => Self::PrintString,
            "print_int" => Self::PrintInt,
            "print_float" => Self::PrintFloat,
            "print_bool" => Self::PrintBool,
            "print_bytesil" => Self::PrintBytesil,
            "print_state" => Self::PrintState,
            "read_line" => Self::ReadLine,
            "read_int" => Self::ReadInt,
            "read_float" => Self::ReadFloat,

            // ═══════════════════════════════════════════════════════════════════
            // ByteSil
            // ═══════════════════════════════════════════════════════════════════
            "bytesil_new" => Self::BytesilNew,
            "bytesil_from_complex" => Self::BytesilFromComplex,
            "bytesil_to_complex" => Self::BytesilToComplex,
            "bytesil_null" => Self::BytesilNull,
            "bytesil_one" => Self::BytesilOne,
            "bytesil_i" => Self::BytesilI,
            "bytesil_neg_one" => Self::BytesilNegOne,
            "bytesil_neg_i" => Self::BytesilNegI,
            "bytesil_max" => Self::BytesilMax,
            "bytesil_mul" => Self::BytesilMul,
            "bytesil_div" => Self::BytesilDiv,
            "bytesil_pow" => Self::BytesilPow,
            "bytesil_root" => Self::BytesilRoot,
            "bytesil_inv" => Self::BytesilInv,
            "bytesil_conj" => Self::BytesilConj,
            "bytesil_xor" => Self::BytesilXor,
            "bytesil_mix" => Self::BytesilMix,
            "bytesil_rho" => Self::BytesilRho,
            "bytesil_theta" => Self::BytesilTheta,
            "bytesil_magnitude" => Self::BytesilMagnitude,
            "bytesil_phase_degrees" => Self::BytesilPhaseDegrees,
            "bytesil_phase_radians" => Self::BytesilPhaseRadians,
            "bytesil_is_null" => Self::BytesilIsNull,
            "bytesil_is_real" => Self::BytesilIsReal,
            "bytesil_is_imaginary" => Self::BytesilIsImaginary,
            "bytesil_norm" => Self::BytesilNorm,
            "bytesil_from_u8" => Self::BytesilFromU8,
            "bytesil_to_u8" => Self::BytesilToU8,
            "bytesil_add" => Self::BytesilAdd,
            "bytesil_sub" => Self::BytesilSub,
            "bytesil_scale" => Self::BytesilScale,
            "bytesil_rotate" => Self::BytesilRotate,

            // ═══════════════════════════════════════════════════════════════════
            // State
            // ═══════════════════════════════════════════════════════════════════
            "state_vacuum" => Self::StateVacuum,
            "state_neutral" => Self::StateNeutral,
            "state_max" | "state_maximum" => Self::StateMax,
            "state_from_bytes" => Self::StateFromBytes,
            "state_to_bytes" => Self::StateToBytes,
            "state_get_layer" => Self::StateGetLayer,
            "state_set_layer" => Self::StateSetLayer,
            "state_xor" => Self::StateXor,
            "state_fold" => Self::StateFold,
            "state_rotate" => Self::StateRotate,
            "state_collapse" | "state_collapse_xor" => Self::StateCollapse,
            "state_count_active_layers" => Self::StateCountActiveLayers,
            "state_count_null_layers" => Self::StateCountNullLayers,
            "state_is_vacuum" => Self::StateIsVacuum,
            "state_is_neutral" => Self::StateIsNeutral,
            "state_hash" => Self::StateHash,
            "state_compare" => Self::StateCompare,
            "state_equals" => Self::StateEquals,
            "state_tensor" => Self::StateTensor,
            "state_project" => Self::StateProject,
            "state_normalize" => Self::StateNormalize,
            "state_collapse_mag" => Self::StateCollapseMag,
            "state_collapse_sum" => Self::StateCollapseSum,
            "state_collapse_first" => Self::StateCollapseFirst,
            "state_collapse_last" => Self::StateCollapseLast,

            // ═══════════════════════════════════════════════════════════════════
            // Math - Basic
            // ═══════════════════════════════════════════════════════════════════
            "math_sin" | "sin" => Self::MathSin,
            "math_cos" | "cos" => Self::MathCos,
            "math_tan" | "tan" => Self::MathTan,
            "math_sqrt" | "sqrt" => Self::MathSqrt,
            "math_pow" | "pow" => Self::MathPow,
            "math_log" | "log" => Self::MathLog,
            "math_exp" | "exp" => Self::MathExp,
            "math_abs" | "abs" => Self::MathAbs,
            "math_floor" | "floor" => Self::MathFloor,
            "math_ceil" | "ceil" => Self::MathCeil,
            "math_round" | "round" => Self::MathRound,
            "math_min" | "min" => Self::MathMin,
            "math_max" | "max" => Self::MathMax,

            // Math - Constants
            "pi" => Self::MathPi,
            "tau" => Self::MathTau,
            "e" => Self::MathE,
            "phi" => Self::MathPhi,

            // Math - Inverse Trig
            "asin" => Self::MathAsin,
            "acos" => Self::MathAcos,
            "atan" => Self::MathAtan,
            "atan2" => Self::MathAtan2,

            // Math - Logarithms
            "ln" => Self::MathLn,
            "log10" => Self::MathLog10,
            "log2" => Self::MathLog2,

            // Math - Type-specific
            "abs_int" => Self::MathAbsInt,
            "abs_float" => Self::MathAbsFloat,
            "min_int" => Self::MathMinInt,
            "max_int" => Self::MathMaxInt,
            "clamp_int" => Self::MathClampInt,
            "pow_float" => Self::MathPowFloat,

            // ═══════════════════════════════════════════════════════════════════
            // String
            // ═══════════════════════════════════════════════════════════════════
            "string_len" | "string_length" => Self::StringLen,
            "string_concat" => Self::StringConcat,
            "string_substr" => Self::StringSubstr,
            "string_slice" => Self::StringSlice,
            "string_contains" => Self::StringContains,
            "string_replace" => Self::StringReplace,
            "string_split" => Self::StringSplit,
            "string_trim" => Self::StringTrim,
            "string_to_upper" => Self::StringToUpper,
            "string_to_lower" => Self::StringToLower,
            "string_starts_with" => Self::StringStartsWith,
            "string_ends_with" => Self::StringEndsWith,
            "int_to_string" => Self::IntToString,
            "float_to_string" => Self::FloatToString,
            "string_to_int" => Self::StringToInt,
            "string_to_float" => Self::StringToFloat,

            // Type conversions
            "float_from_int" => Self::FloatFromInt,
            "int_from_float" => Self::IntFromFloat,
            "float_to_int" => Self::FloatToInt,
            "int_to_float" => Self::IntToFloat,

            // ═══════════════════════════════════════════════════════════════════
            // Transform
            // ═══════════════════════════════════════════════════════════════════
            "apply_feedback" => Self::ApplyFeedback,
            "detect_emergence" => Self::DetectEmergence,
            "transform_phase_shift" => Self::TransformPhaseShift,
            "transform_magnitude_scale" => Self::TransformMagnitudeScale,
            "transform_identity" => Self::TransformIdentity,
            "transform_layer_swap" => Self::TransformLayerSwap,
            "transform_xor_layers" => Self::TransformXorLayers,
            "shift_layers_up" => Self::ShiftLayersUp,
            "shift_layers_down" => Self::ShiftLayersDown,
            "normalize_perception" => Self::NormalizePerception,

            // ═══════════════════════════════════════════════════════════════════
            // HTTP
            // ═══════════════════════════════════════════════════════════════════
            "http_get" => Self::HttpGet,
            "http_get_with_status" => Self::HttpGetWithStatus,
            "http_post" => Self::HttpPost,
            "http_post_json" => Self::HttpPostJson,
            "http_put" => Self::HttpPut,
            "http_put_json" => Self::HttpPutJson,
            "http_delete" => Self::HttpDelete,
            "http_patch" => Self::HttpPatch,
            "http_patch_json" => Self::HttpPatchJson,
            "http_head" => Self::HttpHead,
            "http_get_with_header" => Self::HttpGetWithHeader,
            "http_get_auth" => Self::HttpGetAuth,
            "http_post_auth" => Self::HttpPostAuth,
            "url_encode" => Self::UrlEncode,
            "url_decode" => Self::UrlDecode,
            "http_status_ok" => Self::HttpStatusOk,
            "http_status_text" => Self::HttpStatusText,

            // ═══════════════════════════════════════════════════════════════════
            // Layer Functions
            // ═══════════════════════════════════════════════════════════════════
            "fuse_vision_audio" => Self::FuseVisionAudio,
            "fuse_multimodal" => Self::FuseMultimodal,
            "rotate_layers" => Self::RotateLayers,
            "spread_to_group" => Self::SpreadToGroup,
            "emergence_pattern" => Self::EmergencePattern,
            "autopoietic_loop" => Self::AutopoieticLoop,

            // ═══════════════════════════════════════════════════════════════════
            // Complex Math
            // ═══════════════════════════════════════════════════════════════════
            "complex_add" => Self::ComplexAdd,
            "complex_sub" => Self::ComplexSub,
            "complex_scale" => Self::ComplexScale,
            "complex_rotate" => Self::ComplexRotate,
            "complex_lerp" => Self::ComplexLerp,
            "degrees_to_radians" => Self::DegreesToRadians,
            "radians_to_degrees" => Self::RadiansToDegrees,
            "clamp_float" => Self::ClampFloat,
            "min_float" => Self::MinFloat,
            "max_float" => Self::MaxFloat,
            "sign_float" => Self::SignFloat,
            "sign_int" => Self::SignInt,

            // ═══════════════════════════════════════════════════════════════════
            // Energy & Benchmark Metrics
            // ═══════════════════════════════════════════════════════════════════
            "energy_begin" => Self::EnergyBegin,
            "energy_end_joules" => Self::EnergyEndJoules,
            "energy_end_watts" => Self::EnergyEndWatts,
            "energy_end_samples_per_joule" => Self::EnergyEndSamplesPerJoule,
            "carbon_footprint_grams" => Self::CarbonFootprintGrams,
            "timestamp_nanos_low" => Self::TimestampNanosLow,
            "timestamp_nanos_high" => Self::TimestampNanosHigh,
            "benchmark_start" => Self::BenchmarkStart,
            "benchmark_end" => Self::BenchmarkEnd,
            "flop_count" => Self::FlopCount,
            "mac_count" => Self::MacCount,
            "throughput_samples_per_sec" => Self::ThroughputSamplesPerSec,
            "latency_micros" => Self::LatencyMicros,
            "memory_peak_bytes" => Self::MemoryPeakBytes,
            "accuracy_per_watt" => Self::AccuracyPerWatt,
            "energy_delay_product" => Self::EnergyDelayProduct,

            // ═══════════════════════════════════════════════════════════════════
            // Debug
            // ═══════════════════════════════════════════════════════════════════
            "debug_print" => Self::DebugPrint,
            "assert_eq" => Self::AssertEq,
            "assert_true" | "assert" => Self::AssertTrue,
            "assert_false" => Self::AssertFalse,
            "trace_state" => Self::TraceState,
            "timestamp_millis" => Self::TimestampMillis,
            "timestamp_micros" => Self::TimestampMicros,
            "sleep_millis" => Self::SleepMillis,
            "memory_used" => Self::MemoryUsed,
            "assert_eq_int" => Self::AssertEqInt,
            "assert_eq_bytesil" => Self::AssertEqBytesil,
            "assert_eq_state" => Self::AssertEqState,
            "timestamp_nanos_state" => Self::TimestampNanosState,
            "energy_total_joules" => Self::EnergyTotalJoules,
            "energy_efficiency" => Self::EnergyEfficiency,
            "benchmark_reset" => Self::BenchmarkReset,

            _ => return None,
        })
    }

    /// Check if a name is a stdlib intrinsic
    pub fn is_intrinsic(name: &str) -> bool {
        Self::from_name(name).is_some()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TOKENS
// ═══════════════════════════════════════════════════════════════════════════════

/// Token do lexer
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// Mnemônico de instrução
    Mnemonic(String),
    /// Registrador R0-R15
    Register(u8),
    /// Camada L0-LF
    Layer(u8),
    /// Número imediato
    Immediate(i64),
    /// Literal hexadecimal
    Hex(u64),
    /// Label (nome:)
    Label(String),
    /// Referência a label
    LabelRef(String),
    /// Diretiva (.mode, .data, etc)
    Directive(String),
    /// String literal
    StringLit(String),
    /// Vírgula
    Comma,
    /// Abre colchetes
    LBracket,
    /// Fecha colchetes
    RBracket,
    /// Nova linha
    Newline,
    /// Fim de arquivo
    Eof,
}

/// Posição no código fonte
#[derive(Debug, Clone, Copy, Default)]
pub struct SourcePos {
    pub line: u32,
    pub column: u32,
}

impl std::fmt::Display for SourcePos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// LEXER
// ═══════════════════════════════════════════════════════════════════════════════

/// Lexer para assembly SIL
pub struct Lexer<'a> {
    _input: &'a str,
    chars: std::iter::Peekable<std::str::Chars<'a>>,
    pos: SourcePos,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            _input: input,
            chars: input.chars().peekable(),
            pos: SourcePos { line: 1, column: 1 },
        }
    }
    
    fn advance(&mut self) -> Option<char> {
        let c = self.chars.next()?;
        if c == '\n' {
            self.pos.line += 1;
            self.pos.column = 1;
        } else {
            self.pos.column += 1;
        }
        Some(c)
    }
    
    fn peek(&mut self) -> Option<&char> {
        self.chars.peek()
    }
    
    fn skip_whitespace(&mut self) {
        while let Some(&c) = self.peek() {
            if c == ' ' || c == '\t' || c == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }
    
    fn skip_comment(&mut self) {
        while let Some(&c) = self.peek() {
            if c == '\n' {
                break;
            }
            self.advance();
        }
    }
    
    fn read_identifier(&mut self, first: char) -> String {
        let mut s = String::new();
        s.push(first);
        while let Some(&c) = self.peek() {
            if c.is_alphanumeric() || c == '_' || c == '.' {
                s.push(c);
                self.advance();
            } else {
                break;
            }
        }
        s
    }
    
    fn read_number(&mut self, first: char) -> Token {
        let mut s = String::new();
        s.push(first);
        
        // Verificar se é hex
        if first == '0' {
            if let Some(&'x') | Some(&'X') = self.peek() {
                self.advance();
                // Ler hex
                while let Some(&c) = self.peek() {
                    if c.is_ascii_hexdigit() {
                        s.push(c);
                        self.advance();
                    } else {
                        break;
                    }
                }
                let hex = u64::from_str_radix(&s[1..], 16).unwrap_or(0);
                return Token::Hex(hex);
            }
        }
        
        // Decimal
        while let Some(&c) = self.peek() {
            if c.is_ascii_digit() {
                s.push(c);
                self.advance();
            } else {
                break;
            }
        }
        Token::Immediate(s.parse().unwrap_or(0))
    }
    
    fn read_string(&mut self) -> String {
        let mut s = String::new();
        while let Some(c) = self.advance() {
            if c == '"' {
                break;
            }
            if c == '\\' {
                if let Some(esc) = self.advance() {
                    match esc {
                        'n' => s.push('\n'),
                        't' => s.push('\t'),
                        'r' => s.push('\r'),
                        '\\' => s.push('\\'),
                        '"' => s.push('"'),
                        _ => s.push(esc),
                    }
                }
            } else {
                s.push(c);
            }
        }
        s
    }
    
    pub fn next_token(&mut self) -> (Token, SourcePos) {
        self.skip_whitespace();
        
        let pos = self.pos;
        
        let c = match self.advance() {
            Some(c) => c,
            None => return (Token::Eof, pos),
        };
        
        let token = match c {
            ';' => {
                self.skip_comment();
                return self.next_token();
            }
            '\n' => Token::Newline,
            ',' => Token::Comma,
            '[' => Token::LBracket,
            ']' => Token::RBracket,
            '"' => Token::StringLit(self.read_string()),
            '.' => {
                let name = self.read_identifier(c);
                Token::Directive(name)
            }
            'L' | 'l' if self.peek().map(|c| c.is_ascii_hexdigit() && !c.is_ascii_lowercase()).unwrap_or(false) => {
                // Camada L0-L9 ou LA-LF (apenas hexdigits maiúsculos ou números)
                let c = self.peek().cloned();
                if let Some(c) = c {
                    if c.is_ascii_digit() || c.is_ascii_uppercase() {
                        self.advance();
                        let layer = u8::from_str_radix(&c.to_string(), 16).unwrap_or(0);
                        // Verificar se próximo caractere NÃO é alfanumérico (para não confundir com labels)
                        if self.peek().map(|c| c.is_alphanumeric() || *c == '_').unwrap_or(false) {
                            // É um identificador como LSTATE, LOAD, etc
                            let rest = self.read_identifier(c);
                            let name = format!("L{}", rest);
                            if let Some(&':') = self.peek() {
                                self.advance();
                                Token::Label(name)
                            } else if is_mnemonic(&name.to_uppercase()) {
                                Token::Mnemonic(name.to_uppercase())
                            } else {
                                Token::LabelRef(name)
                            }
                        } else {
                            Token::Layer(layer)
                        }
                    } else {
                        // Começa com letra minúscula após L - é identificador
                        let rest = self.read_identifier(c);
                        let name = format!("L{}", rest);
                        if let Some(&':') = self.peek() {
                            self.advance();
                            Token::Label(name)
                        } else if is_mnemonic(&name.to_uppercase()) {
                            Token::Mnemonic(name.to_uppercase())
                        } else {
                            Token::LabelRef(name)
                        }
                    }
                } else {
                    Token::LabelRef("L".to_string())
                }
            }
            'R' | 'r' if self.peek().map(|c| c.is_ascii_hexdigit() && !c.is_ascii_lowercase()).unwrap_or(false) => {
                // Registrador R0-R9 ou RA-RF (apenas hexdigits maiúsculos ou números)
                let c = self.peek().cloned();
                if let Some(c) = c {
                    if c.is_ascii_digit() || c.is_ascii_uppercase() {
                        self.advance();
                        let reg = u8::from_str_radix(&c.to_string(), 16).unwrap_or(0);
                        // Verificar se próximo caractere NÃO é alfanumérico
                        if self.peek().map(|c| c.is_alphanumeric() || *c == '_').unwrap_or(false) {
                            // É um identificador
                            let rest = self.read_identifier(c);
                            let name = format!("R{}", rest);
                            if let Some(&':') = self.peek() {
                                self.advance();
                                Token::Label(name)
                            } else if is_mnemonic(&name.to_uppercase()) {
                                Token::Mnemonic(name.to_uppercase())
                            } else {
                                Token::LabelRef(name)
                            }
                        } else {
                            Token::Register(reg)
                        }
                    } else {
                        // Começa com letra minúscula - é identificador  
                        let rest = self.read_identifier(c);
                        let name = format!("R{}", rest);
                        if let Some(&':') = self.peek() {
                            self.advance();
                            Token::Label(name)
                        } else if is_mnemonic(&name.to_uppercase()) {
                            Token::Mnemonic(name.to_uppercase())
                        } else {
                            Token::LabelRef(name)
                        }
                    }
                } else {
                    Token::LabelRef("R".to_string())
                }
            }
            c if c.is_alphabetic() || c == '_' => {
                let name = self.read_identifier(c);
                // Verificar se é label
                if let Some(&':') = self.peek() {
                    self.advance();
                    Token::Label(name)
                } else {
                    // Pode ser mnemônico ou referência
                    let upper = name.to_uppercase();
                    if is_mnemonic(&upper) {
                        Token::Mnemonic(upper)
                    } else {
                        Token::LabelRef(name)
                    }
                }
            }
            c if c.is_ascii_digit() => self.read_number(c),
            '-' => {
                if let Some(&c) = self.peek() {
                    if c.is_ascii_digit() {
                        let c = self.advance().unwrap();
                        if let Token::Immediate(n) = self.read_number(c) {
                            Token::Immediate(-n)
                        } else {
                            Token::Immediate(0)
                        }
                    } else {
                        Token::Immediate(0)
                    }
                } else {
                    Token::Immediate(0)
                }
            }
            _ => return self.next_token(),
        };
        
        (token, pos)
    }
    
    pub fn tokenize(&mut self) -> Vec<(Token, SourcePos)> {
        let mut tokens = Vec::new();
        loop {
            let (tok, pos) = self.next_token();
            let is_eof = tok == Token::Eof;
            tokens.push((tok, pos));
            if is_eof {
                break;
            }
        }
        tokens
    }
}

fn is_mnemonic(s: &str) -> bool {
    matches!(s,
        "NOP" | "HLT" | "RET" | "YIELD" |
        "JMP" | "JZ" | "JN" | "JC" | "JO" | "CALL" | "LOOP" |
        "MOV" | "MOVI" | "LOAD" | "STORE" | "PUSH" | "POP" | "XCHG" | "LSTATE" | "SSTATE" |
        "MUL" | "DIV" | "POW" | "ROOT" | "INV" | "CONJ" | "ADD" | "SUB" |
        "MAG" | "PHASE" | "SCALE" | "ROTATE" |
        // Int/Float mode-aware ops
        "ADDINT" | "SUBINT" | "MULINT" | "DIVINT" | "MODINT" | "POWINT" | "NEGINT" | "ABSINT" |
        "ADDFLOAT" | "SUBFLOAT" | "MULFLOAT" | "DIVFLOAT" | "POWFLOAT" | "SQRTFLOAT" | "NEGFLOAT" | "ABSFLOAT" | "FLOORFLOAT" | "CEILFLOAT" |
        // Comparação
        "CMPINT" | "CMPFLOAT" | "TESTINT" |
        // Bitwise inteiros
        "ANDINT" | "ORINT" | "XORINT" | "NOTINT" | "SHLINT" | "SHRINT" |
        // Conversões
        "CVTI2F" | "CVTF2I" | "CVTI2B" | "CVTB2I" | "CVTF2B" | "CVTB2F" |
        // Layer ops
        "XORL" | "ANDL" | "ORL" | "NOTL" | "SHIFTL" | "ROTATL" | "FOLD" | "SPREAD" | "GATHER" |
        "TRANS" | "PIPE" | "LERP" | "SLERP" | "GRAD" | "DESCENT" | "EMERGE" | "COLLAPSE" |
        "SETMODE" | "PROMOTE" | "DEMOTE" | "TRUNCATE" | "XORDEM" | "AVGDEM" | "MAXDEM" | "COMPAT" |
        // Quantum/BitDeSil
        "BIT.H" | "BIT.X" | "BIT.Y" | "BIT.Z" | "BIT.COLLAPSE" | "BIT.MEASURE" | "BIT.ROTQ" | "BIT.NORM" |
        "IN" | "OUT" | "SENSE" | "ACT" | "SYNC" | "BROADCAST" | "RECEIVE" | "ENTANGLE" |
        "HINT.CPU" | "HINT.GPU" | "HINT.NPU" | "HINT.ANY" | "HINT.FPGA" | "HINT.DSP" | "BATCH" | "UNBATCH" | "PREFETCH" | "FENCE" |
        "SYSCALL"
    )
}

// ═══════════════════════════════════════════════════════════════════════════════
// PARSER
// ═══════════════════════════════════════════════════════════════════════════════

/// Instrução parseada
#[derive(Debug, Clone)]
pub struct ParsedInstruction {
    pub opcode: Opcode,
    pub operands: Vec<Operand>,
    pub pos: SourcePos,
}

/// Operando
#[derive(Debug, Clone)]
pub enum Operand {
    Register(u8),
    Immediate(i64),
    LabelRef(String),
    Address(u32),
}

/// Statement parseado
#[derive(Debug, Clone)]
pub enum Statement {
    Label(String),
    Instruction(ParsedInstruction),
    Directive(Directive),
}

/// Diretiva
#[derive(Debug, Clone)]
pub enum Directive {
    Mode(String),
    Data,
    Code,
    Global(String),
    Extern(String),
    Align(u32),
    Byte(Vec<u8>),
    State(String, Vec<u8>),
    /// String literal (stored in global string table for runtime)
    StringLit(String),
}

/// Parser para assembly SIL
pub struct Parser {
    tokens: Vec<(Token, SourcePos)>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<(Token, SourcePos)>) -> Self {
        Self { tokens, pos: 0 }
    }
    
    fn current(&self) -> &Token {
        &self.tokens.get(self.pos).map(|(t, _)| t).unwrap_or(&Token::Eof)
    }
    
    fn current_pos(&self) -> SourcePos {
        self.tokens.get(self.pos).map(|(_, p)| *p).unwrap_or_default()
    }
    
    fn advance(&mut self) -> &Token {
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        self.current()
    }
    
    fn skip_newlines(&mut self) {
        while matches!(self.current(), Token::Newline) {
            self.advance();
        }
    }

    fn parse_operand(&mut self) -> VspResult<Operand> {
        let pos = self.current_pos();
        match self.current().clone() {
            Token::Register(r) => {
                self.advance();
                Ok(Operand::Register(r))
            }
            Token::Layer(l) => {
                self.advance();
                // Camada é tratada como imediato (0-15)
                Ok(Operand::Immediate(l as i64))
            }
            Token::Immediate(n) => {
                self.advance();
                Ok(Operand::Immediate(n))
            }
            Token::Hex(n) => {
                self.advance();
                Ok(Operand::Immediate(n as i64))
            }
            Token::LabelRef(name) => {
                self.advance();
                Ok(Operand::LabelRef(name))
            }
            _ => Err(VspError::AssemblerError(format!(
                "Expected operand at {}", pos
            ))),
        }
    }
    
    fn parse_instruction(&mut self, mnemonic: &str) -> VspResult<ParsedInstruction> {
        let pos = self.current_pos();
        let opcode = mnemonic_to_opcode(mnemonic)
            .ok_or_else(|| VspError::AssemblerError(format!(
                "Unknown mnemonic '{}' at {}", mnemonic, pos
            )))?;
        
        let format = opcode.format();
        let mut operands = Vec::new();
        
        // Parse operands based on format
        match format {
            InstructionFormat::FormatA => {
                // No operands
            }
            InstructionFormat::FormatB => {
                // 1 register
                if matches!(self.current(), Token::Register(_) | Token::LabelRef(_) | Token::Immediate(_)) {
                    operands.push(self.parse_operand()?);
                }
            }
            InstructionFormat::FormatC => {
                // 2 operands
                if !matches!(self.current(), Token::Newline | Token::Eof) {
                    operands.push(self.parse_operand()?);
                    if matches!(self.current(), Token::Comma) {
                        self.advance();
                        operands.push(self.parse_operand()?);
                    }
                }
            }
            InstructionFormat::FormatD => {
                // Up to 3 operands
                while !matches!(self.current(), Token::Newline | Token::Eof) {
                    operands.push(self.parse_operand()?);
                    if matches!(self.current(), Token::Comma) {
                        self.advance();
                    } else {
                        break;
                    }
                }
            }
        }
        
        Ok(ParsedInstruction { opcode, operands, pos })
    }
    
    fn parse_directive(&mut self, name: &str) -> VspResult<Directive> {
        match name.to_lowercase().as_str() {
            ".mode" => {
                if let Token::LabelRef(mode) | Token::Mnemonic(mode) = self.current().clone() {
                    self.advance();
                    Ok(Directive::Mode(mode))
                } else {
                    Ok(Directive::Mode("SIL-128".to_string()))
                }
            }
            ".data" => Ok(Directive::Data),
            ".code" | ".text" => Ok(Directive::Code),
            ".global" | ".globl" => {
                if let Token::LabelRef(name) = self.current().clone() {
                    self.advance();
                    Ok(Directive::Global(name))
                } else {
                    Err(VspError::AssemblerError("Expected label name".to_string()))
                }
            }
            ".extern" => {
                if let Token::LabelRef(name) = self.current().clone() {
                    self.advance();
                    Ok(Directive::Extern(name))
                } else {
                    Err(VspError::AssemblerError("Expected symbol name".to_string()))
                }
            }
            ".align" => {
                if let Token::Immediate(n) = self.current().clone() {
                    self.advance();
                    Ok(Directive::Align(n as u32))
                } else {
                    Ok(Directive::Align(4))
                }
            }
            ".byte" => {
                let mut bytes = Vec::new();
                while matches!(self.current(), Token::Immediate(_) | Token::Hex(_)) {
                    match self.current().clone() {
                        Token::Immediate(n) => bytes.push(n as u8),
                        Token::Hex(n) => bytes.push(n as u8),
                        _ => break,
                    }
                    self.advance();
                    if matches!(self.current(), Token::Comma) {
                        self.advance();
                    }
                }
                Ok(Directive::Byte(bytes))
            }
            ".state" => {
                // .state 0xHHHHHHHHHHHHHHHH (16 bytes como hex)
                let mut bytes = Vec::new();
                if let Token::Hex(n) = self.current().clone() {
                    self.advance();
                    // Converter o número hex para 16 bytes (little-endian)
                    let value = n as u64;
                    bytes.extend_from_slice(&value.to_le_bytes());
                    // Padding para 16 bytes (SilState completo)
                    bytes.extend_from_slice(&[0u8; 8]);
                } else {
                    // Default: 16 bytes zerados
                    bytes.extend_from_slice(&[0u8; 16]);
                }
                Ok(Directive::State(String::new(), bytes))
            }
            ".string" => {
                // .string "literal text"
                if let Token::StringLit(s) = self.current().clone() {
                    self.advance();
                    Ok(Directive::StringLit(s))
                } else {
                    Err(VspError::AssemblerError("Expected string literal after .string".to_string()))
                }
            }
            _ => Err(VspError::AssemblerError(format!(
                "Unknown directive '{}'", name
            ))),
        }
    }
    
    pub fn parse(&mut self) -> VspResult<Vec<Statement>> {
        let mut statements = Vec::new();
        
        loop {
            self.skip_newlines();
            
            match self.current().clone() {
                Token::Eof => break,
                Token::Label(name) => {
                    self.advance();
                    statements.push(Statement::Label(name));
                }
                Token::Mnemonic(m) => {
                    self.advance();
                    let instr = self.parse_instruction(&m)?;
                    statements.push(Statement::Instruction(instr));
                }
                Token::Directive(d) => {
                    self.advance();
                    let dir = self.parse_directive(&d)?;
                    statements.push(Statement::Directive(dir));
                }
                Token::LabelRef(name) => {
                    // Pode ser `identifier .directive value` na seção de dados
                    self.advance();
                    if let Token::Directive(d) = self.current().clone() {
                        self.advance();
                        // Criar label para o identificador
                        statements.push(Statement::Label(name));
                        // Parsear diretiva
                        let dir = self.parse_directive(&d)?;
                        statements.push(Statement::Directive(dir));
                    }
                    // Senão, ignorar (já avançamos)
                }
                _ => {
                    self.advance();
                }
            }
        }
        
        Ok(statements)
    }
}

fn mnemonic_to_opcode(m: &str) -> Option<Opcode> {
    Some(match m {
        // Controle
        "NOP" => Opcode::Nop,
        "HLT" => Opcode::Hlt,
        "RET" => Opcode::Ret,
        "YIELD" => Opcode::Yield,
        "JMP" => Opcode::Jmp,
        "JZ" => Opcode::Jz,
        "JN" => Opcode::Jn,
        "JC" => Opcode::Jc,
        "JO" => Opcode::Jo,
        "CALL" => Opcode::Call,
        "LOOP" => Opcode::Loop,

        // Movimento de dados
        "MOV" => Opcode::Mov,
        "MOVI" => Opcode::Movi,
        "LOAD" => Opcode::Load,
        "STORE" => Opcode::Store,
        "PUSH" => Opcode::Push,
        "POP" => Opcode::Pop,
        "XCHG" => Opcode::Xchg,
        "LSTATE" => Opcode::Lstate,
        "SSTATE" => Opcode::Sstate,

        // Aritmética ByteSil
        "MUL" => Opcode::Mul,
        "DIV" => Opcode::Div,
        "POW" => Opcode::Pow,
        "ROOT" => Opcode::Root,
        "INV" => Opcode::Inv,
        "CONJ" => Opcode::Conj,
        "ADD" => Opcode::Add,
        "SUB" => Opcode::Sub,
        "MAG" => Opcode::Mag,
        "PHASE" => Opcode::Phase,
        "SCALE" => Opcode::Scale,
        "ROTATE" => Opcode::Rotate,

        // Aritmética de inteiros (mode-aware)
        "ADDINT" => Opcode::AddInt,
        "SUBINT" => Opcode::SubInt,
        "MULINT" => Opcode::MulInt,
        "DIVINT" => Opcode::DivInt,
        "MODINT" => Opcode::ModInt,
        "POWINT" => Opcode::PowInt,
        "NEGINT" => Opcode::NegInt,
        "ABSINT" => Opcode::AbsInt,

        // Aritmética de floats (mode-aware)
        "ADDFLOAT" => Opcode::AddFloat,
        "SUBFLOAT" => Opcode::SubFloat,
        "MULFLOAT" => Opcode::MulFloat,
        "DIVFLOAT" => Opcode::DivFloat,
        "POWFLOAT" => Opcode::PowFloat,
        "SQRTFLOAT" => Opcode::SqrtFloat,
        "NEGFLOAT" => Opcode::NegFloat,
        "ABSFLOAT" => Opcode::AbsFloat,
        "FLOORFLOAT" => Opcode::FloorFloat,
        "CEILFLOAT" => Opcode::CeilFloat,

        // Comparação
        "CMPINT" => Opcode::CmpInt,
        "CMPFLOAT" => Opcode::CmpFloat,
        "TESTINT" => Opcode::TestInt,

        // Bitwise de inteiros
        "ANDINT" => Opcode::AndInt,
        "ORINT" => Opcode::OrInt,
        "XORINT" => Opcode::XorInt,
        "NOTINT" => Opcode::NotInt,
        "SHLINT" => Opcode::ShlInt,
        "SHRINT" => Opcode::ShrInt,

        // Conversões
        "CVTI2F" => Opcode::CvtIntToFloat,
        "CVTF2I" => Opcode::CvtFloatToInt,
        "CVTI2B" => Opcode::CvtIntToByteSil,
        "CVTB2I" => Opcode::CvtByteSilToInt,
        "CVTF2B" => Opcode::CvtFloatToByteSil,
        "CVTB2F" => Opcode::CvtByteSilToFloat,

        // Operações de camada
        "XORL" => Opcode::Xorl,
        "ANDL" => Opcode::Andl,
        "ORL" => Opcode::Orl,
        "NOTL" => Opcode::Notl,
        "SHIFTL" => Opcode::Shiftl,
        "ROTATL" => Opcode::Rotatl,
        "FOLD" => Opcode::Fold,
        "SPREAD" => Opcode::Spread,
        "GATHER" => Opcode::Gather,

        // Transformações
        "TRANS" => Opcode::Trans,
        "PIPE" => Opcode::Pipe,
        "LERP" => Opcode::Lerp,
        "SLERP" => Opcode::Slerp,
        "GRAD" => Opcode::Grad,
        "DESCENT" => Opcode::Descent,
        "EMERGE" => Opcode::Emerge,
        "COLLAPSE" => Opcode::Collapse,

        // Compatibilidade
        "SETMODE" => Opcode::Setmode,
        "PROMOTE" => Opcode::Promote,
        "DEMOTE" => Opcode::Demote,
        "TRUNCATE" => Opcode::Truncate,
        "XORDEM" => Opcode::Xordem,
        "AVGDEM" => Opcode::Avgdem,
        "MAXDEM" => Opcode::Maxdem,
        "COMPAT" => Opcode::Compat,

        // Quantum/BitDeSil
        "BIT.H" => Opcode::BitHadamard,
        "BIT.X" => Opcode::BitPauliX,
        "BIT.Y" => Opcode::BitPauliY,
        "BIT.Z" => Opcode::BitPauliZ,
        "BIT.COLLAPSE" => Opcode::BitCollapse,
        "BIT.MEASURE" => Opcode::BitMeasure,
        "BIT.ROTQ" => Opcode::BitRotateQ,
        "BIT.NORM" => Opcode::BitNormalize,

        // I/O e Sistema
        "IN" => Opcode::In,
        "OUT" => Opcode::Out,
        "SENSE" => Opcode::Sense,
        "ACT" => Opcode::Act,
        "SYNC" => Opcode::Sync,
        "BROADCAST" => Opcode::Broadcast,
        "RECEIVE" => Opcode::Receive,
        "ENTANGLE" => Opcode::Entangle,

        // Hardware Hints
        "HINT.CPU" => Opcode::HintCpu,
        "HINT.GPU" => Opcode::HintGpu,
        "HINT.NPU" => Opcode::HintNpu,
        "HINT.ANY" => Opcode::HintAny,
        "HINT.FPGA" => Opcode::HintFpga,
        "HINT.DSP" => Opcode::HintDsp,
        "BATCH" => Opcode::Batch,
        "UNBATCH" => Opcode::Unbatch,
        "PREFETCH" => Opcode::Prefetch,
        "FENCE" => Opcode::Fence,
        "SYSCALL" => Opcode::Syscall,

        _ => return None,
    })
}

// ═══════════════════════════════════════════════════════════════════════════════
// ASSEMBLER
// ═══════════════════════════════════════════════════════════════════════════════

/// Assembler SIL
pub struct Assembler {
    /// Tabela de símbolos
    symbols: HashMap<String, u32>,
    /// Símbolos globais
    globals: Vec<String>,
    /// Símbolos externos
    externs: Vec<String>,
    /// Código gerado
    code: Vec<u8>,
    /// Dados
    data: Vec<u8>,
    /// Referências não resolvidas (offset, label)
    unresolved: Vec<(usize, String, SourcePos)>,
    /// Modo SIL
    mode: SilMode,
    /// Seção atual
    section: Section,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Section {
    Code,
    Data,
}

impl Default for Assembler {
    fn default() -> Self {
        Self::new()
    }
}

impl Assembler {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            globals: Vec::new(),
            externs: Vec::new(),
            code: Vec::new(),
            data: Vec::new(),
            unresolved: Vec::new(),
            mode: SilMode::Sil128,
            section: Section::Code,
        }
    }
    
    /// Assembla código fonte para bytecode
    pub fn assemble(&mut self, source: &str) -> VspResult<SilcFile> {
        // Tokenize
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        
        // Parse
        let mut parser = Parser::new(tokens);
        let statements = parser.parse()?;
        
        // First pass: collect labels
        self.first_pass(&statements)?;
        
        // Second pass: generate code
        self.second_pass(&statements)?;
        
        // Resolve references
        self.resolve_references()?;
        
        // Build SILC file
        self.build_silc()
    }
    
    fn first_pass(&mut self, statements: &[Statement]) -> VspResult<()> {
        let mut offset = 0u32;
        self.section = Section::Code;
        
        for stmt in statements {
            match stmt {
                Statement::Label(name) => {
                    let addr = if self.section == Section::Code {
                        offset
                    } else {
                        self.data.len() as u32
                    };
                    self.symbols.insert(name.clone(), addr);
                }
                Statement::Instruction(instr) => {
                    if self.section == Section::Code {
                        offset += instr.opcode.format().size() as u32;
                    }
                }
                Statement::Directive(dir) => {
                    match dir {
                        Directive::Code => self.section = Section::Code,
                        Directive::Data => self.section = Section::Data,
                        Directive::Global(name) => self.globals.push(name.clone()),
                        Directive::Extern(name) => self.externs.push(name.clone()),
                        Directive::Mode(mode) => {
                            self.mode = match mode.to_uppercase().as_str() {
                                "SIL-8" => SilMode::Sil8,
                                "SIL-16" => SilMode::Sil16,
                                "SIL-32" => SilMode::Sil32,
                                "SIL-64" => SilMode::Sil64,
                                "SIL-128" | _ => SilMode::Sil128,
                            };
                        }
                        _ => {}
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn second_pass(&mut self, statements: &[Statement]) -> VspResult<()> {
        self.section = Section::Code;
        
        for stmt in statements {
            match stmt {
                Statement::Label(_) => {}
                Statement::Instruction(instr) => {
                    if self.section == Section::Code {
                        self.emit_instruction(instr)?;
                    }
                }
                Statement::Directive(dir) => {
                    self.emit_directive(dir)?;
                }
            }
        }
        
        Ok(())
    }
    
    fn emit_instruction(&mut self, instr: &ParsedInstruction) -> VspResult<()> {
        // Special handling for CALL with stdlib intrinsics
        if instr.opcode == Opcode::Call {
            if let Some(Operand::LabelRef(name)) = instr.operands.first() {
                if let Some(intrinsic) = StdlibIntrinsic::from_name(name) {
                    // Emit SYSCALL with intrinsic ID
                    self.code.push(Opcode::Syscall as u8);
                    self.code.push(intrinsic as u8);

                    // For print_string, include the string index in bytes 2-3
                    if intrinsic == StdlibIntrinsic::PrintString {
                        if let Some(Operand::Immediate(string_id)) = instr.operands.get(1) {
                            let id = *string_id as u16;
                            self.code.push((id & 0xFF) as u8);       // low byte
                            self.code.push(((id >> 8) & 0xFF) as u8); // high byte
                        } else {
                            self.code.push(0);
                            self.code.push(0);
                        }
                    } else {
                        self.code.push(0); // padding
                        self.code.push(0); // padding
                    }
                    return Ok(());
                }
            }
        }

        let opcode = instr.opcode as u8;
        let format = instr.opcode.format();

        match format {
            InstructionFormat::FormatA => {
                self.code.push(opcode);
            }
            InstructionFormat::FormatB => {
                self.code.push(opcode);
                let operand = self.encode_operand(instr.operands.first(), instr.pos)?;
                self.code.push(operand as u8);
            }
            InstructionFormat::FormatC => {
                self.code.push(opcode);
                
                // MOVI tem formato especial: reg + imm8
                if instr.opcode == Opcode::Movi {
                    let reg = self.encode_operand(instr.operands.first(), instr.pos)? as u8;
                    let imm = self.encode_operand(instr.operands.get(1), instr.pos)? as u8;
                    self.code.push(reg);
                    self.code.push(imm);
                } else {
                    let op1 = self.encode_operand(instr.operands.first(), instr.pos)?;
                    let op2 = self.encode_operand(instr.operands.get(1), instr.pos)?;
                    // Pack two 4-bit registers: reg_a in low bits, reg_b in high bits
                    self.code.push((op1 & 0x0F) as u8 | ((op2 & 0x0F) << 4) as u8);
                    self.code.push((op1 >> 4) as u8);
                }
            }
            InstructionFormat::FormatD => {
                self.code.push(opcode);

                // Encode operands
                match instr.operands.len() {
                    0 => {
                        self.code.extend_from_slice(&[0, 0, 0]);
                    }
                    1 => {
                        // Address or immediate
                        let value = self.encode_operand_32(instr.operands.first(), instr.pos)?;
                        let bytes = value.to_le_bytes();
                        self.code.extend_from_slice(&bytes[..3]);
                    }
                    2 => {
                        let op1 = self.encode_operand(instr.operands.first(), instr.pos)?;
                        let value = self.encode_operand_32(instr.operands.get(1), instr.pos)?;
                        self.code.push(op1 as u8);
                        self.code.extend_from_slice(&value.to_le_bytes()[..2]);
                    }
                    _ => {
                        let op1 = self.encode_operand(instr.operands.first(), instr.pos)?;
                        let op2 = self.encode_operand(instr.operands.get(1), instr.pos)?;
                        let op3 = self.encode_operand(instr.operands.get(2), instr.pos)?;
                        self.code.push(((op1 & 0x0F) << 4) as u8 | (op2 & 0x0F) as u8);
                        self.code.push(op3 as u8);
                        self.code.push(0);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn encode_operand(&mut self, operand: Option<&Operand>, pos: SourcePos) -> VspResult<u32> {
        match operand {
            Some(Operand::Register(r)) => Ok(*r as u32),
            Some(Operand::Immediate(n)) => Ok(*n as u32),
            Some(Operand::Address(a)) => Ok(*a),
            Some(Operand::LabelRef(name)) => {
                if let Some(&addr) = self.symbols.get(name) {
                    Ok(addr)
                } else {
                    // Mark for resolution
                    self.unresolved.push((self.code.len(), name.clone(), pos));
                    Ok(0)
                }
            }
            None => Ok(0),
        }
    }
    
    fn encode_operand_32(&mut self, operand: Option<&Operand>, pos: SourcePos) -> VspResult<u32> {
        match operand {
            Some(Operand::Register(r)) => Ok(*r as u32),
            Some(Operand::Immediate(n)) => Ok(*n as u32),
            Some(Operand::Address(a)) => Ok(*a),
            Some(Operand::LabelRef(name)) => {
                if let Some(&addr) = self.symbols.get(name) {
                    Ok(addr)
                } else {
                    // Mark for resolution
                    let offset = self.code.len();
                    self.unresolved.push((offset, name.clone(), pos));
                    Ok(0)
                }
            }
            None => Ok(0),
        }
    }
    
    fn emit_directive(&mut self, dir: &Directive) -> VspResult<()> {
        match dir {
            Directive::Code => self.section = Section::Code,
            Directive::Data => self.section = Section::Data,
            Directive::Byte(bytes) => {
                if self.section == Section::Data {
                    self.data.extend_from_slice(bytes);
                }
            }
            Directive::State(_, bytes) => {
                if self.section == Section::Data {
                    self.data.extend_from_slice(bytes);
                }
            }
            Directive::Align(n) => {
                let target = if self.section == Section::Code {
                    &mut self.code
                } else {
                    &mut self.data
                };
                let padding = (*n as usize - (target.len() % *n as usize)) % *n as usize;
                target.extend(std::iter::repeat(0).take(padding));
            }
            _ => {}
        }
        Ok(())
    }
    
    fn resolve_references(&mut self) -> VspResult<()> {
        for (offset, name, pos) in &self.unresolved {
            let addr = self.symbols.get(name)
                .ok_or_else(|| VspError::AssemblerError(format!(
                    "Undefined symbol '{}' at {}", name, pos
                )))?;

            // Patch the address
            let bytes = addr.to_le_bytes();
            if *offset + 2 < self.code.len() {
                self.code[*offset] = bytes[0];
                self.code[*offset + 1] = bytes[1];
                if *offset + 2 < self.code.len() {
                    self.code[*offset + 2] = bytes[2];
                }
            }
        }
        Ok(())
    }
    
    fn build_silc(&self) -> VspResult<SilcFile> {
        let mut builder = SilcBuilder::new()
            .mode(self.mode)
            .code(self.code.clone())
            .data(self.data.clone());
        
        // Add symbols
        for (name, &addr) in &self.symbols {
            let is_global = self.globals.contains(name);
            builder = builder.symbol(SymbolInfo {
                name: name.clone(),
                address: addr,
                is_global,
                is_extern: false,
            });
        }
        
        // Add external symbols
        for name in &self.externs {
            builder = builder.symbol(SymbolInfo {
                name: name.clone(),
                address: 0,
                is_global: false,
                is_extern: true,
            });
        }
        
        Ok(builder.build())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// FUNÇÕES DE CONVENIÊNCIA
// ═══════════════════════════════════════════════════════════════════════════════

/// Assembla código fonte para bytecode
pub fn assemble(source: &str) -> VspResult<SilcFile> {
    Assembler::new().assemble(source)
}

/// Assembla arquivo .sil para .silc
pub fn assemble_file(input_path: &std::path::Path) -> VspResult<SilcFile> {
    let source = std::fs::read_to_string(input_path)
        .map_err(|e| VspError::IoError(e.to_string()))?;
    assemble(&source)
}

// ═══════════════════════════════════════════════════════════════════════════════
// DISASSEMBLER
// ═══════════════════════════════════════════════════════════════════════════════

/// Disassembla bytecode para texto
pub fn disassemble(code: &[u8]) -> String {
    let mut output = String::new();
    let mut offset = 0;
    
    while offset < code.len() {
        let opcode_byte = code[offset];
        let opcode = match Opcode::from_byte(opcode_byte) {
            Some(op) => op,
            None => {
                output.push_str(&format!("{:04X}:  .byte 0x{:02X}\n", offset, opcode_byte));
                offset += 1;
                continue;
            }
        };
        
        let format = opcode.format();
        let size = format.size();
        
        output.push_str(&format!("{:04X}:  {:8}", offset, opcode.mnemonic()));
        
        match format {
            InstructionFormat::FormatA => {}
            InstructionFormat::FormatB => {
                if offset + 1 < code.len() {
                    output.push_str(&format!(" R{}", code[offset + 1] & 0x0F));
                }
            }
            InstructionFormat::FormatC => {
                if offset + 2 < code.len() {
                    // MOVI tem formato especial: reg + imm8
                    if opcode == Opcode::Movi {
                        let reg = code[offset + 1] & 0x0F;
                        let imm = code[offset + 2];
                        output.push_str(&format!(" R{:X}, 0x{:02X}", reg, imm));
                    } else {
                        let packed = code[offset + 1];
                        let r1 = (packed >> 4) & 0x0F;
                        let r2 = packed & 0x0F;
                        output.push_str(&format!(" R{:X}, R{:X}", r1, r2));
                    }
                }
            }
            InstructionFormat::FormatD => {
                if offset + 3 < code.len() {
                    let addr = u32::from_le_bytes([
                        code[offset + 1],
                        code[offset + 2],
                        code[offset + 3],
                        0,
                    ]);
                    output.push_str(&format!(" 0x{:06X}", addr));
                }
            }
        }
        
        output.push('\n');
        offset += size;
    }
    
    output
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTES
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lexer_basic() {
        let mut lexer = Lexer::new("NOP\nMOV R0, R1");
        let tokens = lexer.tokenize();
        
        assert!(matches!(tokens[0].0, Token::Mnemonic(ref s) if s == "NOP"));
        assert!(matches!(tokens[1].0, Token::Newline));
        assert!(matches!(tokens[2].0, Token::Mnemonic(ref s) if s == "MOV"));
        assert!(matches!(tokens[3].0, Token::Register(0)));
    }
    
    #[test]
    fn test_lexer_labels() {
        let mut lexer = Lexer::new("start:\n    JMP start");
        let tokens = lexer.tokenize();
        
        assert!(matches!(tokens[0].0, Token::Label(ref s) if s == "start"));
        assert!(matches!(tokens[2].0, Token::Mnemonic(ref s) if s == "JMP"));
        assert!(matches!(tokens[3].0, Token::LabelRef(ref s) if s == "start"));
    }
    
    #[test]
    fn test_assemble_simple() {
        let source = r#"
.mode SIL-128
.code
    NOP
    HLT
"#;
        let silc = assemble(source).unwrap();
        assert_eq!(silc.code.len(), 2);
        assert_eq!(silc.code[0], Opcode::Nop as u8);
        assert_eq!(silc.code[1], Opcode::Hlt as u8);
    }
    
    #[test]
    fn test_assemble_with_labels() {
        let source = r#"
.code
start:
    NOP
    JMP start
"#;
        let silc = assemble(source).unwrap();
        assert!(silc.symbols.iter().any(|s| s.name == "start"));
    }
    
    #[test]
    fn test_disassemble() {
        let code = vec![
            Opcode::Nop as u8,
            Opcode::Hlt as u8,
        ];
        let output = disassemble(&code);
        assert!(output.contains("NOP"));
        assert!(output.contains("HLT"));
    }
}
