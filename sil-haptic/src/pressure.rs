//! Implementação de sensor de pressão (L4)

use serde::{Deserialize, Serialize};
use sil_core::prelude::*;
use sil_core::traits::{SilComponent, Sensor, SensorError};
use crate::error::{HapticError, HapticResult};
use crate::types::{HapticData, HapticReading, Pressure, Temperature, Vibration};

/// Configuração do sensor de pressão
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PressureConfig {
    /// Taxa de amostragem (Hz)
    pub sample_rate: u32,
    /// Número de leituras por captura
    pub buffer_size: usize,
    /// Sensibilidade (0.0 - 1.0)
    pub sensitivity: f32,
    /// Compensação de temperatura
    pub temperature_compensation: bool,
}

impl Default for PressureConfig {
    fn default() -> Self {
        Self {
            sample_rate: 100,
            buffer_size: 10,
            sensitivity: 0.8,
            temperature_compensation: true,
        }
    }
}

/// Sensor de pressão háptico (L4)
///
/// Mede pressão tátil e temperatura. No formato SIL:
/// - ρ (magnitude): pressão normalizada [-8, 7]
/// - θ (fase): temperatura mapeada [0, 255]
#[derive(Debug, Clone)]
pub struct PressureSensor {
    config: PressureConfig,
    ready: bool,
    reading_count: u64,
    calibration_offset: f32,
}

impl PressureSensor {
    /// Cria novo sensor de pressão com configuração padrão
    pub fn new() -> HapticResult<Self> {
        Self::with_config(PressureConfig::default())
    }

    /// Cria sensor com configuração específica
    pub fn with_config(config: PressureConfig) -> HapticResult<Self> {
        if config.sample_rate == 0 || config.sample_rate > 10_000 {
            return Err(HapticError::InvalidConfig(
                "Sample rate must be between 1 and 10000 Hz".into(),
            ));
        }

        if config.buffer_size == 0 || config.buffer_size > 1000 {
            return Err(HapticError::InvalidConfig(
                "Buffer size must be between 1 and 1000".into(),
            ));
        }

        if config.sensitivity < 0.0 || config.sensitivity > 1.0 {
            return Err(HapticError::InvalidConfig(
                "Sensitivity must be between 0.0 and 1.0".into(),
            ));
        }

        Ok(Self {
            config,
            ready: true,
            reading_count: 0,
            calibration_offset: 0.0,
        })
    }

    /// Retorna taxa de amostragem
    pub fn sample_rate(&self) -> u32 {
        self.config.sample_rate
    }

    /// Retorna número de leituras realizadas
    pub fn reading_count(&self) -> u64 {
        self.reading_count
    }

    /// Retorna offset de calibração
    pub fn calibration_offset(&self) -> f32 {
        self.calibration_offset
    }
}

impl Default for PressureSensor {
    fn default() -> Self {
        Self::new().expect("Default PressureSensor creation should not fail")
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// IMPLEMENTAÇÃO DOS TRAITS DO CORE
// ═══════════════════════════════════════════════════════════════════════════════

impl SilComponent for PressureSensor {
    fn name(&self) -> &str {
        "PressureSensor"
    }

    fn layers(&self) -> &[u8] {
        &[4] // L4 = Háptico/Dérmico
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn is_ready(&self) -> bool {
        self.ready
    }
}

impl Sensor for PressureSensor {
    type RawData = HapticData;
    type Config = PressureConfig;

    fn configure(&mut self, config: Self::Config) -> Result<(), SensorError> {
        if config.sample_rate == 0 || config.sample_rate > 10_000 {
            return Err(SensorError::InvalidConfig(
                "Sample rate must be between 1 and 10000 Hz".into(),
            ));
        }

        if config.buffer_size == 0 || config.buffer_size > 1000 {
            return Err(SensorError::InvalidConfig(
                "Buffer size must be between 1 and 1000".into(),
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

        // Mock: gera dados sintéticos de pressão
        // Em produção, leria de sensores físicos (FSR, piezoelétrico, etc.)
        let data = self.generate_mock_data();
        self.reading_count += data.len() as u64;

        Ok(data)
    }

    fn to_byte_sil(&self, raw: &Self::RawData) -> ByteSil {
        // Usa a média das leituras para conversão
        let avg_pressure = raw.avg_pressure();
        let avg_temperature = raw.avg_temperature();

        // ρ (magnitude): pressão normalizada para [-8, 7]
        let rho = avg_pressure.to_sil_rho();

        // θ (fase): temperatura mapeada para [0, 255]
        let theta = avg_temperature.to_sil_theta();

        ByteSil::new(rho, theta)
    }

    fn target_layer(&self) -> u8 {
        4 // L4
    }

    fn calibrate(&mut self) -> Result<(), SensorError> {
        // Calibração: estabelece baseline de pressão atmosférica
        self.ready = false;

        // Faz várias leituras para determinar o offset
        let mut total = 0.0;
        let samples = 10;

        for _ in 0..samples {
            let mock_pressure = self.simulate_pressure_reading();
            total += mock_pressure;
        }

        // Offset é a diferença entre média e pressão atmosférica padrão
        let average = total / samples as f32;
        self.calibration_offset = 101_325.0 - average;

        self.ready = true;
        self.reading_count = 0;

        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// IMPLEMENTAÇÃO AUXILIAR
// ═══════════════════════════════════════════════════════════════════════════════

impl PressureSensor {
    /// Gera dados mock para testes
    fn generate_mock_data(&self) -> HapticData {
        let mut readings = Vec::with_capacity(self.config.buffer_size);

        for i in 0..self.config.buffer_size {
            let pressure = self.simulate_pressure_reading();
            let temperature = self.simulate_temperature_reading();
            let vibration = self.simulate_vibration(i);

            let reading = HapticReading::new(
                Pressure::new(pressure + self.calibration_offset),
                Temperature::new(temperature),
                vibration,
            );

            readings.push(reading);
        }

        HapticData::new(readings, self.config.sample_rate)
    }

    /// Simula leitura de pressão (mock)
    fn simulate_pressure_reading(&self) -> f32 {
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as f32;

        // Pressão atmosférica base
        let base = 101_325.0;

        // Variação sinusoidal (simula toque periódico)
        let variation = (now / 1000.0).sin() * 5000.0 * self.config.sensitivity;

        // Adiciona ruído
        let noise = (now * 123.456).sin() * 100.0;

        base + variation + noise
    }

    /// Simula leitura de temperatura (mock)
    fn simulate_temperature_reading(&self) -> f32 {
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as f32;

        // Temperatura ambiente base
        let base = 22.0;

        // Variação lenta (aquecimento por toque)
        let variation = (now / 10.0).sin() * 3.0;

        // Compensação de temperatura se habilitada
        let compensation = if self.config.temperature_compensation {
            -0.5
        } else {
            0.0
        };

        base + variation + compensation
    }

    /// Simula vibração (mock)
    fn simulate_vibration(&self, sample_index: usize) -> Vibration {
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as f32;

        // Frequência varia com o tempo
        let freq = 50.0 + (now / 500.0).sin() * 30.0;

        // Amplitude varia com o índice da amostra
        let amplitude = ((sample_index as f32 / 10.0).sin() * 0.3 + 0.3) * self.config.sensitivity;

        Vibration::new(freq, amplitude)
    }
}
