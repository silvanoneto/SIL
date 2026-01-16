//! Sensor climático implementando o trait Sensor (L7)

use serde::{Deserialize, Serialize};
use sil_core::prelude::*;
use sil_core::traits::{SilComponent, Sensor, SensorError};
use crate::error::{EnvironmentError, EnvironmentResult};
use crate::types::EnvironmentData;

/// Configuração do sensor climático
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClimateConfig {
    /// Taxa de amostragem em Hz (0 = sob demanda)
    pub sample_rate: f32,
    /// Habilitar calibração automática
    pub auto_calibrate: bool,
    /// Intervalo de calibração em segundos
    pub calibration_interval: u64,
    /// Aplicar filtro de média móvel
    pub enable_smoothing: bool,
    /// Tamanho da janela de suavização
    pub smoothing_window: usize,
}

impl Default for ClimateConfig {
    fn default() -> Self {
        Self {
            sample_rate: 1.0, // 1 Hz - uma leitura por segundo
            auto_calibrate: true,
            calibration_interval: 3600, // 1 hora
            enable_smoothing: true,
            smoothing_window: 5,
        }
    }
}

/// Sensor climático que captura dados ambientais (L7)
///
/// O ClimateSensor implementa o trait Sensor e captura múltiplos
/// parâmetros ambientais, convertendo-os para o formato ByteSil:
///
/// - ρ (magnitude): score de conforto normalizado para [-8, 7]
/// - θ (fase): índice de qualidade do ar [0, 255]
#[derive(Debug, Clone)]
pub struct ClimateSensor {
    config: ClimateConfig,
    ready: bool,
    sample_count: u64,
    last_calibration: u64,
    // Buffer para suavização
    temperature_buffer: Vec<f32>,
    humidity_buffer: Vec<f32>,
    // Offsets de calibração
    temp_offset: f32,
    humidity_offset: f32,
}

impl ClimateSensor {
    /// Cria novo sensor climático com configuração padrão
    pub fn new() -> EnvironmentResult<Self> {
        Self::with_config(ClimateConfig::default())
    }

    /// Cria sensor com configuração específica
    pub fn with_config(config: ClimateConfig) -> EnvironmentResult<Self> {
        if config.sample_rate < 0.0 {
            return Err(EnvironmentError::InvalidConfig(
                "Sample rate must be non-negative".into(),
            ));
        }

        if config.smoothing_window == 0 {
            return Err(EnvironmentError::InvalidConfig(
                "Smoothing window must be > 0".into(),
            ));
        }

        let smoothing_window = config.smoothing_window;

        Ok(Self {
            config,
            ready: false,
            sample_count: 0,
            last_calibration: 0,
            temperature_buffer: Vec::with_capacity(smoothing_window),
            humidity_buffer: Vec::with_capacity(smoothing_window),
            temp_offset: 0.0,
            humidity_offset: 0.0,
        })
    }

    /// Retorna contagem de amostras
    pub fn sample_count(&self) -> u64 {
        self.sample_count
    }

    /// Verifica se precisa calibrar
    fn should_calibrate(&self) -> bool {
        if !self.config.auto_calibrate {
            return false;
        }

        let now = Self::current_timestamp();
        now - self.last_calibration > self.config.calibration_interval
    }

    /// Timestamp atual em segundos
    fn current_timestamp() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    /// Aplica suavização aos dados
    fn smooth_temperature(&mut self, raw_temp: f32) -> f32 {
        if !self.config.enable_smoothing {
            return raw_temp;
        }

        self.temperature_buffer.push(raw_temp);
        if self.temperature_buffer.len() > self.config.smoothing_window {
            self.temperature_buffer.remove(0);
        }

        self.temperature_buffer.iter().sum::<f32>() / self.temperature_buffer.len() as f32
    }

    /// Aplica suavização à umidade
    fn smooth_humidity(&mut self, raw_humidity: f32) -> f32 {
        if !self.config.enable_smoothing {
            return raw_humidity;
        }

        self.humidity_buffer.push(raw_humidity);
        if self.humidity_buffer.len() > self.config.smoothing_window {
            self.humidity_buffer.remove(0);
        }

        self.humidity_buffer.iter().sum::<f32>() / self.humidity_buffer.len() as f32
    }

    /// Gera dados ambientais sintéticos para testes
    ///
    /// Em produção, isso seria substituído por leituras reais de hardware.
    /// Os valores gerados simulam condições ambientais variáveis mas realistas.
    fn generate_mock_data(&mut self) -> EnvironmentData {
        use std::f32::consts::PI;

        // Simula variação diurna usando seno
        let cycle = (self.sample_count as f32 * 0.1) % (2.0 * PI);

        // Temperatura: 18-26°C com variação diurna
        let raw_temp = 22.0 + 4.0 * cycle.sin() + self.temp_offset;
        let temperature = self.smooth_temperature(raw_temp);

        // Umidade: 40-70% inversamente correlacionada com temperatura
        let raw_humidity = 55.0 - 15.0 * cycle.sin() + self.humidity_offset;
        let humidity = self.smooth_humidity(raw_humidity);

        // Pressão: 1010-1020 hPa com variação lenta
        let pressure = 1013.25 + 3.5 * (cycle * 0.5).sin();

        // Qualidade do ar: 20-80 AQI com picos ocasionais
        let aqi_base = 40.0 + 20.0 * (cycle * 0.3).cos();
        let air_quality = if self.sample_count % 100 == 0 {
            aqi_base + 40.0 // Pico ocasional
        } else {
            aqi_base
        };

        // CO2: 400-800 ppm
        let co2_ppm = 500.0 + 150.0 * (cycle * 0.7).sin();

        // VOC: 20-150 ppb
        let voc_ppb = 50.0 + 50.0 * (cycle * 0.4).cos();

        // PM2.5: 5-25 µg/m³
        let pm25 = 12.0 + 8.0 * (cycle * 0.6).sin();

        // PM10: 10-40 µg/m³
        let pm10 = 20.0 + 15.0 * (cycle * 0.5).cos();

        EnvironmentData {
            temperature,
            humidity,
            pressure,
            air_quality,
            co2_ppm,
            voc_ppb,
            pm25,
            pm10,
        }
    }

    /// Retorna dados da última leitura (para debug)
    pub fn last_data(&mut self) -> EnvironmentResult<EnvironmentData> {
        if !self.ready {
            return Err(EnvironmentError::NotInitialized);
        }
        Ok(self.generate_mock_data())
    }
}

impl Default for ClimateSensor {
    fn default() -> Self {
        Self::new().expect("Default ClimateSensor creation should not fail")
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// IMPLEMENTAÇÃO DOS TRAITS DO CORE
// ═══════════════════════════════════════════════════════════════════════════════

impl SilComponent for ClimateSensor {
    fn name(&self) -> &str {
        "ClimateSensor"
    }

    fn layers(&self) -> &[u8] {
        &[7] // L7 = Ambiental
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn is_ready(&self) -> bool {
        self.ready
    }
}

impl Sensor for ClimateSensor {
    type RawData = EnvironmentData;
    type Config = ClimateConfig;

    fn configure(&mut self, config: Self::Config) -> Result<(), SensorError> {
        if config.sample_rate < 0.0 {
            return Err(SensorError::InvalidConfig(
                "Sample rate must be non-negative".into(),
            ));
        }

        if config.smoothing_window == 0 {
            return Err(SensorError::InvalidConfig(
                "Smoothing window must be > 0".into(),
            ));
        }

        self.config = config;
        Ok(())
    }

    fn read(&mut self) -> Result<Self::RawData, SensorError> {
        if !self.ready {
            return Err(SensorError::NotInitialized);
        }

        // Verificar se precisa calibrar
        if self.should_calibrate() {
            self.calibrate()?;
        }

        // Gerar dados (em produção, ler do hardware)
        let data = self.generate_mock_data();
        self.sample_count += 1;

        Ok(data)
    }

    fn to_byte_sil(&self, raw: &Self::RawData) -> ByteSil {
        // ρ (magnitude): score de conforto normalizado para [-8, 7]
        // comfort_score retorna 0.0-1.0, mapeamos para -8 a +7
        let comfort = raw.comfort_score();
        let rho = ((comfort * 15.0) - 8.0) as i8;

        // θ (fase): índice de qualidade do ar [0, 255]
        // composite_aqi retorna 0-500, mapeamos para 0-255
        let aqi = raw.composite_aqi();
        let theta = ((aqi / 500.0) * 255.0) as u8;

        ByteSil::new(rho.clamp(-8, 7), theta)
    }

    fn target_layer(&self) -> u8 {
        7 // L7
    }

    fn sample_rate(&self) -> f32 {
        self.config.sample_rate
    }

    fn calibrate(&mut self) -> Result<(), SensorError> {
        // Calibração mock: ajusta offsets para simular drift
        // Em produção, isso faria calibração real do hardware

        // Reseta buffers
        self.temperature_buffer.clear();
        self.humidity_buffer.clear();

        // Simula pequeno drift nos offsets (±0.5°C, ±2%)
        self.temp_offset = 0.5 * ((self.sample_count % 10) as f32 - 5.0) / 5.0;
        self.humidity_offset = 2.0 * ((self.sample_count % 10) as f32 - 5.0) / 5.0;

        self.last_calibration = Self::current_timestamp();
        self.ready = true;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_sensor() {
        let sensor = ClimateSensor::new();
        assert!(sensor.is_ok());
    }

    #[test]
    fn test_sensor_not_ready_by_default() {
        let sensor = ClimateSensor::new().unwrap();
        assert!(!sensor.is_ready());
    }

    #[test]
    fn test_calibrate_makes_ready() {
        let mut sensor = ClimateSensor::new().unwrap();
        assert!(sensor.calibrate().is_ok());
        assert!(sensor.is_ready());
    }

    #[test]
    fn test_read_before_calibration_fails() {
        let mut sensor = ClimateSensor::new().unwrap();
        let result = sensor.read();
        assert!(result.is_err());
    }

    #[test]
    fn test_read_after_calibration() {
        let mut sensor = ClimateSensor::new().unwrap();
        sensor.calibrate().unwrap();
        let result = sensor.read();
        assert!(result.is_ok());
    }

    #[test]
    fn test_sample_count_increments() {
        let mut sensor = ClimateSensor::new().unwrap();
        sensor.calibrate().unwrap();

        assert_eq!(sensor.sample_count(), 0);
        sensor.read().unwrap();
        assert_eq!(sensor.sample_count(), 1);
        sensor.read().unwrap();
        assert_eq!(sensor.sample_count(), 2);
    }

    #[test]
    fn test_to_byte_sil_ranges() {
        let sensor = ClimateSensor::new().unwrap();
        let data = EnvironmentData::default();
        let byte = sensor.to_byte_sil(&data);

        // ρ deve estar em [-8, 7]
        assert!(byte.rho >= -8 && byte.rho <= 7);
        // θ deve estar em [0, 255]
        assert!(byte.theta <= 255);
    }

    #[test]
    fn test_to_byte_sil_good_environment() {
        let sensor = ClimateSensor::new().unwrap();
        let data = EnvironmentData::default_ideal();
        let byte = sensor.to_byte_sil(&data);

        // Ambiente ideal deve ter ρ positivo (alto conforto)
        assert!(byte.rho > 0, "Ideal environment should have positive rho");
        // AQI baixo deve resultar em θ baixo
        assert!(byte.theta < 100, "Good air quality should have low theta");
    }

    #[test]
    fn test_to_byte_sil_poor_environment() {
        let sensor = ClimateSensor::new().unwrap();
        let mut data = EnvironmentData::default();
        data.temperature = 40.0; // Muito quente
        data.air_quality = 200.0; // Péssima qualidade
        let byte = sensor.to_byte_sil(&data);

        // Ambiente ruim deve ter ρ baixo (baixo conforto)
        assert!(byte.rho <= 2, "Poor environment should have low rho");
        // Verificar que theta está no range válido
        assert!(byte.theta <= 255);
    }

    #[test]
    fn test_configure() {
        let mut sensor = ClimateSensor::new().unwrap();
        let config = ClimateConfig {
            sample_rate: 2.0,
            ..Default::default()
        };
        assert!(sensor.configure(config).is_ok());
        assert_eq!(sensor.sample_rate(), 2.0);
    }

    #[test]
    fn test_configure_invalid_sample_rate() {
        let mut sensor = ClimateSensor::new().unwrap();
        let config = ClimateConfig {
            sample_rate: -1.0,
            ..Default::default()
        };
        assert!(sensor.configure(config).is_err());
    }

    #[test]
    fn test_configure_invalid_window() {
        let config = ClimateConfig {
            smoothing_window: 0,
            ..Default::default()
        };
        let result = ClimateSensor::with_config(config);
        assert!(result.is_err());
    }

    #[test]
    fn test_smoothing_enabled() {
        let config = ClimateConfig {
            enable_smoothing: true,
            smoothing_window: 3,
            ..Default::default()
        };
        let mut sensor = ClimateSensor::with_config(config).unwrap();
        sensor.calibrate().unwrap();

        // Ler múltiplas amostras
        for _ in 0..5 {
            sensor.read().unwrap();
        }

        // Verificar que buffer foi preenchido
        assert!(!sensor.temperature_buffer.is_empty());
        assert!(sensor.temperature_buffer.len() <= 3);
    }

    #[test]
    fn test_smoothing_disabled() {
        let config = ClimateConfig {
            enable_smoothing: false,
            ..Default::default()
        };
        let mut sensor = ClimateSensor::with_config(config).unwrap();
        sensor.calibrate().unwrap();

        sensor.read().unwrap();
        // Buffer não deve ser usado
        assert!(sensor.temperature_buffer.is_empty());
    }

    #[test]
    fn test_sense_method() {
        let mut sensor = ClimateSensor::new().unwrap();
        sensor.calibrate().unwrap();

        let update = sensor.sense().unwrap();
        assert_eq!(update.layer, 7);
        assert_eq!(update.confidence, 1.0);
    }

    #[test]
    fn test_target_layer() {
        let sensor = ClimateSensor::new().unwrap();
        assert_eq!(sensor.target_layer(), 7);
    }

    #[test]
    fn test_component_trait() {
        let sensor = ClimateSensor::new().unwrap();
        assert_eq!(sensor.name(), "ClimateSensor");
        assert_eq!(sensor.layers(), &[7]);
    }

    #[test]
    fn test_last_data() {
        let mut sensor = ClimateSensor::new().unwrap();
        sensor.calibrate().unwrap();
        let data = sensor.last_data();
        assert!(data.is_ok());
    }

    #[test]
    fn test_last_data_not_ready() {
        let mut sensor = ClimateSensor::new().unwrap();
        let data = sensor.last_data();
        assert!(data.is_err());
    }
}
