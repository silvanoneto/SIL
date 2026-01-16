//! # Hardware Detection and Adaptation
//!
//! Device-aware model selection and execution.

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
    /// Detect current hardware (simplified)
    pub fn detect() -> Self {
        // In real implementation, would query actual hardware
        Self {
            device_type: DeviceType::CPU,
            tflops: 0.1,
            memory_mb: 8192,
            power_watts: 65.0,
        }
    }

    /// Estimate if model fits in memory
    pub fn can_fit_model(&self, params_million: f64) -> bool {
        // 4 bytes per param for FP32
        let required_mb = params_million * 4.0;
        required_mb < self.memory_mb as f64
    }

    /// Select model size based on device
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

/// Detect device type
pub fn detect_device() -> DeviceType {
    HardwareProfile::detect().device_type
}

/// Get device TFLOPS
pub fn device_tflops() -> f64 {
    HardwareProfile::detect().tflops
}

/// Check if model fits
pub fn can_fit_model(params_million: f64) -> bool {
    HardwareProfile::detect().can_fit_model(params_million)
}
