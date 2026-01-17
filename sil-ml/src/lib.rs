//! # sil-ml - Machine Learning for SIL 16-layer Architecture
//!
//! Edge-distributed machine learning primitives and architectures for the SIL system.
//!
//! ## Features
//!
//! - **core**: ML primitives (tensors, activations, loss functions, linear algebra, statistics, optimization)
//! - **layers**: Neural network layers (dense, normalization, dropout)
//! - **edge**: Edge computing with ρ_Sil metrics and chromatic routing
//! - **arch**: Advanced architectures (transformers, KAN, SSM, LNN, SNN)
//! - **inference**: Inference engine with routing
//! - **distributed**: Federated learning and Byzantine-robust aggregation
//! - **emergence**: Hebbian learning and Kuramoto oscillators

pub mod error;
pub use error::{SilMlError, Result};

pub mod core;
pub use core::prelude::*;

pub mod layers;
pub use layers::*;

pub mod edge;
pub use edge::*;

#[cfg(feature = "arch")]
pub mod arch;

pub mod inference;
pub use inference::*;

pub mod distributed;
pub use distributed::*;

#[cfg(feature = "emergence")]
pub mod emergence;

/// Prelude module with common re-exports
pub mod prelude {
    pub use crate::error::{SilMlError, Result};
    pub use crate::core::prelude::*;
    pub use crate::layers::*;
    pub use crate::edge::*;
    pub use crate::inference::*;
    pub use crate::distributed::*;

    #[cfg(feature = "arch")]
    pub use crate::arch::*;

    #[cfg(feature = "emergence")]
    pub use crate::emergence::*;
}
