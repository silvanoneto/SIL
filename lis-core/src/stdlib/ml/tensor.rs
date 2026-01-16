//! # StateTensor - Composable States
//!
//! Compose multiple SilStates for larger models (e.g., 64x64 = 256 States).

use sil_core::state::{SilState, NUM_LAYERS};
use crate::stdlib::ml::utils::{magnitude, from_mag_phase};

/// StateTensor: Multiple States as larger tensor
pub struct StateTensor {
    pub states: Vec<SilState>,
    pub shape: Vec<usize>,
}

impl StateTensor {
    /// Create new tensor with given shape
    pub fn new(shape: Vec<usize>) -> Self {
        let total: usize = shape.iter().product();
        let num_states = (total + NUM_LAYERS - 1) / NUM_LAYERS;

        Self {
            states: vec![SilState::vacuum(); num_states],
            shape,
        }
    }

    /// Create from single State
    pub fn from_state(state: SilState) -> Self {
        Self {
            states: vec![state],
            shape: vec![NUM_LAYERS],
        }
    }

    /// Get value at index
    pub fn get(&self, idx: usize) -> f64 {
        let state_idx = idx / NUM_LAYERS;
        let layer_idx = idx % NUM_LAYERS;

        if state_idx < self.states.len() {
            magnitude(&self.states[state_idx].get(layer_idx))
        } else {
            0.0
        }
    }

    /// Set value at index
    pub fn set(&mut self, idx: usize, value: f64) {
        let state_idx = idx / NUM_LAYERS;
        let layer_idx = idx % NUM_LAYERS;

        if state_idx < self.states.len() {
            self.states[state_idx] = self.states[state_idx]
                .with_layer(layer_idx, from_mag_phase(value, 0.0));
        }
    }

    /// Total number of elements
    pub fn numel(&self) -> usize {
        self.shape.iter().product()
    }

    /// Reshape tensor
    pub fn reshape(&mut self, new_shape: Vec<usize>) {
        let old_numel: usize = self.shape.iter().product();
        let new_numel: usize = new_shape.iter().product();
        assert_eq!(old_numel, new_numel, "reshape: size mismatch");
        self.shape = new_shape;
    }
}

impl Default for StateTensor {
    fn default() -> Self {
        Self::new(vec![NUM_LAYERS])
    }
}

/// Tensor Train decomposition (for compression)
pub struct TensorTrain {
    pub cores: Vec<SilState>,
    pub ranks: Vec<usize>,
}

impl TensorTrain {
    pub fn compress_state(state: &SilState, max_rank: usize) -> Self {
        // Simplified TT: just store the state
        Self {
            cores: vec![*state],
            ranks: vec![1, max_rank.min(4), 1],
        }
    }

    pub fn decompress(&self) -> SilState {
        if self.cores.is_empty() {
            SilState::vacuum()
        } else {
            self.cores[0]
        }
    }
}
