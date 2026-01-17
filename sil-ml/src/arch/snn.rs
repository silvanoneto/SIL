//! Spiking Neural Networks: neuromorphic computing with leaky integrate-and-fire.

use crate::error::SilMlError;
use sil_core::state::SilState;

/// Leaky Integrate-and-Fire (LIF) neuron
#[derive(Debug, Clone)]
pub struct LIFNeuron {
    pub tau_mem: f32,
    pub tau_syn: f32,
    pub v_th: f32,
}

impl LIFNeuron {
    pub fn new(tau_mem: f32, tau_syn: f32, v_th: f32) -> Self {
        Self {
            tau_mem,
            tau_syn,
            v_th,
        }
    }

    pub fn step(
        &self,
        _v_mem: f32,
        _i_syn: f32,
    ) -> Result<(f32, bool), SilMlError> {
        // Returns (updated_voltage, spike)
        // TODO: Implement LIF step
        Err(SilMlError::NotImplemented("LIFNeuron::step".into()))
    }
}

/// Spiking Neural Network layer
#[derive(Debug, Clone)]
pub struct SNNLayer {
    pub input_dim: usize,
    pub output_dim: usize,
    pub num_timesteps: usize,
}

impl SNNLayer {
    pub fn new(input_dim: usize, output_dim: usize, num_timesteps: usize) -> Self {
        Self {
            input_dim,
            output_dim,
            num_timesteps,
        }
    }

    pub fn forward(&self, _input: &SilState) -> Result<SilState, SilMlError> {
        // TODO: Implement SNN forward pass over timesteps
        Err(SilMlError::NotImplemented("SNNLayer::forward".into()))
    }
}

/// Spike encoding: rate coding, temporal coding
#[derive(Debug, Clone)]
pub enum SpikeEncoding {
    RateCoding { spike_rate: f32 },
    TemporalCoding { num_timesteps: usize },
}

impl SpikeEncoding {
    pub fn encode(&self, _value: f32) -> Result<Vec<bool>, SilMlError> {
        // TODO: Encode value as spike train
        Err(SilMlError::NotImplemented("SpikeEncoding::encode".into()))
    }
}
