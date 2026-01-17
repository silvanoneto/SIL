//! # Neural Network Architectures
//!
//! High-level architectures built from layers and primitives.
//!
//! - **Transformer**: Attention-based sequence models (encoder, decoder, SwiGLU)
//! - **KAN**: Kolmogorov-Arnold Networks (interpretable function approximation)
//! - **SSM/Mamba**: State Space Models with linear complexity
//! - **LNN**: Liquid Neural Networks for continuous temporal learning
//! - **Attention**: Scaled dot-product, multi-head, causal masking
//! - **Recurrent**: LSTM, GRU, vanilla RNN
//! - **SNN**: Spiking Neural Networks for neuromorphic hardware

pub mod attention;
pub mod kan;
pub mod lnn;
pub mod recurrent;
pub mod ssm;
pub mod snn;
pub mod transformer;

pub use attention::*;
pub use kan::*;
pub use lnn::*;
pub use recurrent::*;
pub use ssm::*;
pub use snn::*;
pub use transformer::*;
