//! Implementação de sensor de toque (L4)

use serde::{Deserialize, Serialize};
use sil_core::prelude::*;
use sil_core::traits::{SilComponent, Sensor, SensorError};
use crate::error::{HapticError, HapticResult};
use crate::types::{HapticData, HapticReading, Pressure, Temperature, Vibration};

/// Configuração do sensor de toque
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HapticConfig {
    /// Taxa de amostragem (Hz)
    pub sample_rate: u32,
    /// Número de leituras por captura
    pub buffer_size: usize,
    /// Limiar de detecção de toque (Pa)
    pub touch_threshold: f32,
    /// Habilita detecção de vibração
    pub vibration_detection: bool,
}

impl Default for HapticConfig {
    fn default() -> Self {
        Self {
            sample_rate: 200,
            buffer_size: 20,
            touch_threshold: 5000.0, // 5kPa
            vibration_detection: true,
        }
    }
}

/// Sensor de toque háptico (L4)
///
/// Mede intensidade de pressão e área/frequência de contato. No formato SIL:
/// - ρ (magnitude): intensidade de pressão [-8, 7]
/// - θ (fase): área de contato ou frequência de vibração [0, 255]
#[derive(Debug, Clone)]
pub struct TouchSensor {
    config: HapticConfig,
    ready: bool,
    touch_count: u64,
    last_contact_area: f32, // em cm²
}

impl TouchSensor {
    /// Cria novo sensor de toque com configuração padrão
    pub fn new() -> HapticResult<Self> {
        Self::with_config(HapticConfig::default())
    }

    /// Cria sensor com configuração específica
    pub fn with_config(config: HapticConfig) -> HapticResult<Self> {
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

        if config.touch_threshold < 0.0 {
            return Err(HapticError::InvalidConfig(
                "Touch threshold must be >= 0".into(),
            ));
        }

        Ok(Self {
            config,
            ready: true,
            touch_count: 0,
            last_contact_area: 0.0,
        })
    }

    /// Retorna taxa de amostragem
    pub fn sample_rate(&self) -> u32 {
        self.config.sample_rate
    }

    /// Retorna número de toques detectados
    pub fn touch_count(&self) -> u64 {
        self.touch_count
    }

    /// Retorna última área de contato (cm²)
    pub fn last_contact_area(&self) -> f32 {
        self.last_contact_area
    }

    /// Verifica se há toque ativo
    pub fn is_touching(&self, data: &HapticData) -> bool {
        data.avg_pressure().pa > 101_325.0 + self.config.touch_threshold
    }
}

impl Default for TouchSensor {
    fn default() -> Self {
        Self::new().expect("Default TouchSensor creation should not fail")
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// IMPLEMENTAÇÃO DOS TRAITS DO CORE
// ═══════════════════════════════════════════════════════════════════════════════

impl SilComponent for TouchSensor {
    fn name(&self) -> &str {
        "TouchSensor"
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

impl Sensor for TouchSensor {
    type RawData = HapticData;
    type Config = HapticConfig;

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

        if config.touch_threshold < 0.0 {
            return Err(SensorError::InvalidConfig(
                "Touch threshold must be >= 0".into(),
            ));
        }

        self.config = config;
        Ok(())
    }

    fn read(&mut self) -> Result<Self::RawData, SensorError> {
        if !self.ready {
            return Err(SensorError::NotInitialized);
        }

        // Mock: gera dados sintéticos de toque
        // Em produção, leria de sensores capacitivos, resistivos, etc.
        let data = self.generate_mock_data();

        // Conta toques detectados
        if self.is_touching(&data) {
            self.touch_count += 1;
        }

        Ok(data)
    }

    fn to_byte_sil(&self, raw: &Self::RawData) -> ByteSil {
        // Usa a média das leituras para conversão
        let avg_pressure = raw.avg_pressure();
        let avg_vibration = raw.avg_vibration();

        // ρ (magnitude): intensidade de pressão normalizada para [-8, 7]
        // Mapeia de pressão atmosférica a +50kPa
        let pressure_intensity = (avg_pressure.pa - 101_325.0) / 50_000.0;
        let rho = (pressure_intensity * 15.0 - 8.0)
            .round()
            .clamp(-8.0, 7.0) as i8;

        // θ (fase): área de contato OU frequência de vibração [0, 255]
        let theta = if self.config.vibration_detection && avg_vibration.is_perceptible() {
            // Usa frequência de vibração
            avg_vibration.to_sil_theta()
        } else {
            // Usa área de contato (0-10 cm² mapeado para 0-255)
            let normalized_area = (self.last_contact_area / 10.0).clamp(0.0, 1.0);
            (normalized_area * 255.0).round() as u8
        };

        ByteSil::new(rho, theta)
    }

    fn target_layer(&self) -> u8 {
        4 // L4
    }

    fn calibrate(&mut self) -> Result<(), SensorError> {
        // Calibração: estabelece baseline de não-toque
        self.ready = false;

        // Reset de contadores
        self.touch_count = 0;
        self.last_contact_area = 0.0;

        // Em produção, faria calibração do sensor capacitivo/resistivo
        // Mock: apenas marca como pronto
        self.ready = true;

        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// IMPLEMENTAÇÃO AUXILIAR
// ═══════════════════════════════════════════════════════════════════════════════

impl TouchSensor {
    /// Gera dados mock para testes
    fn generate_mock_data(&mut self) -> HapticData {
        let mut readings = Vec::with_capacity(self.config.buffer_size);
        let mut last_area = 0.0;

        for i in 0..self.config.buffer_size {
            let (pressure, _contact_area) = self.simulate_touch(i);
            let temperature = self.simulate_skin_temperature();
            let vibration = if self.config.vibration_detection {
                self.simulate_vibration(i)
            } else {
                Vibration::new(0.0, 0.0)
            };

            let reading = HapticReading::new(
                Pressure::new(pressure),
                Temperature::new(temperature),
                vibration,
            );

            // Atualiza área de contato durante o loop
            last_area = self.estimate_contact_area_value(pressure);

            readings.push(reading);
        }

        // Atualiza última área de contato
        self.last_contact_area = last_area;

        HapticData::new(readings, self.config.sample_rate)
    }

    /// Simula toque com pressão variável (mock)
    fn simulate_touch(&self, sample_index: usize) -> (f32, f32) {
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as f32;

        // Pressão atmosférica base
        let base = 101_325.0;

        // Simula toque periódico (press and release)
        let touch_phase = ((now / 1000.0) + sample_index as f32 / 10.0).sin();
        let is_pressing = touch_phase > 0.0;

        let pressure = if is_pressing {
            // Toque ativo: adiciona pressão proporcional
            let touch_intensity = touch_phase * 20_000.0; // Até 20kPa
            base + touch_intensity
        } else {
            // Sem toque: apenas pressão atmosférica com ruído
            base + ((now * 0.1).sin() * 50.0)
        };

        // Área de contato correlacionada com pressão
        let contact_area = if is_pressing {
            (touch_phase * 5.0).max(0.0) // 0-5 cm²
        } else {
            0.0
        };

        (pressure, contact_area)
    }

    /// Simula temperatura da pele (mock)
    fn simulate_skin_temperature(&self) -> f32 {
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as f32;

        // Temperatura corporal típica na pele: 32-34°C
        let base = 33.0;

        // Pequena variação
        let variation = (now / 30.0).sin() * 1.0;

        base + variation
    }

    /// Simula vibração tátil (mock)
    fn simulate_vibration(&self, sample_index: usize) -> Vibration {
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as f32;

        // Frequência varia entre 50-300 Hz (faixa típica de toque)
        let freq = 150.0 + ((now / 200.0 + sample_index as f32).sin() * 100.0);

        // Amplitude varia com o tempo
        let amplitude = ((now / 500.0).cos().abs() * 0.6).max(0.1);

        Vibration::new(freq, amplitude)
    }

    /// Estima área de contato baseada na pressão (mock)
    fn estimate_contact_area_value(&self, pressure_pa: f32) -> f32 {
        // Modelo simplificado: mais pressão = maior área
        let excess_pressure = (pressure_pa - 101_325.0).max(0.0);
        let area = (excess_pressure / 5000.0).min(10.0); // Max 10 cm²
        area
    }
}
