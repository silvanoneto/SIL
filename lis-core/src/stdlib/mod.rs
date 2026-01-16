//! Standard Library for LIS
//!
//! Contains intrinsic functions and utilities for the LIS language.
//! Provides access to ByteSil operations, State manipulation, mathematical
//! functions, I/O, string handling, layer operations, transforms, and debugging tools.

// Core modules
pub mod bytesil;
pub mod state;
pub mod math;
pub mod string;
pub mod layers;
pub mod transforms;
pub mod debug;

// I/O modules
#[cfg(feature = "jsil")]
pub mod io;
pub mod console;

// Network modules
#[cfg(feature = "http")]
pub mod http;

// Machine Learning modules
#[cfg(feature = "ml")]
pub mod ml;

// Re-exports for convenience
pub use bytesil::*;
pub use console::*;
pub use debug::*;
pub use layers::*;
pub use math::*;
pub use state::*;
pub use string::*;
pub use transforms::*;

#[cfg(feature = "jsil")]
pub use io::*;

#[cfg(feature = "http")]
pub use http::*;

#[cfg(feature = "ml")]
pub use ml::*;
