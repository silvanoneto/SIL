//! # Edge Computing Module
//!
//! Hardware detection, ρ_Sil metric, and offload decisions.

// Module stubs for future implementation
// mod rho_sil;
// mod chromatic;
// mod offload;
// mod hw;

use sil_core::state::{SilState, NUM_LAYERS};
use crate::core::tensor::magnitude;
use crate::core::stats;

/// Device class based on ρ_Sil threshold
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceClass {
    Nano,      // ρ < 0.1
    Micro,     // 0.1 ≤ ρ < 0.2
    Mini,      // 0.2 ≤ ρ < 0.4
    Standard,  // 0.4 ≤ ρ < 0.6
    Edge,      // 0.6 ≤ ρ < 0.8
    Cloud,     // ρ ≥ 0.8
}

/// Device type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    MCU = 0,
    CPU = 1,
    GPU = 2,
    TPU = 3,
    NPU = 4,
    Neuromorphic = 5,
}

/// Hardware profile
#[derive(Debug, Clone)]
pub struct HardwareProfile {
    pub device_type: DeviceType,
    pub tflops: f64,
    pub memory_mb: usize,
    pub power_watts: f64,
}

impl HardwareProfile {
    pub fn detect() -> Self {
        #[cfg(target_os = "macos")]
        {
            Self {
                device_type: DeviceType::NPU,
                tflops: 15.0,
                memory_mb: 16384,
                power_watts: 30.0,
            }
        }
        
        #[cfg(not(target_os = "macos"))]
        {
            Self {
                device_type: DeviceType::CPU,
                tflops: 0.1,
                memory_mb: 8192,
                power_watts: 65.0,
            }
        }
    }

    pub fn can_fit_model(&self, params_million: f64) -> bool {
        let required_mb = params_million * 4.0;
        required_mb < self.memory_mb as f64
    }

    pub fn select_model_size(&self) -> &'static str {
        match self.device_type {
            DeviceType::MCU => "nano",
            DeviceType::CPU if self.tflops < 0.5 => "tiny",
            DeviceType::CPU => "small",
            DeviceType::GPU if self.tflops < 10.0 => "medium",
            DeviceType::GPU => "large",
            DeviceType::TPU | DeviceType::NPU => "large",
            DeviceType::Neuromorphic => "tiny",
        }
    }
}

impl Default for HardwareProfile {
    fn default() -> Self {
        Self::detect()
    }
}

/// Calculate ρ_Sil complexity metric
pub fn rho_sil(state: &SilState) -> f64 {
    let alpha = 0.4;
    let beta = 0.3;
    let gamma = 0.3;

    let transitions = transition_count(state);
    let entropy = stats::entropy(state) / (NUM_LAYERS as f64).ln();
    let variance = stats::variance(state);
    let norm_variance = (variance / 1.0).min(1.0);

    alpha * transitions + beta * entropy + gamma * norm_variance
}

/// Fast ρ_Sil approximation
pub fn rho_sil_fast(state: &SilState) -> f64 {
    transition_count(state)
}

fn transition_count(state: &SilState) -> f64 {
    let mut count = 0.0;
    let threshold = 0.1;

    for i in 1..NUM_LAYERS {
        let prev = magnitude(&state.get(i - 1));
        let curr = magnitude(&state.get(i));
        let diff = (curr - prev).abs();

        if diff > threshold {
            count += 1.0;
        }
    }

    count / (NUM_LAYERS - 1) as f64
}

/// Calculate dynamic critical threshold
pub fn rho_critical(rho_base: f64, cpu_load: f64, mem_load: f64, battery: f64) -> f64 {
    let phi_cpu = 1.0 - cpu_load.clamp(0.0, 1.0) * 0.5;
    let phi_mem = 1.0 - mem_load.clamp(0.0, 1.0) * 0.3;
    let phi_bat = battery.clamp(0.1, 1.0);

    rho_base * phi_cpu * phi_mem * phi_bat
}

/// Determine if should offload
pub fn should_offload(rho: f64, rho_critical: f64) -> bool {
    rho > rho_critical
}

/// Get device class from ρ_Sil
pub fn device_class(rho: f64) -> DeviceClass {
    if rho < 0.1 {
        DeviceClass::Nano
    } else if rho < 0.2 {
        DeviceClass::Micro
    } else if rho < 0.4 {
        DeviceClass::Mini
    } else if rho < 0.6 {
        DeviceClass::Standard
    } else if rho < 0.8 {
        DeviceClass::Edge
    } else {
        DeviceClass::Cloud
    }
}

/// Chromatic zone for routing decisions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChromaticZone {
    UltraLocal,  // On-chip, sub-ms latency
    Local,       // Same device, ms latency
    Near,        // LAN, 10ms latency
    Far,         // WAN, 100ms latency
    HPC,         // Cloud/HPC, seconds latency
}

impl ChromaticZone {
    pub fn from_rho(rho: f64) -> Self {
        if rho < 0.2 {
            ChromaticZone::UltraLocal
        } else if rho < 0.4 {
            ChromaticZone::Local
        } else if rho < 0.6 {
            ChromaticZone::Near
        } else if rho < 0.8 {
            ChromaticZone::Far
        } else {
            ChromaticZone::HPC
        }
    }
    
    pub fn max_latency_ms(&self) -> f64 {
        match self {
            ChromaticZone::UltraLocal => 0.1,
            ChromaticZone::Local => 1.0,
            ChromaticZone::Near => 10.0,
            ChromaticZone::Far => 100.0,
            ChromaticZone::HPC => 1000.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rho_sil() {
        let state = SilState::neutral();
        let rho = rho_sil(&state);
        assert!(rho >= 0.0 && rho <= 1.0);
    }

    #[test]
    fn test_device_class() {
        assert_eq!(device_class(0.05), DeviceClass::Nano);
        assert_eq!(device_class(0.5), DeviceClass::Standard);
        assert_eq!(device_class(0.9), DeviceClass::Cloud);
    }
    
    #[test]
    fn test_chromatic_zone() {
        assert_eq!(ChromaticZone::from_rho(0.1), ChromaticZone::UltraLocal);
        assert_eq!(ChromaticZone::from_rho(0.5), ChromaticZone::Near);
        assert_eq!(ChromaticZone::from_rho(0.9), ChromaticZone::HPC);
    }
}
