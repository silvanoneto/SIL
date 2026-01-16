//! Implementação de sensor gustativo (L3)

use serde::{Deserialize, Serialize};
use sil_core::prelude::*;
use sil_core::traits::{SilComponent, Sensor, SensorError};
use crate::error::{GustatoryError, GustatoryResult};
use crate::types::{PhLevel, TasteProfile, Salinity, TasteData};

/// Configuração do sensor gustativo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TasteSensorConfig {
    /// Sensibilidade geral (0.0 - 1.0)
    pub sensitivity: f32,
    /// Taxa de amostragem (Hz)
    pub sample_rate: f32,
    /// Calibração de pH (offset)
    pub ph_offset: f32,
    /// Calibração de salinidade (multiplicador)
    pub salinity_multiplier: f32,
}

impl Default for TasteSensorConfig {
    fn default() -> Self {
        Self {
            sensitivity: 0.5,
            sample_rate: 10.0,
            ph_offset: 0.0,
            salinity_multiplier: 1.0,
        }
    }
}

/// Sensor gustativo (L3) - detecta pH, gostos básicos e salinidade
#[derive(Debug, Clone)]
pub struct TasteSensor {
    config: TasteSensorConfig,
    ready: bool,
    last_reading: TasteData,
    sample_count: u64,
}

impl TasteSensor {
    /// Cria novo sensor gustativo
    pub fn new() -> GustatoryResult<Self> {
        Self::with_config(TasteSensorConfig::default())
    }

    /// Cria sensor com configuração específica
    pub fn with_config(config: TasteSensorConfig) -> GustatoryResult<Self> {
        if config.sensitivity < 0.0 || config.sensitivity > 1.0 {
            return Err(GustatoryError::InvalidConfig(
                "Sensitivity must be between 0.0 and 1.0".into(),
            ));
        }

        if config.sample_rate <= 0.0 {
            return Err(GustatoryError::InvalidConfig(
                "Sample rate must be positive".into(),
            ));
        }

        Ok(Self {
            config,
            ready: true,
            last_reading: TasteData::neutral(),
            sample_count: 0,
        })
    }

    /// Retorna última leitura
    pub fn last_reading(&self) -> &TasteData {
        &self.last_reading
    }

    /// Retorna número de amostras coletadas
    pub fn sample_count(&self) -> u64 {
        self.sample_count
    }

    /// Retorna configuração atual
    pub fn config(&self) -> &TasteSensorConfig {
        &self.config
    }
}

impl Default for TasteSensor {
    fn default() -> Self {
        Self::new().expect("Default TasteSensor creation should not fail")
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// IMPLEMENTAÇÃO DOS TRAITS DO CORE
// ═══════════════════════════════════════════════════════════════════════════════

impl SilComponent for TasteSensor {
    fn name(&self) -> &str {
        "TasteSensor"
    }

    fn layers(&self) -> &[u8] {
        &[3] // L3 = Gustativo
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn is_ready(&self) -> bool {
        self.ready
    }
}

impl Sensor for TasteSensor {
    type RawData = TasteData;
    type Config = TasteSensorConfig;

    fn configure(&mut self, config: Self::Config) -> Result<(), SensorError> {
        if config.sensitivity < 0.0 || config.sensitivity > 1.0 {
            return Err(SensorError::InvalidConfig(
                "Sensitivity must be between 0.0 and 1.0".into(),
            ));
        }

        if config.sample_rate <= 0.0 {
            return Err(SensorError::InvalidConfig(
                "Sample rate must be positive".into(),
            ));
        }

        self.config = config;
        Ok(())
    }

    fn read(&mut self) -> Result<Self::RawData, SensorError> {
        if !self.ready {
            return Err(SensorError::NotInitialized);
        }

        // Mock: simula leitura de sensor gustativo
        // Em produção, leria de sensores físicos (pH probe, electrochemical taste sensors)
        let taste_data = self.simulate_taste_reading();
        self.last_reading = taste_data.clone();
        self.sample_count += 1;

        Ok(taste_data)
    }

    fn to_byte_sil(&self, raw: &Self::RawData) -> ByteSil {
        // ρ (rho): pH level normalizado [-8, 7]
        // pH 0-14 mapeia para rho -8 a +7
        let rho = raw.ph.to_rho();

        // θ (theta): tipo de gosto dominante [0, 255]
        let (dominant_taste, intensity) = raw.profile.dominant();
        let base_theta = dominant_taste.to_theta();

        // Ajusta theta baseado na intensidade (adiciona offset dentro do range do gosto)
        // Cada gosto ocupa ~51 unidades, então ajustamos proporcionalmente
        let intensity_offset = (intensity * 50.0) as u8;
        let theta = base_theta.saturating_add(intensity_offset).min(255);

        ByteSil::new(rho, theta)
    }

    fn target_layer(&self) -> u8 {
        3 // L3 = Gustativo
    }

    fn sample_rate(&self) -> f32 {
        self.config.sample_rate
    }

    fn calibrate(&mut self) -> Result<(), SensorError> {
        // Calibração: reseta para água neutra e ajusta offsets
        self.ready = true;
        self.last_reading = TasteData::neutral();
        self.sample_count = 0;

        // Em produção, faria calibração física com soluções de referência
        // (pH 4.0, 7.0, 10.0 buffers, soluções padrão de NaCl, etc.)

        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// IMPLEMENTAÇÃO AUXILIAR
// ═══════════════════════════════════════════════════════════════════════════════

impl TasteSensor {
    /// Simula leitura de sensor gustativo (mock para desenvolvimento)
    fn simulate_taste_reading(&self) -> TasteData {
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Simula variação temporal usando diferentes frequências para cada parâmetro
        let time = now as f32;

        // pH varia entre 4.0 (ácido) e 10.0 (básico), centrando em 7.0
        let ph_base = 7.0;
        let ph_variation = 2.5 * ((time / 120.0) * 2.0 * std::f32::consts::PI).sin();
        let ph_value = (ph_base + ph_variation + self.config.ph_offset).clamp(0.0, 14.0);
        let ph = PhLevel::clamped(ph_value);

        // Perfil de gosto varia com diferentes frequências
        let sweet = ((time / 30.0).sin() * 0.5 + 0.5) * self.config.sensitivity;
        let sour = ((time / 45.0 + 1.0).sin() * 0.5 + 0.5) * self.config.sensitivity;
        let salty = ((time / 60.0 + 2.0).sin() * 0.5 + 0.5) * self.config.sensitivity;
        let bitter = ((time / 75.0 + 3.0).sin() * 0.5 + 0.5) * self.config.sensitivity;
        let umami = ((time / 90.0 + 4.0).sin() * 0.5 + 0.5) * self.config.sensitivity;

        let profile = TasteProfile {
            sweet: sweet.clamp(0.0, 1.0),
            sour: sour.clamp(0.0, 1.0),
            salty: salty.clamp(0.0, 1.0),
            bitter: bitter.clamp(0.0, 1.0),
            umami: umami.clamp(0.0, 1.0),
        };

        // Salinidade varia entre 0 (água doce) e 35000 (água do mar)
        let salinity_base = 5000.0; // Ligeiramente salobra
        let salinity_variation = 10000.0 * ((time / 150.0) * 2.0 * std::f32::consts::PI).sin();
        let salinity_ppm = ((salinity_base + salinity_variation) * self.config.salinity_multiplier)
            .max(0.0);
        let salinity = Salinity::new(salinity_ppm);

        // Condutividade correlaciona com TDS (aproximadamente 2:1 ratio)
        // TDS (mg/L) ≈ 0.5 * conductivity (µS/cm)
        let conductivity = salinity_ppm * 2.0; // µS/cm
        let tds = salinity_ppm; // ppm (mg/L)

        TasteData::full(
            ph,
            profile,
            salinity,
            Some(conductivity),
            Some(tds),
        )
    }
}
