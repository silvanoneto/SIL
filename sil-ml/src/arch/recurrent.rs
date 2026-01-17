//! Recurrent Neural Networks: RNN, LSTM, GRU.

use crate::error::SilMlError;
use sil_core::state::SilState;

/// Long Short-Term Memory (LSTM) cell
#[derive(Debug, Clone)]
pub struct LSTMCell {
    pub input_dim: usize,
    pub hidden_dim: usize,
}

impl LSTMCell {
    pub fn new(input_dim: usize, hidden_dim: usize) -> Self {
        Self {
            input_dim,
            hidden_dim,
        }
    }

    pub fn forward(
        &self,
        _input: &SilState,
        _h_prev: &SilState,
        _c_prev: &SilState,
    ) -> Result<(SilState, SilState), SilMlError> {
        // TODO: Implement LSTM cell forward pass
        Err(SilMlError::NotImplemented("LSTMCell::forward".into()))
    }
}

/// Gated Recurrent Unit (GRU) cell
#[derive(Debug, Clone)]
pub struct GRUCell {
    pub input_dim: usize,
    pub hidden_dim: usize,
}

impl GRUCell {
    pub fn new(input_dim: usize, hidden_dim: usize) -> Self {
        Self {
            input_dim,
            hidden_dim,
        }
    }

    pub fn forward(
        &self,
        _input: &SilState,
        _h_prev: &SilState,
    ) -> Result<SilState, SilMlError> {
        // TODO: Implement GRU cell forward pass
        Err(SilMlError::NotImplemented("GRUCell::forward".into()))
    }
}

/// Vanilla RNN cell
#[derive(Debug, Clone)]
pub struct RNNCell {
    pub input_dim: usize,
    pub hidden_dim: usize,
}

impl RNNCell {
    pub fn new(input_dim: usize, hidden_dim: usize) -> Self {
        Self {
            input_dim,
            hidden_dim,
        }
    }

    pub fn forward(
        &self,
        _input: &SilState,
        _h_prev: &SilState,
    ) -> Result<SilState, SilMlError> {
        // TODO: Implement RNN cell forward pass
        Err(SilMlError::NotImplemented("RNNCell::forward".into()))
    }
}
