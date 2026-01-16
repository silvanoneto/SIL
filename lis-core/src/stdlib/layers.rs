//! Layer-Specific Operations for LIS
//!
//! Semantic operations for the 16-layer SIL architecture.

use crate::error::Result;
use sil_core::state::{ByteSil, SilState};

// Layer index constants
pub const LAYER_PHOTONIC: u8 = 0;
pub const LAYER_ACOUSTIC: u8 = 1;
pub const LAYER_OLFACTORY: u8 = 2;
pub const LAYER_GUSTATORY: u8 = 3;
pub const LAYER_DERMIC: u8 = 4;
pub const LAYER_ELECTRONIC: u8 = 5;
pub const LAYER_PSYCHOMOTOR: u8 = 6;
pub const LAYER_ENVIRONMENTAL: u8 = 7;
pub const LAYER_CYBERNETIC: u8 = 8;
pub const LAYER_GEOPOLITICAL: u8 = 9;
pub const LAYER_COSMOPOLITICAL: u8 = 10;
pub const LAYER_SYNERGIC: u8 = 11;
pub const LAYER_QUANTUM: u8 = 12;
pub const LAYER_SUPERPOSITION: u8 = 13;
pub const LAYER_ENTANGLEMENT: u8 = 14;
pub const LAYER_COLLAPSE: u8 = 15;

/// @stdlib_layers fn fuse_vision_audio(vision: ByteSil, audio: ByteSil) -> ByteSil
///
/// Fuses visual and auditory sensory data using XOR blend.
pub fn fuse_vision_audio(vision: &ByteSil, audio: &ByteSil) -> Result<ByteSil> {
    Ok(vision.mix(audio))
}

/// @stdlib_layers fn fuse_multimodal(layers: [ByteSil; 5]) -> ByteSil
///
/// Fuses all perception layers (L0-L4) into a single value.
pub fn fuse_multimodal(layers: &[ByteSil; 5]) -> Result<ByteSil> {
    let mut result = layers[0];
    for i in 1..5 {
        result = result.mix(&layers[i]);
    }
    Ok(result)
}

/// @stdlib_layers fn normalize_perception(s: State) -> State
///
/// Normalizes perception layers to neutral values.
pub fn normalize_perception(s: &SilState) -> Result<SilState> {
    let mut result = *s;
    for i in 0..5 {
        let layer = result.layer(i);
        let normalized = layer.mul(&ByteSil::ONE);
        result = result.with_layer(i as usize, normalized);
    }
    Ok(result)
}

/// @stdlib_layers fn shift_layers_up(s: State) -> State
///
/// Shifts all layers up by one (wraps around).
pub fn shift_layers_up(s: &SilState) -> Result<SilState> {
    let mut layers = [ByteSil::NULL; 16];
    for i in 0..16 {
        layers[(i + 1) % 16] = s.layer(i);
    }
    Ok(SilState::from_layers(layers))
}

/// @stdlib_layers fn shift_layers_down(s: State) -> State
///
/// Shifts all layers down by one (wraps around).
pub fn shift_layers_down(s: &SilState) -> Result<SilState> {
    let mut layers = [ByteSil::NULL; 16];
    for i in 0..16 {
        layers[i] = s.layer((i + 1) % 16);
    }
    Ok(SilState::from_layers(layers))
}

/// @stdlib_layers fn rotate_layers(s: State, amount: Int) -> State
///
/// Rotates layers by a specified amount.
pub fn rotate_layers(s: &SilState, amount: i32) -> Result<SilState> {
    let mut layers = [ByteSil::NULL; 16];
    for i in 0..16 {
        let new_idx = ((i as i32 + amount) % 16 + 16) % 16;
        layers[new_idx as usize] = s.layer(i);
    }
    Ok(SilState::from_layers(layers))
}

/// @stdlib_layers fn spread_to_group(s: State, start: Int, value: ByteSil, count: Int) -> State
///
/// Spreads a value to multiple consecutive layers.
pub fn spread_to_group(s: &SilState, start: u8, value: ByteSil, count: u8) -> Result<SilState> {
    let mut result = *s;
    for i in 0..count {
        let idx = (start + i) % 16;
        result = result.with_layer(idx as usize, value);
    }
    Ok(result)
}
