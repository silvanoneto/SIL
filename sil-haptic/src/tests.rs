//! Testes do módulo sil-haptic

use super::*;
use sil_core::traits::{Sensor, SilComponent};

// ═══════════════════════════════════════════════════════════════════════════════
// TESTES DE PRESSURE SENSOR
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_pressure_sensor_creation() {
    let sensor = PressureSensor::new();
    assert!(sensor.is_ok());

    let sensor = sensor.unwrap();
    assert_eq!(sensor.name(), "PressureSensor");
    assert_eq!(sensor.layers(), &[4]); // L4 = Háptico
    assert!(sensor.is_ready());
}

#[test]
fn test_pressure_sensor_with_config() {
    let config = PressureConfig {
        sample_rate: 500,
        buffer_size: 50,
        sensitivity: 0.9,
        temperature_compensation: true,
    };

    let sensor = PressureSensor::with_config(config).unwrap();
    assert_eq!(sensor.sample_rate(), 500);
}

#[test]
fn test_pressure_sensor_invalid_sample_rate() {
    let config = PressureConfig {
        sample_rate: 0, // Inválido
        buffer_size: 10,
        sensitivity: 0.8,
        temperature_compensation: true,
    };

    let result = PressureSensor::with_config(config);
    assert!(result.is_err());

    let config = PressureConfig {
        sample_rate: 20_000, // Muito alto
        buffer_size: 10,
        sensitivity: 0.8,
        temperature_compensation: true,
    };

    let result = PressureSensor::with_config(config);
    assert!(result.is_err());
}

#[test]
fn test_pressure_sensor_invalid_buffer_size() {
    let config = PressureConfig {
        sample_rate: 100,
        buffer_size: 0, // Inválido
        sensitivity: 0.8,
        temperature_compensation: true,
    };

    let result = PressureSensor::with_config(config);
    assert!(result.is_err());
}

#[test]
fn test_pressure_sensor_invalid_sensitivity() {
    let config = PressureConfig {
        sample_rate: 100,
        buffer_size: 10,
        sensitivity: 1.5, // Inválido
        temperature_compensation: true,
    };

    let result = PressureSensor::with_config(config);
    assert!(result.is_err());
}

#[test]
fn test_pressure_sensor_read() {
    let mut sensor = PressureSensor::new().unwrap();
    let result = sensor.read();

    assert!(result.is_ok());
    let data = result.unwrap();

    assert_eq!(data.sample_rate, 100);
    assert!(!data.is_empty());
    assert_eq!(data.len(), 10); // Buffer size padrão
}

#[test]
fn test_pressure_sensor_to_byte_sil() {
    let sensor = PressureSensor::new().unwrap();
    let data = HapticData::zeros(5, 100);

    let byte = sensor.to_byte_sil(&data);

    // Pressão atmosférica normal deve ter rho positivo
    assert!(byte.rho >= -8 && byte.rho <= 7);
    assert!(byte.theta <= 255);
}

#[test]
fn test_pressure_sensor_sense() {
    let mut sensor = PressureSensor::new().unwrap();
    let result = sensor.sense();

    assert!(result.is_ok());
    let update = result.unwrap();

    // Deve ser para L4
    assert_eq!(update.layer, 4);
    assert!(update.byte.rho >= -8 && update.byte.rho <= 7);
}

#[test]
fn test_pressure_sensor_reading_count() {
    let mut sensor = PressureSensor::new().unwrap();
    assert_eq!(sensor.reading_count(), 0);

    sensor.read().unwrap();
    assert_eq!(sensor.reading_count(), 10); // Buffer size padrão

    sensor.read().unwrap();
    assert_eq!(sensor.reading_count(), 20);
}

#[test]
fn test_pressure_sensor_calibrate() {
    let mut sensor = PressureSensor::new().unwrap();
    let result = sensor.calibrate();

    assert!(result.is_ok());
    assert!(sensor.is_ready());
    assert_eq!(sensor.reading_count(), 0); // Reset após calibração
}

#[test]
fn test_pressure_sensor_configure() {
    let mut sensor = PressureSensor::new().unwrap();

    let new_config = PressureConfig {
        sample_rate: 200,
        buffer_size: 30,
        sensitivity: 0.7,
        temperature_compensation: false,
    };

    let result = sensor.configure(new_config);
    assert!(result.is_ok());
    assert_eq!(sensor.sample_rate(), 200);
}

#[test]
fn test_pressure_sensor_default() {
    let sensor = PressureSensor::default();
    assert_eq!(sensor.sample_rate(), 100);
    assert!(sensor.is_ready());
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTES DE TOUCH SENSOR
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_touch_sensor_creation() {
    let sensor = TouchSensor::new();
    assert!(sensor.is_ok());

    let sensor = sensor.unwrap();
    assert_eq!(sensor.name(), "TouchSensor");
    assert_eq!(sensor.layers(), &[4]); // L4 = Háptico
    assert!(sensor.is_ready());
}

#[test]
fn test_touch_sensor_with_config() {
    let config = HapticConfig {
        sample_rate: 400,
        buffer_size: 30,
        touch_threshold: 10_000.0,
        vibration_detection: false,
    };

    let sensor = TouchSensor::with_config(config).unwrap();
    assert_eq!(sensor.sample_rate(), 400);
}

#[test]
fn test_touch_sensor_invalid_sample_rate() {
    let config = HapticConfig {
        sample_rate: 0, // Inválido
        buffer_size: 20,
        touch_threshold: 5000.0,
        vibration_detection: true,
    };

    let result = TouchSensor::with_config(config);
    assert!(result.is_err());
}

#[test]
fn test_touch_sensor_invalid_buffer_size() {
    let config = HapticConfig {
        sample_rate: 200,
        buffer_size: 2000, // Muito grande
        touch_threshold: 5000.0,
        vibration_detection: true,
    };

    let result = TouchSensor::with_config(config);
    assert!(result.is_err());
}

#[test]
fn test_touch_sensor_invalid_threshold() {
    let config = HapticConfig {
        sample_rate: 200,
        buffer_size: 20,
        touch_threshold: -1000.0, // Negativo
        vibration_detection: true,
    };

    let result = TouchSensor::with_config(config);
    assert!(result.is_err());
}

#[test]
fn test_touch_sensor_read() {
    let mut sensor = TouchSensor::new().unwrap();
    let result = sensor.read();

    assert!(result.is_ok());
    let data = result.unwrap();

    assert_eq!(data.sample_rate, 200);
    assert!(!data.is_empty());
    assert_eq!(data.len(), 20); // Buffer size padrão
}

#[test]
fn test_touch_sensor_to_byte_sil() {
    let sensor = TouchSensor::new().unwrap();
    let data = HapticData::zeros(5, 200);

    let byte = sensor.to_byte_sil(&data);

    assert!(byte.rho >= -8 && byte.rho <= 7);
    assert!(byte.theta <= 255);
}

#[test]
fn test_touch_sensor_sense() {
    let mut sensor = TouchSensor::new().unwrap();
    let result = sensor.sense();

    assert!(result.is_ok());
    let update = result.unwrap();

    assert_eq!(update.layer, 4);
    assert!(update.byte.rho >= -8 && update.byte.rho <= 7);
}

#[test]
fn test_touch_sensor_touch_detection() {
    let sensor = TouchSensor::new().unwrap();

    // Sem toque (apenas pressão atmosférica)
    let no_touch = HapticData::zeros(5, 200);
    assert!(!sensor.is_touching(&no_touch));

    // Com toque (pressão elevada)
    let readings = vec![HapticReading::new(
        Pressure::new(120_000.0), // Acima do limiar
        Temperature::new(33.0),
        Vibration::new(100.0, 0.5),
    )];
    let with_touch = HapticData::new(readings, 200);
    assert!(sensor.is_touching(&with_touch));
}

#[test]
fn test_touch_sensor_touch_count() {
    let mut sensor = TouchSensor::new().unwrap();
    assert_eq!(sensor.touch_count(), 0);

    // Simula algumas leituras
    for _ in 0..5 {
        sensor.read().unwrap();
    }

    // Touch count deve ter aumentado (depende dos dados mock)
    let count = sensor.touch_count();
    assert!(count <= 5); // No máximo 5 toques
}

#[test]
fn test_touch_sensor_calibrate() {
    let mut sensor = TouchSensor::new().unwrap();
    sensor.read().unwrap(); // Faz uma leitura

    let result = sensor.calibrate();
    assert!(result.is_ok());
    assert!(sensor.is_ready());
    assert_eq!(sensor.touch_count(), 0); // Reset após calibração
}

#[test]
fn test_touch_sensor_configure() {
    let mut sensor = TouchSensor::new().unwrap();

    let new_config = HapticConfig {
        sample_rate: 300,
        buffer_size: 40,
        touch_threshold: 8000.0,
        vibration_detection: false,
    };

    let result = sensor.configure(new_config);
    assert!(result.is_ok());
    assert_eq!(sensor.sample_rate(), 300);
}

#[test]
fn test_touch_sensor_default() {
    let sensor = TouchSensor::default();
    assert_eq!(sensor.sample_rate(), 200);
    assert!(sensor.is_ready());
}

#[test]
fn test_touch_sensor_contact_area() {
    let mut sensor = TouchSensor::new().unwrap();
    assert_eq!(sensor.last_contact_area(), 0.0);

    sensor.read().unwrap();

    // Área de contato deve ter sido atualizada
    let area = sensor.last_contact_area();
    assert!(area >= 0.0);
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTES DE TYPES
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_pressure() {
    let p = Pressure::new(101_325.0);
    assert_eq!(p.pa, 101_325.0);

    let normalized = p.normalized();
    assert!(normalized > 0.99 && normalized < 1.02);
}

#[test]
fn test_pressure_sil_conversion() {
    let low = Pressure::new(0.0);
    assert_eq!(low.to_sil_rho(), -8);

    let atm = Pressure::new(100_000.0);
    let rho = atm.to_sil_rho();
    assert!(rho >= 5 && rho <= 7);

    let high = Pressure::new(200_000.0);
    assert_eq!(high.to_sil_rho(), 7);
}

#[test]
fn test_temperature() {
    let t = Temperature::new(25.0);
    assert_eq!(t.celsius, 25.0);
    assert_eq!(t.to_fahrenheit(), 77.0);
    assert!((t.to_kelvin() - 298.15).abs() < 0.01);
}

#[test]
fn test_temperature_sil_conversion() {
    let cold = Temperature::new(-40.0);
    assert_eq!(cold.to_sil_theta(), 0);

    let hot = Temperature::new(100.0);
    assert_eq!(hot.to_sil_theta(), 255);

    let room = Temperature::new(20.0);
    let theta = room.to_sil_theta();
    assert!(theta > 100 && theta < 120);
}

#[test]
fn test_vibration() {
    let v = Vibration::new(250.0, 0.7);
    assert_eq!(v.hz, 250.0);
    assert_eq!(v.amplitude, 0.7);
    assert!(v.is_perceptible());

    let weak = Vibration::new(5.0, 0.01);
    assert!(!weak.is_perceptible());
}

#[test]
fn test_vibration_sil_conversion() {
    let v = Vibration::new(500.0, 0.8);

    let theta = v.to_sil_theta();
    assert!(theta > 120 && theta < 130); // 500/1000 * 255 ≈ 127

    let rho = v.to_sil_rho();
    assert!(rho >= 3 && rho <= 5); // 0.8 * 15 - 8 = 4
}

#[test]
fn test_haptic_reading() {
    let reading = HapticReading::new(
        Pressure::new(105_000.0),
        Temperature::new(30.0),
        Vibration::new(150.0, 0.6),
    );

    assert_eq!(reading.pressure.pa, 105_000.0);
    assert_eq!(reading.temperature.celsius, 30.0);
    assert_eq!(reading.vibration.hz, 150.0);
    assert!(reading.timestamp_ms > 0);
}

#[test]
fn test_haptic_reading_zero() {
    let zero = HapticReading::zero();

    assert_eq!(zero.pressure.pa, 101_325.0); // Atmosférica
    assert_eq!(zero.temperature.celsius, 20.0);
    assert_eq!(zero.vibration.hz, 0.0);
}

#[test]
fn test_haptic_data_empty() {
    let data = HapticData::empty(100);
    assert!(data.is_empty());
    assert_eq!(data.len(), 0);
    assert_eq!(data.sample_rate, 100);
}

#[test]
fn test_haptic_data_zeros() {
    let data = HapticData::zeros(10, 200);
    assert_eq!(data.len(), 10);
    assert!(!data.is_empty());
    assert_eq!(data.sample_rate, 200);
}

#[test]
fn test_haptic_data_push() {
    let mut data = HapticData::empty(100);
    assert_eq!(data.len(), 0);

    data.push(HapticReading::zero());
    assert_eq!(data.len(), 1);

    data.push(HapticReading::zero());
    assert_eq!(data.len(), 2);
}

#[test]
fn test_haptic_data_averages() {
    let readings = vec![
        HapticReading::new(
            Pressure::new(100_000.0),
            Temperature::new(20.0),
            Vibration::new(100.0, 0.4),
        ),
        HapticReading::new(
            Pressure::new(120_000.0),
            Temperature::new(30.0),
            Vibration::new(200.0, 0.6),
        ),
    ];

    let data = HapticData::new(readings, 100);

    let avg_p = data.avg_pressure();
    assert!((avg_p.pa - 110_000.0).abs() < 1.0);

    let avg_t = data.avg_temperature();
    assert!((avg_t.celsius - 25.0).abs() < 0.01);

    let avg_v = data.avg_vibration();
    assert!((avg_v.hz - 150.0).abs() < 1.0);
    assert!((avg_v.amplitude - 0.5).abs() < 0.01);
}

#[test]
fn test_haptic_data_latest() {
    let mut data = HapticData::empty(100);
    assert!(data.latest().is_none());

    let reading1 = HapticReading::zero();
    data.push(reading1);

    let reading2 = HapticReading::new(
        Pressure::new(110_000.0),
        Temperature::new(25.0),
        Vibration::new(150.0, 0.5),
    );
    data.push(reading2);

    let latest = data.latest().unwrap();
    assert_eq!(latest.pressure.pa, 110_000.0);
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTES DE INTEGRAÇÃO
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_full_pipeline_pressure() {
    let mut sensor = PressureSensor::new().unwrap();

    // Lê dados
    let data = sensor.read().unwrap();
    assert!(!data.is_empty());

    // Converte para ByteSil
    let byte = sensor.to_byte_sil(&data);
    assert!(byte.rho >= -8 && byte.rho <= 7);

    // Usa sense() para fazer tudo de uma vez
    let update = sensor.sense().unwrap();
    assert_eq!(update.layer, 4);
}

#[test]
fn test_full_pipeline_touch() {
    let mut sensor = TouchSensor::new().unwrap();

    // Lê dados
    let data = sensor.read().unwrap();
    assert!(!data.is_empty());

    // Converte para ByteSil
    let byte = sensor.to_byte_sil(&data);
    assert!(byte.rho >= -8 && byte.rho <= 7);

    // Usa sense() para fazer tudo de uma vez
    let update = sensor.sense().unwrap();
    assert_eq!(update.layer, 4);
}

#[test]
fn test_multiple_reads_pressure() {
    let mut sensor = PressureSensor::new().unwrap();

    for i in 0..5 {
        let result = sensor.read();
        assert!(result.is_ok(), "Read {} failed", i);

        let data = result.unwrap();
        assert!(!data.is_empty());
    }

    assert_eq!(sensor.reading_count(), 50); // 5 reads × 10 buffer size
}

#[test]
fn test_multiple_reads_touch() {
    let mut sensor = TouchSensor::new().unwrap();

    for i in 0..5 {
        let result = sensor.read();
        assert!(result.is_ok(), "Read {} failed", i);

        let data = result.unwrap();
        assert!(!data.is_empty());
    }

    // Touch count varia dependendo dos dados mock
    assert!(sensor.touch_count() <= 5);
}

#[test]
fn test_sensor_component_traits() {
    let pressure = PressureSensor::new().unwrap();
    assert_eq!(pressure.name(), "PressureSensor");
    assert_eq!(pressure.target_layer(), 4);
    assert!(pressure.is_ready());

    let touch = TouchSensor::new().unwrap();
    assert_eq!(touch.name(), "TouchSensor");
    assert_eq!(touch.target_layer(), 4);
    assert!(touch.is_ready());
}
