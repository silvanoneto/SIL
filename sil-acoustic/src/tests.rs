//! Testes do módulo sil-acoustic

use super::*;
use sil_core::traits::{Sensor, SilComponent};

// ═══════════════════════════════════════════════════════════════════════════════
// TESTES DE MICROPHONE
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_microphone_creation() {
    let microphone = MicrophoneSensor::new();
    assert!(microphone.is_ok());

    let microphone = microphone.unwrap();
    assert_eq!(microphone.name(), "MicrophoneSensor");
    assert_eq!(microphone.layers(), &[1]); // L1 = Acústico
    assert!(microphone.is_ready());
}

#[test]
fn test_microphone_with_config() {
    let config = AudioConfig {
        sample_rate: 48000,
        buffer_size: 2048,
        channels: 2,
        bit_depth: 16,
    };

    let microphone = MicrophoneSensor::with_config(config).unwrap();
    assert_eq!(microphone.sample_rate(), 48000);
    assert_eq!(microphone.channels(), 2);
    assert_eq!(microphone.buffer_size(), 2048);
}

#[test]
fn test_microphone_invalid_sample_rate() {
    let config = AudioConfig {
        sample_rate: 1000, // Muito baixo
        buffer_size: 1024,
        channels: 1,
        bit_depth: 16,
    };

    let result = MicrophoneSensor::with_config(config);
    assert!(result.is_err());
}

#[test]
fn test_microphone_invalid_channels() {
    let config = AudioConfig {
        sample_rate: 44100,
        buffer_size: 1024,
        channels: 0, // Inválido
        bit_depth: 16,
    };

    let result = MicrophoneSensor::with_config(config);
    assert!(result.is_err());

    let config = AudioConfig {
        sample_rate: 44100,
        buffer_size: 1024,
        channels: 10, // Muito alto
        bit_depth: 16,
    };

    let result = MicrophoneSensor::with_config(config);
    assert!(result.is_err());
}

#[test]
fn test_microphone_invalid_buffer_size() {
    let config = AudioConfig {
        sample_rate: 44100,
        buffer_size: 0, // Inválido
        channels: 1,
        bit_depth: 16,
    };

    let result = MicrophoneSensor::with_config(config);
    assert!(result.is_err());
}

#[test]
fn test_microphone_read() {
    let mut microphone = MicrophoneSensor::new().unwrap();
    let result = microphone.read();

    assert!(result.is_ok());
    let audio = result.unwrap();

    assert_eq!(audio.sample_rate, 44100);
    assert_eq!(audio.channels, 1);
    assert!(!audio.samples.is_empty());
}

#[test]
fn test_microphone_to_byte_sil() {
    let microphone = MicrophoneSensor::new().unwrap();
    let audio = AudioData::silence(44100, 100, 1); // 100ms de silêncio

    let byte = microphone.to_byte_sil(&audio);

    // Silêncio deve ter rho = -8
    assert_eq!(byte.rho, -8);
    assert_eq!(byte.theta, 0); // Sem frequência dominante
}

#[test]
fn test_microphone_sense() {
    let mut microphone = MicrophoneSensor::new().unwrap();
    let result = microphone.sense();

    assert!(result.is_ok());
    let update = result.unwrap();

    // Deve ser para L1
    assert_eq!(update.layer, 1);
    assert!(update.byte.rho >= -8 && update.byte.rho <= 7);
}

#[test]
fn test_microphone_sample_count() {
    let mut microphone = MicrophoneSensor::new().unwrap();
    assert_eq!(microphone.sample_count(), 0);

    microphone.read().unwrap();
    let count_after_first = microphone.sample_count();
    assert!(count_after_first > 0);

    microphone.read().unwrap();
    assert!(microphone.sample_count() > count_after_first);
}

#[test]
fn test_microphone_calibrate() {
    let mut microphone = MicrophoneSensor::new().unwrap();
    microphone.read().unwrap(); // Faz uma leitura

    let result = microphone.calibrate();
    assert!(result.is_ok());
    assert!(microphone.is_ready());
    assert_eq!(microphone.sample_count(), 0); // Reset após calibração
}

#[test]
fn test_microphone_configure() {
    let mut microphone = MicrophoneSensor::new().unwrap();

    let new_config = AudioConfig {
        sample_rate: 48000,
        buffer_size: 2048,
        channels: 2,
        bit_depth: 24,
    };

    let result = microphone.configure(new_config);
    assert!(result.is_ok());
    assert_eq!(microphone.sample_rate(), 48000);
}

#[test]
fn test_microphone_default() {
    let microphone = MicrophoneSensor::default();
    assert_eq!(microphone.sample_rate(), 44100);
    assert_eq!(microphone.channels(), 1);
    assert!(microphone.is_ready());
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTES DE TYPES
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_audio_sample() {
    let sample = AudioSample::new(16384);
    assert_eq!(sample.value, 16384);

    let normalized = sample.normalized();
    assert!(normalized > 0.49 && normalized < 0.51); // ~0.5

    let amplitude = sample.amplitude();
    assert_eq!(amplitude, 16384);
}

#[test]
fn test_audio_sample_negative() {
    let sample = AudioSample::new(-16384);
    assert_eq!(sample.value, -16384);

    let normalized = sample.normalized();
    assert!(normalized < -0.49 && normalized > -0.51); // ~-0.5

    let amplitude = sample.amplitude();
    assert_eq!(amplitude, 16384); // Valor absoluto
}

#[test]
fn test_frequency_audible() {
    let freq = Frequency::new(440.0); // A4
    assert!(freq.is_audible());
    assert!(!freq.is_ultrasound());
    assert!(!freq.is_infrasound());

    let freq = Frequency::new(10_000.0); // 10kHz
    assert!(freq.is_audible());
}

#[test]
fn test_frequency_ultrasound() {
    let freq = Frequency::new(40_000.0);
    assert!(freq.is_ultrasound());
    assert!(!freq.is_audible());
    assert!(!freq.is_infrasound());
}

#[test]
fn test_frequency_infrasound() {
    let freq = Frequency::new(10.0);
    assert!(freq.is_infrasound());
    assert!(!freq.is_audible());
    assert!(!freq.is_ultrasound());
}

#[test]
fn test_amplitude_from_samples() {
    // Onda quadrada simples
    let samples = vec![16384, 16384, -16384, -16384];
    let amp = Amplitude::from_samples(&samples);

    assert!(amp.rms > 0.0 && amp.rms <= 1.0);
    assert!(amp.to_db() < 0.0); // Sempre negativo para amplitudes < 1.0
}

#[test]
fn test_amplitude_silence() {
    let samples = vec![0, 0, 0, 0];
    let amp = Amplitude::from_samples(&samples);

    assert_eq!(amp.rms, 0.0);
    assert_eq!(amp.to_db(), -96.0); // Limite de silêncio
    assert_eq!(amp.to_sil_rho(), -8);
}

#[test]
fn test_amplitude_to_sil_rho() {
    // Amplitude baixa
    let low = Amplitude::new(0.0);
    assert_eq!(low.to_sil_rho(), -8);

    // Amplitude alta
    let high = Amplitude::new(1.0);
    assert_eq!(high.to_sil_rho(), 7);

    // Amplitude média
    let mid = Amplitude::new(0.5);
    let rho = mid.to_sil_rho();
    assert!(rho >= -1 && rho <= 0);
}

#[test]
fn test_audio_data_silence() {
    let audio = AudioData::silence(44100, 1000, 1); // 1 segundo, mono
    assert_eq!(audio.sample_rate, 44100);
    assert_eq!(audio.channels, 1);
    assert_eq!(audio.duration_ms(), 1000);
    assert_eq!(audio.amplitude().rms, 0.0);
    assert_eq!(audio.peak(), 0);
    assert!(!audio.is_clipping());
}

#[test]
fn test_audio_data_stereo() {
    let audio = AudioData::silence(48000, 500, 2); // 500ms, estéreo
    assert_eq!(audio.sample_rate, 48000);
    assert_eq!(audio.channels, 2);
    assert_eq!(audio.duration_ms(), 500);
}

#[test]
fn test_audio_data_duration() {
    let audio = AudioData::new(44100, vec![0; 44100], 1); // 1 segundo
    assert_eq!(audio.duration_ms(), 1000);

    let audio = AudioData::new(48000, vec![0; 24000], 1); // 0.5 segundos
    assert_eq!(audio.duration_ms(), 500);
}

#[test]
fn test_audio_data_peak() {
    let samples = vec![100, -200, 50, -300, 150];
    let audio = AudioData::new(44100, samples, 1);

    assert_eq!(audio.peak(), 300); // Máximo absoluto
}

#[test]
fn test_audio_data_clipping() {
    let samples = vec![i16::MAX, 100, 200];
    let audio = AudioData::new(44100, samples, 1);

    assert!(audio.is_clipping());

    let samples = vec![100, 200, 300];
    let audio = AudioData::new(44100, samples, 1);

    assert!(!audio.is_clipping());
}

#[test]
fn test_audio_data_dominant_frequency() {
    // Cria uma onda simples com alguns zero-crossings
    let samples = vec![100, 200, 100, -100, -200, -100, 100, 200];
    let audio = AudioData::new(44100, samples, 1);

    let freq = audio.dominant_frequency();
    assert!(freq.hz >= 0.0); // Apenas verifica que retorna algo válido
}

#[test]
fn test_audio_config_default() {
    let config = AudioConfig::default();
    assert_eq!(config.sample_rate, 44100);
    assert_eq!(config.buffer_size, 1024);
    assert_eq!(config.channels, 1);
    assert_eq!(config.bit_depth, 16);
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTES DE INTEGRAÇÃO
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_full_pipeline() {
    let mut microphone = MicrophoneSensor::new().unwrap();

    // Lê dados
    let audio = microphone.read().unwrap();
    assert!(!audio.samples.is_empty());

    // Converte para ByteSil
    let byte = microphone.to_byte_sil(&audio);
    assert!(byte.rho >= -8 && byte.rho <= 7);

    // Usa sense() para fazer tudo de uma vez
    let update = microphone.sense().unwrap();
    assert_eq!(update.layer, 1);
}

#[test]
fn test_multiple_reads() {
    let mut microphone = MicrophoneSensor::new().unwrap();

    for i in 0..10 {
        let result = microphone.read();
        assert!(result.is_ok(), "Read {} failed", i);

        let audio = result.unwrap();
        assert!(!audio.samples.is_empty());
    }

    assert_eq!(microphone.sample_count(), 10 * 1024); // 10 buffers de 1024 amostras
}
