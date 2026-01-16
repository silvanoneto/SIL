//! Implementação de sensor de luz ambiente (L0)

use serde::{Deserialize, Serialize};
use sil_core::prelude::*;
use sil_core::traits::{SilComponent, Sensor, SensorError};
use crate::error::{PhotonicError, PhotonicResult};
use crate::types::Intensity;

/// Configuração do sensor de luz
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightConfig {
    /// Sensibilidade (0.0 - 1.0)
    pub sensitivity: f32,
    /// Taxa de amostragem (Hz)
    pub sample_rate: f32,
}

impl Default for LightConfig {
    fn default() -> Self {
        Self {
            sensitivity: 0.5,
            sample_rate: 100.0,
        }
    }
}

/// Sensor de luz ambiente (L0)
#[derive(Debug, Clone)]
pub struct LightSensor {
    config: LightConfig,
    ready: bool,
    last_reading: f32,
}

impl LightSensor {
    /// Cria novo sensor de luz
    pub fn new() -> PhotonicResult<Self> {
        Self::with_config(LightConfig::default())
    }

    /// Cria sensor com configuração específica
    pub fn with_config(config: LightConfig) -> PhotonicResult<Self> {
        if config.sensitivity < 0.0 || config.sensitivity > 1.0 {
            return Err(PhotonicError::InvalidConfig(
                "Sensitivity must be between 0.0 and 1.0".into(),
            ));
        }

        Ok(Self {
            config,
            ready: true,
            last_reading: 0.0,
        })
    }

    /// Retorna última leitura em lux
    pub fn last_reading(&self) -> f32 {
        self.last_reading
    }
}

impl Default for LightSensor {
    fn default() -> Self {
        Self::new().expect("Default LightSensor creation should not fail")
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// IMPLEMENTAÇÃO DOS TRAITS DO CORE
// ═══════════════════════════════════════════════════════════════════════════════

impl SilComponent for LightSensor {
    fn name(&self) -> &str {
        "LightSensor"
    }

    fn layers(&self) -> &[u8] {
        &[0] // L0 = Fotônico
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn is_ready(&self) -> bool {
        self.ready
    }
}

impl Sensor for LightSensor {
    type RawData = Intensity;
    type Config = LightConfig;

    fn configure(&mut self, config: Self::Config) -> Result<(), SensorError> {
        if config.sensitivity < 0.0 || config.sensitivity > 1.0 {
            return Err(SensorError::InvalidConfig(
                "Sensitivity must be between 0.0 and 1.0".into(),
            ));
        }
        self.config = config;
        Ok(())
    }

    fn read(&mut self) -> Result<Self::RawData, SensorError> {
        if !self.ready {
            return Err(SensorError::NotInitialized);
        }

        // Mock: simula leitura de sensor de luz
        // Em produção, leria de um sensor físico (TSL2561, BH1750, etc.)
        let lux = self.simulate_light_reading();
        self.last_reading = lux;

        Ok(Intensity::new(lux))
    }

    fn to_byte_sil(&self, raw: &Self::RawData) -> ByteSil {
        // ρ (magnitude): intensidade luminosa normalizada
        // 0 lux = -8, 1000 lux = +7
        let normalized = (raw.value / 1000.0).clamp(0.0, 1.0);
        let rho = ((normalized * 15.0) - 8.0) as i8;

        // θ (fase): não usado para luz ambiente (sempre 0)
        ByteSil::new(rho.clamp(-8, 7), 0)
    }

    fn target_layer(&self) -> u8 {
        0 // L0
    }

    fn calibrate(&mut self) -> Result<(), SensorError> {
        // Calibração: ajusta baseline
        self.ready = true;
        self.last_reading = 0.0;
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// IMPLEMENTAÇÃO AUXILIAR
// ═══════════════════════════════════════════════════════════════════════════════

impl LightSensor {
    /// Simula leitura de luz (mock para desenvolvimento)
    fn simulate_light_reading(&self) -> f32 {
        // Simula variação de luz ambiente entre 10 e 900 lux
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Varia sinusoidalmente
        let base = 450.0; // Luz média
        let amplitude = 400.0 * self.config.sensitivity;
        let period = 60.0; // Período de 60 segundos

        base + amplitude * ((now as f32 / period) * 2.0 * std::f32::consts::PI).sin()
    }
}
