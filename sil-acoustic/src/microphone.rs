//! Implementação de sensor de microfone (L1)

use serde::{Deserialize, Serialize};
use sil_core::prelude::*;
use sil_core::traits::{SilComponent, Sensor, SensorError};
use crate::error::{AcousticError, AcousticResult};
use crate::types::AudioData;

/// Configuração de microfone
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub sample_rate: u32,
    pub buffer_size: usize,
    pub channels: u8,
    pub bit_depth: u8,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            buffer_size: 1024,
            channels: 1, // Mono
            bit_depth: 16,
        }
    }
}

/// Sensor de microfone acústico (L1)
#[derive(Debug, Clone)]
pub struct MicrophoneSensor {
    config: AudioConfig,
    ready: bool,
    sample_count: u64,
}

impl MicrophoneSensor {
    /// Cria novo microfone com configuração padrão
    pub fn new() -> AcousticResult<Self> {
        Self::with_config(AudioConfig::default())
    }

    /// Cria microfone com configuração específica
    pub fn with_config(config: AudioConfig) -> AcousticResult<Self> {
        if config.sample_rate < 8000 || config.sample_rate > 192_000 {
            return Err(AcousticError::InvalidSampleRate(config.sample_rate));
        }

        if config.buffer_size == 0 {
            return Err(AcousticError::InvalidConfig(
                "Buffer size must be > 0".into(),
            ));
        }

        if config.channels == 0 || config.channels > 8 {
            return Err(AcousticError::InvalidConfig(
                "Channels must be between 1 and 8".into(),
            ));
        }

        Ok(Self {
            config,
            ready: true,
            sample_count: 0,
        })
    }

    /// Retorna taxa de amostragem atual
    pub fn sample_rate(&self) -> u32 {
        self.config.sample_rate
    }

    /// Retorna número de canais
    pub fn channels(&self) -> u8 {
        self.config.channels
    }

    /// Retorna tamanho do buffer
    pub fn buffer_size(&self) -> usize {
        self.config.buffer_size
    }

    /// Número de amostras capturadas
    pub fn sample_count(&self) -> u64 {
        self.sample_count
    }
}

impl Default for MicrophoneSensor {
    fn default() -> Self {
        Self::new().expect("Default MicrophoneSensor creation should not fail")
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// IMPLEMENTAÇÃO DOS TRAITS DO CORE
// ═══════════════════════════════════════════════════════════════════════════════

impl SilComponent for MicrophoneSensor {
    fn name(&self) -> &str {
        "MicrophoneSensor"
    }

    fn layers(&self) -> &[u8] {
        &[1] // L1 = Acústico
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn is_ready(&self) -> bool {
        self.ready
    }
}

impl Sensor for MicrophoneSensor {
    type RawData = AudioData;
    type Config = AudioConfig;

    fn configure(&mut self, config: Self::Config) -> Result<(), SensorError> {
        if config.sample_rate < 8000 || config.sample_rate > 192_000 {
            return Err(SensorError::InvalidConfig(format!(
                "Sample rate must be between 8000 and 192000, got {}",
                config.sample_rate
            )));
        }

        if config.buffer_size == 0 {
            return Err(SensorError::InvalidConfig(
                "Buffer size must be > 0".into(),
            ));
        }

        if config.channels == 0 || config.channels > 8 {
            return Err(SensorError::InvalidConfig(
                "Channels must be between 1 and 8".into(),
            ));
        }

        self.config = config;
        Ok(())
    }

    fn read(&mut self) -> Result<Self::RawData, SensorError> {
        if !self.ready {
            return Err(SensorError::NotInitialized);
        }

        // Mock: gera áudio sintético
        // Em produção, aqui faria a captura real do microfone
        let data = self.generate_mock_audio();
        self.sample_count += data.samples.len() as u64;

        Ok(data)
    }

    fn to_byte_sil(&self, raw: &Self::RawData) -> ByteSil {
        // ρ (magnitude): amplitude/volume normalizada para [-8, 7]
        let amplitude = raw.amplitude();
        let rho = amplitude.to_sil_rho();

        // θ (fase): frequência dominante mapeada para [0, 255]
        let frequency = raw.dominant_frequency();
        let theta = if frequency.hz > 0.0 {
            // Mapeia 0-20kHz para 0-255
            let normalized = (frequency.hz / 20_000.0).clamp(0.0, 1.0);
            (normalized * 255.0) as u8
        } else {
            0
        };

        ByteSil::new(rho, theta)
    }

    fn target_layer(&self) -> u8 {
        1 // L1
    }

    fn calibrate(&mut self) -> Result<(), SensorError> {
        // Calibração mock: em produção, ajustaria ganho, ruído de fundo, etc.
        self.ready = true;
        self.sample_count = 0;
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// IMPLEMENTAÇÃO AUXILIAR
// ═══════════════════════════════════════════════════════════════════════════════

impl MicrophoneSensor {
    /// Gera áudio mock para testes
    fn generate_mock_audio(&self) -> AudioData {
        use std::f32::consts::PI;

        let mut samples = Vec::with_capacity(self.config.buffer_size * self.config.channels as usize);

        // Gera uma onda senoidal com frequência variável baseada no tempo
        let base_freq = 440.0; // A4
        let sample_rate = self.config.sample_rate as f32;

        for i in 0..self.config.buffer_size {
            // Adiciona variação temporal usando sample_count
            let t = (self.sample_count + i as u64) as f32 / sample_rate;

            // Frequência varia de 200Hz a 2000Hz
            let freq = base_freq + 200.0 * (t * 0.5).sin();

            // Amplitude varia de 0.1 a 0.8
            let amplitude = 0.45 + 0.35 * (t * 0.3).cos();

            // Gera amostra
            let value = (amplitude * (2.0 * PI * freq * t).sin() * i16::MAX as f32) as i16;

            // Adiciona para cada canal
            for _ in 0..self.config.channels {
                samples.push(value);
            }
        }

        AudioData::new(
            self.config.sample_rate,
            samples,
            self.config.channels,
        )
    }
}
