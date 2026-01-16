//! Testes do módulo sil-photonic

use super::*;
use sil_core::traits::{Sensor, SilComponent};

// ═══════════════════════════════════════════════════════════════════════════════
// TESTES DE CAMERA
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_camera_creation() {
    let camera = CameraSensor::new();
    assert!(camera.is_ok());

    let camera = camera.unwrap();
    assert_eq!(camera.name(), "CameraSensor");
    assert_eq!(camera.layers(), &[0]);
    assert!(camera.is_ready());
}

#[test]
fn test_camera_with_config() {
    let config = CameraConfig {
        width: 1920,
        height: 1080,
        fps: 60,
        auto_exposure: false,
    };

    let camera = CameraSensor::with_config(config).unwrap();
    assert_eq!(camera.resolution(), (1920, 1080));
    assert_eq!(camera.fps(), 60);
}

#[test]
fn test_camera_invalid_config() {
    let config = CameraConfig {
        width: 0,
        height: 0,
        fps: 30,
        auto_exposure: true,
    };

    let result = CameraSensor::with_config(config);
    assert!(result.is_err());
}

#[test]
fn test_camera_read() {
    let mut camera = CameraSensor::new().unwrap();
    let result = camera.read();

    assert!(result.is_ok());
    let image = result.unwrap();

    assert_eq!(image.width, 640);
    assert_eq!(image.height, 480);
    assert_eq!(image.pixels.len(), (640 * 480) as usize);
}

#[test]
fn test_camera_to_byte_sil() {
    let camera = CameraSensor::new().unwrap();
    let image = ImageData::black(100, 100);

    let byte = camera.to_byte_sil(&image);

    // Imagem preta: intensidade = 0 → rho = -8
    assert_eq!(byte.rho, -8);
}

#[test]
fn test_camera_sense() {
    let mut camera = CameraSensor::new().unwrap();
    let result = camera.sense();

    assert!(result.is_ok());
    let update = result.unwrap();

    // Deve ser para L0
    assert_eq!(update.layer, 0);
    assert!(update.byte.rho >= -8 && update.byte.rho <= 7);
}

#[test]
fn test_camera_frame_count() {
    let mut camera = CameraSensor::new().unwrap();
    assert_eq!(camera.frame_count(), 0);

    camera.read().unwrap();
    assert_eq!(camera.frame_count(), 1);

    camera.read().unwrap();
    camera.read().unwrap();
    assert_eq!(camera.frame_count(), 3);
}

#[test]
fn test_camera_calibrate() {
    let mut camera = CameraSensor::new().unwrap();
    let result = camera.calibrate();
    assert!(result.is_ok());
    assert!(camera.is_ready());
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTES DE LIGHT SENSOR
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_light_creation() {
    let sensor = LightSensor::new();
    assert!(sensor.is_ok());

    let sensor = sensor.unwrap();
    assert_eq!(sensor.name(), "LightSensor");
    assert_eq!(sensor.layers(), &[0]);
    assert!(sensor.is_ready());
}

#[test]
fn test_light_with_config() {
    let config = LightConfig {
        sensitivity: 0.8,
        sample_rate: 200.0,
    };

    let sensor = LightSensor::with_config(config);
    assert!(sensor.is_ok());
}

#[test]
fn test_light_invalid_sensitivity() {
    let config = LightConfig {
        sensitivity: 1.5, // Inválido
        sample_rate: 100.0,
    };

    let result = LightSensor::with_config(config);
    assert!(result.is_err());
}

#[test]
fn test_light_read() {
    let mut sensor = LightSensor::new().unwrap();
    let result = sensor.read();

    assert!(result.is_ok());
    let intensity = result.unwrap();

    // Deve estar em uma faixa razoável (10-900 lux)
    assert!(intensity.value >= 10.0 && intensity.value <= 900.0);
}

#[test]
fn test_light_to_byte_sil() {
    let sensor = LightSensor::new().unwrap();

    // Luz baixa
    let low = Intensity::new(0.0);
    let byte = sensor.to_byte_sil(&low);
    assert_eq!(byte.rho, -8);

    // Luz alta
    let high = Intensity::new(1000.0);
    let byte = sensor.to_byte_sil(&high);
    assert_eq!(byte.rho, 7);

    // Luz média
    let mid = Intensity::new(500.0);
    let byte = sensor.to_byte_sil(&mid);
    assert!(byte.rho >= -1 && byte.rho <= 0);
}

#[test]
fn test_light_sense() {
    let mut sensor = LightSensor::new().unwrap();
    let result = sensor.sense();

    assert!(result.is_ok());
    let update = result.unwrap();

    assert_eq!(update.layer, 0);
    assert!(update.byte.rho >= -8 && update.byte.rho <= 7);
}

#[test]
fn test_light_last_reading() {
    let mut sensor = LightSensor::new().unwrap();
    assert_eq!(sensor.last_reading(), 0.0);

    sensor.read().unwrap();
    assert!(sensor.last_reading() > 0.0);
}

#[test]
fn test_light_calibrate() {
    let mut sensor = LightSensor::new().unwrap();
    sensor.read().unwrap(); // Faz uma leitura

    sensor.calibrate().unwrap();
    assert_eq!(sensor.last_reading(), 0.0); // Reset após calibração
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTES DE TYPES
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_pixel() {
    let pixel = Pixel::new(128, 64, 192);
    assert_eq!(pixel.r, 128);
    assert_eq!(pixel.g, 64);
    assert_eq!(pixel.b, 192);

    let intensity = pixel.intensity();
    assert_eq!(intensity, 128); // (128+64+192)/3 = 128
}

#[test]
fn test_pixel_hue() {
    let red = Pixel::new(255, 0, 0);
    let hue = red.hue();
    assert!(hue >= 0.0 && hue < 60.0); // Vermelho ≈ 0°

    let green = Pixel::new(0, 255, 0);
    let hue = green.hue();
    assert!(hue >= 100.0 && hue < 140.0); // Verde ≈ 120°

    let blue = Pixel::new(0, 0, 255);
    let hue = blue.hue();
    assert!(hue >= 220.0 && hue < 260.0); // Azul ≈ 240°
}

#[test]
fn test_image_data() {
    let image = ImageData::black(10, 10);
    assert_eq!(image.width, 10);
    assert_eq!(image.height, 10);
    assert_eq!(image.pixels.len(), 100);
    assert_eq!(image.avg_intensity(), 0);
}

#[test]
fn test_image_get_pixel() {
    let mut image = ImageData::black(5, 5);
    image.pixels[12] = Pixel::new(255, 0, 0); // (2, 2) = índice 12

    let pixel = image.get(2, 2);
    assert!(pixel.is_some());
    assert_eq!(pixel.unwrap().r, 255);

    let out_of_bounds = image.get(10, 10);
    assert!(out_of_bounds.is_none());
}

#[test]
fn test_intensity() {
    let intensity = Intensity::new(500.0);
    assert_eq!(intensity.value, 500.0);
    assert_eq!(intensity.normalized(), 0.5);

    let high = Intensity::new(2000.0);
    assert_eq!(high.normalized(), 1.0); // Clamped
}
