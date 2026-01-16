//! # Machine Learning Standard Library for LIS
//!
//! Comprehensive ML toolkit optimized for edge deployment using ByteSil and State types.
//!
//! ## Overview
//!
//! This module provides ~125 intrinsic functions organized into categories:
//!
//! - **Core**: Activations, loss functions, linear algebra, statistics
//! - **Layers**: Dense, convolution, normalization, dropout
//! - **Architecture**: KAN, SSM/Mamba, LNN, attention, recurrent
//! - **Optimization**: SGD, Adam, learning rate schedulers
//! - **Edge/Hardware**: ρ_Sil metric, chromatic routing, device detection
//! - **Signal**: FFT, filters
//! - **Federated**: FedAvg, Byzantine detection
//! - **Emergent**: Hebbian learning, Kuramoto oscillators
//!
//! ## Key Features
//!
//! - **ByteSil**: O(1) complex arithmetic via log-polar representation
//! - **State**: Native 16-element vectors / 4x4 matrices
//! - **ρ_Sil**: Informational complexity metric for edge/cloud decisions
//! - **Hardware hints**: `@cpu`, `@gpu`, `@npu`, `@neuromorphic`
//!
//! ## Example
//!
//! ```lis
//! fn inference(input: State) {
//!     let weights = state_neutral();
//!     let bias = state_vacuum();
//!
//!     @gpu
//!     let hidden = dense_forward(input, weights, bias, "relu");
//!     let output = softmax(hidden);
//!     let class = argmax(output);
//! }
//! ```
//!
//! ## Whitepaper References
//!
//! - §C.18: KANs (Kolmogorov-Arnold Networks)
//! - §C.19: SSMs/Mamba
//! - §C.20: LNNs (Liquid Neural Networks)
//! - §C.33: ρ_Sil metric

// Utilities (helpers for ByteSil/State operations)
pub mod utils;

// Core modules
pub mod activations;
pub mod loss;
pub mod linalg;
pub mod stats;

// Layer modules
pub mod layers;
pub mod conv;
pub mod embedding;

// Architecture modules
pub mod attention;
pub mod recurrent;
pub mod transformer;
pub mod kan;
pub mod ssm;
pub mod lnn;
pub mod snn;

// Optimization modules
pub mod optim;
pub mod sparsity;
pub mod tensor;

// Edge/Hardware modules
pub mod hardware;
pub mod rho_sil;
pub mod chromatic;
pub mod offload;

// Signal processing
pub mod signal;

// Federated learning
pub mod federated;

// Emergent patterns
pub mod mimetics;
pub mod tda;

// Re-exports
pub use utils::{magnitude, phase, from_mag_phase};
pub use activations::*;
pub use loss::*;
pub use linalg::*;
pub use stats::*;
pub use layers::*;
pub use conv::*;
pub use embedding::*;
pub use attention::*;
pub use recurrent::*;
pub use transformer::*;
pub use kan::*;
pub use ssm::*;
pub use lnn::*;
pub use snn::*;
pub use optim::*;
pub use sparsity::*;
pub use tensor::*;
pub use hardware::*;
pub use rho_sil::*;
pub use chromatic::*;
pub use offload::*;
pub use signal::*;
pub use federated::*;
pub use mimetics::*;
pub use tda::*;
