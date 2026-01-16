//! Tipos de dados acústicos

use serde::{Deserialize, Serialize};

/// Amostra de áudio individual (16-bit PCM)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudioSample {
    pub value: i16,
}

impl AudioSample {
    pub fn new(value: i16) -> Self {
        Self { value }
    }

    /// Retorna amplitude normalizada [-1.0, 1.0]
    pub fn normalized(&self) -> f32 {
        self.value as f32 / i16::MAX as f32
    }

    /// Retorna amplitude absoluta
    pub fn amplitude(&self) -> u16 {
        self.value.unsigned_abs()
    }
}

/// Frequência em Hz
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Frequency {
    /// Valor em Hz
    pub hz: f32,
}

impl Frequency {
    pub fn new(hz: f32) -> Self {
        Self {
            hz: hz.clamp(0.0, 192_000.0), // Até 192kHz
        }
    }

    /// Retorna se está na faixa audível (20Hz - 20kHz)
    pub fn is_audible(&self) -> bool {
        self.hz >= 20.0 && self.hz <= 20_000.0
    }

    /// Retorna se é ultrassom (>20kHz)
    pub fn is_ultrasound(&self) -> bool {
        self.hz > 20_000.0
    }

    /// Retorna se é infrassom (<20Hz)
    pub fn is_infrasound(&self) -> bool {
        self.hz < 20.0 && self.hz > 0.0
    }
}

/// Amplitude de áudio
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Amplitude {
    /// Valor RMS normalizado [0.0, 1.0]
    pub rms: f32,
}

impl Amplitude {
    pub fn new(rms: f32) -> Self {
        Self {
            rms: rms.clamp(0.0, 1.0),
        }
    }

    /// Cria amplitude a partir de amostras PCM
    pub fn from_samples(samples: &[i16]) -> Self {
        if samples.is_empty() {
            return Self { rms: 0.0 };
        }

        let sum_squares: f64 = samples
            .iter()
            .map(|&s| {
                let normalized = s as f64 / i16::MAX as f64;
                normalized * normalized
            })
            .sum();

        let rms = (sum_squares / samples.len() as f64).sqrt();
        Self::new(rms as f32)
    }

    /// Retorna em decibéis (dB)
    pub fn to_db(&self) -> f32 {
        if self.rms <= 0.0 {
            return -96.0; // Silêncio (limite do 16-bit)
        }
        20.0 * self.rms.log10()
    }

    /// Normaliza para a faixa [-8, 7] do SIL
    pub fn to_sil_rho(&self) -> i8 {
        let normalized = self.rms * 15.0 - 8.0;
        normalized.round().clamp(-8.0, 7.0) as i8
    }
}

/// Dados de áudio capturados
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioData {
    pub sample_rate: u32,
    pub samples: Vec<i16>,
    pub channels: u8,
}

impl AudioData {
    /// Cria dados de áudio vazios (silêncio)
    pub fn silence(sample_rate: u32, duration_ms: u32, channels: u8) -> Self {
        let num_samples = (sample_rate * duration_ms / 1000) as usize;
        Self {
            sample_rate,
            samples: vec![0; num_samples * channels as usize],
            channels,
        }
    }

    /// Cria dados de áudio a partir de amostras
    pub fn new(sample_rate: u32, samples: Vec<i16>, channels: u8) -> Self {
        Self {
            sample_rate,
            samples,
            channels,
        }
    }

    /// Retorna duração em milissegundos
    pub fn duration_ms(&self) -> u32 {
        if self.sample_rate == 0 || self.channels == 0 {
            return 0;
        }
        ((self.samples.len() as u32 / self.channels as u32) * 1000) / self.sample_rate
    }

    /// Retorna amplitude RMS média
    pub fn amplitude(&self) -> Amplitude {
        Amplitude::from_samples(&self.samples)
    }

    /// Retorna frequência dominante (simplificada, sem FFT completa)
    pub fn dominant_frequency(&self) -> Frequency {
        if self.samples.len() < 2 {
            return Frequency::new(0.0);
        }

        // Detecção simplificada de zero-crossings
        let mut crossings = 0;
        for i in 1..self.samples.len() {
            if (self.samples[i - 1] < 0 && self.samples[i] >= 0)
                || (self.samples[i - 1] >= 0 && self.samples[i] < 0)
            {
                crossings += 1;
            }
        }

        // Frequência = (crossings / 2) / duração_segundos
        let duration_s = self.samples.len() as f32 / self.sample_rate as f32;
        let freq = (crossings as f32 / 2.0) / duration_s;

        Frequency::new(freq)
    }

    /// Retorna peak amplitude (valor máximo absoluto)
    pub fn peak(&self) -> i16 {
        self.samples
            .iter()
            .map(|&s| s.abs())
            .max()
            .unwrap_or(0)
    }

    /// Verifica se o áudio está clipping
    pub fn is_clipping(&self) -> bool {
        self.samples.iter().any(|&s| s.abs() >= i16::MAX - 10)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_sample() {
        let sample = AudioSample::new(16384);
        assert_eq!(sample.value, 16384);
        assert!((sample.normalized() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_frequency() {
        let freq = Frequency::new(440.0); // A4
        assert!(freq.is_audible());
        assert!(!freq.is_ultrasound());
        assert!(!freq.is_infrasound());

        let ultra = Frequency::new(40_000.0);
        assert!(ultra.is_ultrasound());
    }

    #[test]
    fn test_amplitude_from_samples() {
        let samples = vec![0, 16384, 0, -16384]; // Onda quadrada
        let amp = Amplitude::from_samples(&samples);
        assert!(amp.rms > 0.0 && amp.rms <= 1.0);
    }

    #[test]
    fn test_audio_data_silence() {
        let audio = AudioData::silence(44100, 1000, 2); // 1 segundo, estéreo
        assert_eq!(audio.sample_rate, 44100);
        assert_eq!(audio.channels, 2);
        assert_eq!(audio.duration_ms(), 1000);
        assert_eq!(audio.amplitude().rms, 0.0);
    }
}
