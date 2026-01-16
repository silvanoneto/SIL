//! Transform Utilities for LIS
//!
//! Common transform patterns and signal processing operations.

use crate::error::Result;
use num_complex::Complex;
use sil_core::state::{ByteSil, SilState};

/// @stdlib_transforms fn transform_phase_shift(s: State, amount: Int) -> State
///
/// Shifts the phase of all layers by a specified amount.
pub fn transform_phase_shift(s: &SilState, amount: u8) -> Result<SilState> {
    let shift = ByteSil::new(0, amount);
    let mut result = *s;
    for i in 0..16 {
        let layer = result.layer(i);
        result = result.with_layer(i as usize, layer.mul(&shift));
    }
    Ok(result)
}

/// @stdlib_transforms fn transform_magnitude_scale(s: State, delta: Int) -> State
///
/// Scales the magnitude of all layers.
pub fn transform_magnitude_scale(s: &SilState, delta: i8) -> Result<SilState> {
    let mut result = *s;
    for i in 0..16 {
        let layer = result.layer(i);
        let new_rho = (layer.rho as i16 + delta as i16).clamp(0, 255) as u8;
        result = result.with_layer(i as usize, ByteSil::new(new_rho as i8, layer.theta));
    }
    Ok(result)
}

/// @stdlib_transforms fn transform_layer_swap(s: State, a: Int, b: Int) -> State
///
/// Swaps two layers.
pub fn transform_layer_swap(s: &SilState, a: u8, b: u8) -> Result<SilState> {
    if a >= 16 || b >= 16 {
        return Err(crate::error::Error::SemanticError {
            message: "Layer indices out of bounds".into(),
        });
    }
    let layer_a = s.layer(a as usize);
    let layer_b = s.layer(b as usize);
    let mut result = *s;
    result = result.with_layer(a as usize, layer_b);
    result = result.with_layer(b as usize, layer_a);
    Ok(result)
}

/// @stdlib_transforms fn transform_xor_layers(s: State, src_a: Int, src_b: Int, dest: Int) -> State
///
/// XORs two source layers and writes to destination layer.
pub fn transform_xor_layers(s: &SilState, src_a: u8, src_b: u8, dest: u8) -> Result<SilState> {
    if src_a >= 16 || src_b >= 16 || dest >= 16 {
        return Err(crate::error::Error::SemanticError {
            message: "Layer indices out of bounds".into(),
        });
    }
    let layer_a = s.layer(src_a as usize);
    let layer_b = s.layer(src_b as usize);
    let xored = layer_a.xor(&layer_b);
    Ok(s.with_layer(dest as usize, xored))
}

/// @stdlib_transforms fn transform_identity(s: State) -> State
///
/// Identity transform (returns state unchanged).
pub fn transform_identity(s: &SilState) -> Result<SilState> {
    Ok(*s)
}

/// @stdlib_transforms fn apply_feedback(s: State, gain: Float) -> State
///
/// Applies feedback by mixing state with itself scaled by gain.
pub fn apply_feedback(s: &SilState, gain: f64) -> Result<SilState> {
    let gain_clamped = gain.clamp(0.0, 1.0);
    let mut result = *s;
    for i in 0..16 {
        let layer = result.layer(i);
        let c = layer.to_complex(); let (re, im) = (c.re, c.im);
        let feedback = ByteSil::from_complex(Complex::new(re * gain_clamped, im * gain_clamped));
        result = result.with_layer(i as usize, layer.mix(&feedback));
    }
    Ok(result)
}

/// @stdlib_transforms fn detect_emergence(s: State, threshold: Float) -> Bool
///
/// Detects if emergence has occurred based on magnitude threshold.
pub fn detect_emergence(s: &SilState, threshold: f64) -> Result<bool> {
    let emergence_layers = s.emergence();
    for layer in &emergence_layers {
        let c = layer.to_complex(); let (re, im) = (c.re, c.im);
        let magnitude = (re * re + im * im).sqrt();
        if magnitude > threshold {
            return Ok(true);
        }
    }
    Ok(false)
}

/// @stdlib_transforms fn emergence_pattern(s: State) -> ByteSil
///
/// Extracts the emergence pattern from a state.
pub fn emergence_pattern(s: &SilState) -> Result<ByteSil> {
    let emergence = s.emergence();
    Ok(emergence[0].mix(&emergence[1]))
}

/// @stdlib_transforms fn autopoietic_loop(s: State, iterations: Int) -> State
///
/// Runs an autopoietic feedback loop for N iterations.
pub fn autopoietic_loop(s: &SilState, iterations: u32) -> Result<SilState> {
    let mut result = *s;
    for _ in 0..iterations {
        result = result.tensor(&result);
        // Collapse and spread back
        let collapsed = result.collapse(sil_core::state::CollapseStrategy::Xor);
        for i in 0..16 {
            result = result.with_layer(i as usize, result.layer(i).mix(&collapsed));
        }
    }
    Ok(result)
}
