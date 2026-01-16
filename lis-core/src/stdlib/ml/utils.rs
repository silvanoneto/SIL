//! # ML Utility Functions
//!
//! Helper functions for working with ByteSil and SilState in ML operations.
//! These provide convenient access to magnitude and phase as f64 values.

use sil_core::state::{ByteSil, SilState, NUM_LAYERS};
use std::f64::consts::PI;

/// Get the linear magnitude of a ByteSil value
///
/// ByteSil stores log-magnitude (rho), so magnitude = e^rho
#[inline]
pub fn magnitude(b: &ByteSil) -> f64 {
    (b.rho as f64).exp()
}

/// Get the phase of a ByteSil value in radians
///
/// ByteSil stores phase index (theta), so phase = theta * π/8
#[inline]
pub fn phase(b: &ByteSil) -> f64 {
    (b.theta as f64) * PI / 8.0
}

/// Create a ByteSil from linear magnitude and phase (radians)
///
/// Converts magnitude to log-magnitude and phase to phase index
#[inline]
pub fn from_mag_phase(mag: f64, phase_rad: f64) -> ByteSil {
    // Convert magnitude to log-magnitude
    let rho = if mag <= 0.0 {
        -8 // Minimum (essentially zero)
    } else {
        mag.ln().clamp(-8.0, 7.0) as i8
    };

    // Convert phase to index (0-15)
    // Normalize to [0, 2π) then divide by π/8
    let phase_norm = ((phase_rad % (2.0 * PI)) + 2.0 * PI) % (2.0 * PI);
    let theta = ((phase_norm * 8.0 / PI).round() as u8) & 0x0F;

    ByteSil::new(rho, theta)
}

/// Create a ByteSil from just magnitude (phase = 0)
#[inline]
pub fn from_magnitude(mag: f64) -> ByteSil {
    from_mag_phase(mag, 0.0)
}

/// Get layer magnitude from state
#[inline]
pub fn layer_magnitude(state: &SilState, index: usize) -> f64 {
    magnitude(&state.get(index))
}

/// Get layer phase from state
#[inline]
pub fn layer_phase(state: &SilState, index: usize) -> f64 {
    phase(&state.get(index))
}

/// Set layer by magnitude and phase
#[inline]
pub fn with_layer_mag_phase(state: &SilState, index: usize, mag: f64, phase_rad: f64) -> SilState {
    state.with_layer(index, from_mag_phase(mag, phase_rad))
}

/// Set layer by magnitude only (preserves original phase)
#[inline]
pub fn with_layer_magnitude(state: &SilState, index: usize, mag: f64) -> SilState {
    let original_phase = layer_phase(state, index);
    state.with_layer(index, from_mag_phase(mag, original_phase))
}

/// Convert state to vector of magnitudes
pub fn state_to_magnitudes(state: &SilState) -> [f64; NUM_LAYERS] {
    let mut result = [0.0; NUM_LAYERS];
    for i in 0..NUM_LAYERS {
        result[i] = layer_magnitude(state, i);
    }
    result
}

/// Convert state to vector of phases
pub fn state_to_phases(state: &SilState) -> [f64; NUM_LAYERS] {
    let mut result = [0.0; NUM_LAYERS];
    for i in 0..NUM_LAYERS {
        result[i] = layer_phase(state, i);
    }
    result
}

/// Create state from magnitudes and phases
pub fn state_from_mag_phase(mags: &[f64; NUM_LAYERS], phases: &[f64; NUM_LAYERS]) -> SilState {
    let mut state = SilState::vacuum();
    for i in 0..NUM_LAYERS {
        state = state.with_layer(i, from_mag_phase(mags[i], phases[i]));
    }
    state
}

/// Create state from magnitudes only (all phases = 0)
pub fn state_from_magnitudes(mags: &[f64; NUM_LAYERS]) -> SilState {
    let mut state = SilState::vacuum();
    for i in 0..NUM_LAYERS {
        state = state.with_layer(i, from_magnitude(mags[i]));
    }
    state
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magnitude_roundtrip() {
        let b = from_magnitude(1.0);
        let mag = magnitude(&b);
        assert!((mag - 1.0).abs() < 0.1); // Allow some quantization error
    }

    #[test]
    fn test_phase_roundtrip() {
        let b = from_mag_phase(1.0, PI / 2.0);
        let p = phase(&b);
        assert!((p - PI / 2.0).abs() < 0.4); // Allow quantization error (16 phase values)
    }

    #[test]
    fn test_state_magnitudes() {
        let state = SilState::neutral();
        let mags = state_to_magnitudes(&state);

        // neutral() has rho=0 for all layers, so magnitude = e^0 = 1
        for &m in &mags {
            assert!((m - 1.0).abs() < 0.01);
        }
    }
}
