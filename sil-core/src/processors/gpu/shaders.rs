//! WGSL Compute Shaders para operações SIL
//!
//! Shaders pré-compilados em build time via build.rs

#[allow(dead_code)]
mod compiled {
    // Include shaders compilados pelo build.rs
    include!(concat!(env!("OUT_DIR"), "/compiled_shaders.rs"));
}

#[allow(unused_imports)]
pub use compiled::*;
