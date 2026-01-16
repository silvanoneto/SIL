//! Implementação de sensor de gás (L2)

use serde::{Deserialize, Serialize};
use sil_core::prelude::*;
use sil_core::traits::{SilComponent, Sensor, SensorError};
use crate::error::{OlfactoryError, OlfactoryResult};
use crate::types::{GasData, GasConcentration, GasType};

/// Configuração do sensor de gás
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasSensorConfig {
    /// Taxa de amostragem em Hz
    pub sample_rate: f32,
    /// Temperatura de operação (°C)
    pub operating_temperature: f32,
    /// Tempo de aquecimento (ms)
    pub warmup_time: u32,
    /// Gases a detectar (vazio = todos)
    pub target_gases: Vec<GasType>,
    /// Sensibilidade (0.0 - 1.0)
    pub sensitivity: f32,
}

impl Default for GasSensorConfig {
    fn default() -> Self {
        Self {
            sample_rate: 1.0, // 1 Hz (1 leitura por segundo)
            operating_temperature: 25.0,
            warmup_time: 5000, // 5 segundos
            target_gases: vec![
                GasType::CO,
                GasType::CO2,
                GasType::CH4,
                GasType::NH3,
                GasType::VOC,
            ],
            sensitivity: 0.5,
        }
    }
}

/// Sensor de gás olfativo (L2)
#[derive(Debug, Clone)]
pub struct GasSensor {
    config: GasSensorConfig,
    ready: bool,
    sample_count: u64,
    last_reading: Option<GasData>,
    calibration_baseline: f32,
}

impl GasSensor {
    /// Cria novo sensor de gás com configuração padrão
    pub fn new() -> OlfactoryResult<Self> {
        Self::with_config(GasSensorConfig::default())
    }

    /// Cria sensor com configuração específica
    pub fn with_config(config: GasSensorConfig) -> OlfactoryResult<Self> {
        if config.sample_rate <= 0.0 || config.sample_rate > 100.0 {
            return Err(OlfactoryError::InvalidConfig(
                "Sample rate must be between 0 and 100 Hz".into(),
            ));
        }

        if config.sensitivity < 0.0 || config.sensitivity > 1.0 {
            return Err(OlfactoryError::InvalidConfig(
                "Sensitivity must be between 0.0 and 1.0".into(),
            ));
        }

        Ok(Self {
            config,
            ready: true,
            sample_count: 0,
            last_reading: None,
            calibration_baseline: 0.0,
        })
    }

    /// Retorna taxa de amostragem atual
    pub fn get_sample_rate(&self) -> f32 {
        self.config.sample_rate
    }

    /// Retorna número de amostras capturadas
    pub fn sample_count(&self) -> u64 {
        self.sample_count
    }

    /// Retorna última leitura
    pub fn last_reading(&self) -> Option<&GasData> {
        self.last_reading.as_ref()
    }

    /// Retorna baseline de calibração
    pub fn baseline(&self) -> f32 {
        self.calibration_baseline
    }

    /// Simula leitura de múltiplos gases
    fn simulate_gas_reading(&self) -> GasData {
        use std::f32::consts::PI;
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let time_factor = now as f32;

        // Temperatura e umidade ambiente (simuladas)
        let temperature = 20.0 + 5.0 * ((time_factor / 60.0) * 2.0 * PI).sin();
        let humidity = 50.0 + 10.0 * ((time_factor / 90.0) * 2.0 * PI).cos();

        // Gera concentrações para cada gás alvo
        let mut compounds = Vec::new();

        for (i, &gas_type) in self.config.target_gases.iter().enumerate() {
            let (min, max) = gas_type.typical_range();

            // Cada gás tem um padrão temporal único
            let phase = (i as f32 * PI / 3.0) + (time_factor / 30.0);
            let wave = ((phase * 2.0 * PI).sin() + 1.0) / 2.0; // [0, 1]

            // Aplica sensibilidade
            let base_ppm = min + (max - min) * wave * self.config.sensitivity;

            // Adiciona ruído
            let noise = ((time_factor * (i + 1) as f32).sin() * 0.1 + 1.0) * base_ppm;

            // Aplica baseline de calibração
            let adjusted_ppm = (noise - self.calibration_baseline).max(0.0);

            compounds.push(GasConcentration::new(gas_type, adjusted_ppm));
        }

        GasData::from_compounds(compounds, temperature, humidity)
    }
}

impl Default for GasSensor {
    fn default() -> Self {
        Self::new().expect("Default GasSensor creation should not fail")
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// IMPLEMENTAÇÃO DOS TRAITS DO CORE
// ═══════════════════════════════════════════════════════════════════════════════

impl SilComponent for GasSensor {
    fn name(&self) -> &str {
        "GasSensor"
    }

    fn layers(&self) -> &[u8] {
        &[2] // L2 = Olfativo
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn is_ready(&self) -> bool {
        self.ready
    }
}

impl Sensor for GasSensor {
    type RawData = GasData;
    type Config = GasSensorConfig;

    fn configure(&mut self, config: Self::Config) -> Result<(), SensorError> {
        if config.sample_rate <= 0.0 || config.sample_rate > 100.0 {
            return Err(SensorError::InvalidConfig(
                "Sample rate must be between 0 and 100 Hz".into(),
            ));
        }

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

        // Mock: simula leitura de sensor de gás
        // Em produção, leria de sensores reais (MQ-series, BME680, etc.)
        let data = self.simulate_gas_reading();
        self.sample_count += 1;
        self.last_reading = Some(data.clone());

        Ok(data)
    }

    fn to_byte_sil(&self, raw: &Self::RawData) -> ByteSil {
        // ρ (magnitude): concentração composta normalizada para [-8, 7]
        // Baseado no índice de qualidade do ar (AQI)
        let rho = raw.composite_concentration() as i8;

        // θ (fase): ID do composto dominante ou assinatura de odor
        // Mapeia tipo de gás ou classificação para [0, 255]
        let theta = raw.dominant_signature();

        ByteSil::new(rho.clamp(-8, 7), theta)
    }

    fn target_layer(&self) -> u8 {
        2 // L2 = Olfativo
    }

    fn sample_rate(&self) -> f32 {
        self.config.sample_rate
    }

    fn calibrate(&mut self) -> Result<(), SensorError> {
        // Calibração: lê ar ambiente e define como baseline
        self.ready = true;

        // Faz algumas leituras para estabelecer baseline
        let mut baseline_sum = 0.0;
        let calibration_samples = 5;

        for _ in 0..calibration_samples {
            let data = self.simulate_gas_reading();

            // Soma concentrações de todos os compostos
            for compound in &data.profile.compounds {
                baseline_sum += compound.ppm;
            }
        }

        // Define baseline como média
        self.calibration_baseline = baseline_sum / (calibration_samples as f32);

        // Reset contador
        self.sample_count = 0;
        self.last_reading = None;

        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MÉTODOS AUXILIARES
// ═══════════════════════════════════════════════════════════════════════════════

impl GasSensor {
    /// Adiciona tipo de gás à lista de detecção
    pub fn add_target_gas(&mut self, gas_type: GasType) {
        if !self.config.target_gases.contains(&gas_type) {
            self.config.target_gases.push(gas_type);
        }
    }

    /// Remove tipo de gás da lista de detecção
    pub fn remove_target_gas(&mut self, gas_type: GasType) {
        self.config.target_gases.retain(|&g| g != gas_type);
    }

    /// Define sensibilidade do sensor
    pub fn set_sensitivity(&mut self, sensitivity: f32) -> Result<(), OlfactoryError> {
        if sensitivity < 0.0 || sensitivity > 1.0 {
            return Err(OlfactoryError::InvalidConfig(
                "Sensitivity must be between 0.0 and 1.0".into(),
            ));
        }
        self.config.sensitivity = sensitivity;
        Ok(())
    }

    /// Retorna se há gases perigosos na última leitura
    pub fn has_hazardous_gases(&self) -> bool {
        self.last_reading
            .as_ref()
            .map(|data| data.is_hazardous())
            .unwrap_or(false)
    }

    /// Retorna índice de qualidade do ar da última leitura
    pub fn air_quality_index(&self) -> Option<u16> {
        self.last_reading
            .as_ref()
            .map(|data| data.profile.air_quality_index)
    }
}
