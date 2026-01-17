//! # Core ML Primitives
//!
//! Fundamental operations for machine learning:
//! - Tensor utilities (ByteSil/SilState operations)
//! - Activation functions
//! - Loss functions
//! - Linear algebra
//! - Statistics
//! - Optimization
//! - Semantic layers (16-layer topology)
//! - Linear encoder/decoder (high fidelity)
//! - Native transform pipeline
//! - ML pipeline integration

pub mod tensor;
pub mod activations;
pub mod loss;
pub mod linalg;
pub mod stats;
pub mod optim;

// New ML-specific features
pub mod semantic_layers;
pub mod encoder;
pub mod transforms;
pub mod pipeline;

// Re-export everything
pub use tensor::*;
pub use activations::*;
pub use loss::*;
pub use linalg::*;
pub use stats::*;
pub use optim::*;

// New ML-specific exports
pub use semantic_layers::{SemanticLayer, SemanticLayerSet};
pub use encoder::{LinearEncoder, EncodingStrategy};
pub use transforms::{TransformPipeline, NativeTransform};
pub use pipeline::{MlPipeline, PipelineConfig};

/// Prelude module for core exports
pub mod prelude {
    pub use crate::core::tensor::*;
    pub use crate::core::activations::*;
    pub use crate::core::loss::*;
    pub use crate::core::linalg::*;
    pub use crate::core::stats::*;
    pub use crate::core::optim::*;
    pub use crate::core::semantic_layers::{SemanticLayer, SemanticLayerSet};
    pub use crate::core::encoder::{LinearEncoder, EncodingStrategy};
    pub use crate::core::transforms::{TransformPipeline, NativeTransform};
    pub use crate::core::pipeline::{MlPipeline, PipelineConfig};
}