//! Testes do módulo sil-gustatory

use super::*;
use sil_core::traits::{Sensor, SilComponent};

// ═══════════════════════════════════════════════════════════════════════════════
// TESTES DE TASTE SENSOR
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_taste_sensor_creation() {
    let sensor = TasteSensor::new();
    assert!(sensor.is_ok());

    let sensor = sensor.unwrap();
    assert_eq!(sensor.name(), "TasteSensor");
    assert_eq!(sensor.layers(), &[3]);
    assert!(sensor.is_ready());
}

#[test]
fn test_taste_sensor_with_config() {
    let config = TasteSensorConfig {
        sensitivity: 0.8,
        sample_rate: 20.0,
        ph_offset: 0.1,
        salinity_multiplier: 1.5,
    };

    let sensor = TasteSensor::with_config(config);
    assert!(sensor.is_ok());

    let sensor = sensor.unwrap();
    assert_eq!(sensor.config().sensitivity, 0.8);
    assert_eq!(sensor.config().sample_rate, 20.0);
}

#[test]
fn test_taste_sensor_invalid_sensitivity() {
    let config = TasteSensorConfig {
        sensitivity: 1.5, // Invalid
        sample_rate: 10.0,
        ph_offset: 0.0,
        salinity_multiplier: 1.0,
    };

    let result = TasteSensor::with_config(config);
    assert!(result.is_err());
}

#[test]
fn test_taste_sensor_invalid_sample_rate() {
    let config = TasteSensorConfig {
        sensitivity: 0.5,
        sample_rate: -10.0, // Invalid
        ph_offset: 0.0,
        salinity_multiplier: 1.0,
    };

    let result = TasteSensor::with_config(config);
    assert!(result.is_err());
}

#[test]
fn test_taste_sensor_read() {
    let mut sensor = TasteSensor::new().unwrap();
    let result = sensor.read();

    assert!(result.is_ok());
    let taste_data = result.unwrap();

    // Verifica que pH está no range válido
    assert!(taste_data.ph.value >= 0.0 && taste_data.ph.value <= 14.0);

    // Verifica que perfil tem valores válidos
    assert!(taste_data.profile.sweet >= 0.0 && taste_data.profile.sweet <= 1.0);
    assert!(taste_data.profile.sour >= 0.0 && taste_data.profile.sour <= 1.0);
    assert!(taste_data.profile.salty >= 0.0 && taste_data.profile.salty <= 1.0);
    assert!(taste_data.profile.bitter >= 0.0 && taste_data.profile.bitter <= 1.0);
    assert!(taste_data.profile.umami >= 0.0 && taste_data.profile.umami <= 1.0);

    // Verifica que salinidade é não-negativa
    assert!(taste_data.salinity.ppm >= 0.0);
}

#[test]
fn test_taste_sensor_to_byte_sil() {
    let sensor = TasteSensor::new().unwrap();

    // pH neutro (7.0) → rho ≈ 0
    let neutral_data = TasteData {
        ph: PhLevel::clamped(7.0),
        profile: TasteProfile::single(TasteType::Sweet, 0.5),
        salinity: Salinity::new(0.0),
        conductivity: None,
        tds: None,
    };
    let byte = sensor.to_byte_sil(&neutral_data);
    assert!(byte.rho >= -1 && byte.rho <= 1);

    // pH ácido (0.0) → rho = -8
    let acidic_data = TasteData {
        ph: PhLevel::clamped(0.0),
        profile: TasteProfile::single(TasteType::Sour, 1.0),
        salinity: Salinity::new(0.0),
        conductivity: None,
        tds: None,
    };
    let byte = sensor.to_byte_sil(&acidic_data);
    assert_eq!(byte.rho, -8);

    // pH básico (14.0) → rho = +7
    let basic_data = TasteData {
        ph: PhLevel::clamped(14.0),
        profile: TasteProfile::single(TasteType::Bitter, 1.0),
        salinity: Salinity::new(0.0),
        conductivity: None,
        tds: None,
    };
    let byte = sensor.to_byte_sil(&basic_data);
    assert_eq!(byte.rho, 7);
}

#[test]
fn test_taste_sensor_sense() {
    let mut sensor = TasteSensor::new().unwrap();
    let result = sensor.sense();

    assert!(result.is_ok());
    let update = result.unwrap();

    // Deve ser para L3
    assert_eq!(update.layer, 3);
    assert!(update.byte.rho >= -8 && update.byte.rho <= 7);
    // theta is u8, so it's always <= 255
    assert_eq!(update.confidence, 1.0);
}

#[test]
fn test_taste_sensor_sample_count() {
    let mut sensor = TasteSensor::new().unwrap();
    assert_eq!(sensor.sample_count(), 0);

    sensor.read().unwrap();
    assert_eq!(sensor.sample_count(), 1);

    sensor.read().unwrap();
    sensor.read().unwrap();
    assert_eq!(sensor.sample_count(), 3);
}

#[test]
fn test_taste_sensor_calibrate() {
    let mut sensor = TasteSensor::new().unwrap();
    sensor.read().unwrap();

    let result = sensor.calibrate();
    assert!(result.is_ok());
    assert!(sensor.is_ready());
    assert_eq!(sensor.sample_count(), 0);
    assert_eq!(sensor.last_reading().ph.value, 7.0);
}

#[test]
fn test_taste_sensor_target_layer() {
    let sensor = TasteSensor::new().unwrap();
    assert_eq!(sensor.target_layer(), 3);
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTES DE PH LEVEL
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_ph_level_creation() {
    let ph = PhLevel::new(7.0);
    assert!(ph.is_ok());
    assert_eq!(ph.unwrap().value, 7.0);
}

#[test]
fn test_ph_level_validation() {
    let invalid_low = PhLevel::new(-1.0);
    assert!(invalid_low.is_err());

    let invalid_high = PhLevel::new(15.0);
    assert!(invalid_high.is_err());
}

#[test]
fn test_ph_level_clamped() {
    let ph = PhLevel::clamped(-5.0);
    assert_eq!(ph.value, 0.0);

    let ph = PhLevel::clamped(20.0);
    assert_eq!(ph.value, 14.0);

    let ph = PhLevel::clamped(7.0);
    assert_eq!(ph.value, 7.0);
}

#[test]
fn test_ph_level_to_rho() {
    let ph_acidic = PhLevel::clamped(0.0);
    assert_eq!(ph_acidic.to_rho(), -8);

    let ph_neutral = PhLevel::clamped(7.0);
    assert_eq!(ph_neutral.to_rho(), 0);

    let ph_basic = PhLevel::clamped(14.0);
    assert_eq!(ph_basic.to_rho(), 7);
}

#[test]
fn test_ph_level_properties() {
    let acidic = PhLevel::clamped(3.0);
    assert!(acidic.is_acidic());
    assert!(!acidic.is_neutral());
    assert!(!acidic.is_basic());

    let neutral = PhLevel::clamped(7.0);
    assert!(!neutral.is_acidic());
    assert!(neutral.is_neutral());
    assert!(!neutral.is_basic());

    let basic = PhLevel::clamped(10.0);
    assert!(!basic.is_acidic());
    assert!(!basic.is_neutral());
    assert!(basic.is_basic());
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTES DE TASTE TYPE
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_taste_type_to_theta() {
    assert_eq!(TasteType::Sweet.to_theta(), 0);
    assert_eq!(TasteType::Sour.to_theta(), 51);
    assert_eq!(TasteType::Salty.to_theta(), 102);
    assert_eq!(TasteType::Bitter.to_theta(), 153);
    assert_eq!(TasteType::Umami.to_theta(), 204);
}

#[test]
fn test_taste_type_from_theta() {
    assert_eq!(TasteType::from_theta(0), TasteType::Sweet);
    assert_eq!(TasteType::from_theta(25), TasteType::Sweet);
    assert_eq!(TasteType::from_theta(51), TasteType::Sour);
    assert_eq!(TasteType::from_theta(102), TasteType::Salty);
    assert_eq!(TasteType::from_theta(153), TasteType::Bitter);
    assert_eq!(TasteType::from_theta(204), TasteType::Umami);
    assert_eq!(TasteType::from_theta(255), TasteType::Umami);
}

#[test]
fn test_taste_type_all() {
    let all = TasteType::all();
    assert_eq!(all.len(), 5);
    assert_eq!(all[0], TasteType::Sweet);
    assert_eq!(all[1], TasteType::Sour);
    assert_eq!(all[2], TasteType::Salty);
    assert_eq!(all[3], TasteType::Bitter);
    assert_eq!(all[4], TasteType::Umami);
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTES DE TASTE PROFILE
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_taste_profile_neutral() {
    let profile = TasteProfile::neutral();
    assert_eq!(profile.sweet, 0.0);
    assert_eq!(profile.sour, 0.0);
    assert_eq!(profile.salty, 0.0);
    assert_eq!(profile.bitter, 0.0);
    assert_eq!(profile.umami, 0.0);
}

#[test]
fn test_taste_profile_single() {
    let profile = TasteProfile::single(TasteType::Sweet, 0.8);
    assert_eq!(profile.sweet, 0.8);
    assert_eq!(profile.sour, 0.0);
    assert_eq!(profile.salty, 0.0);
    assert_eq!(profile.bitter, 0.0);
    assert_eq!(profile.umami, 0.0);
}

#[test]
fn test_taste_profile_single_clamp() {
    let profile = TasteProfile::single(TasteType::Bitter, 1.5);
    assert_eq!(profile.bitter, 1.0); // Clamped to 1.0
}

#[test]
fn test_taste_profile_dominant() {
    let mut profile = TasteProfile::neutral();
    profile.salty = 0.7;
    profile.bitter = 0.3;
    profile.umami = 0.5;

    let (dominant, intensity) = profile.dominant();
    assert_eq!(dominant, TasteType::Salty);
    assert_eq!(intensity, 0.7);
}

#[test]
fn test_taste_profile_total_intensity() {
    let mut profile = TasteProfile::neutral();
    profile.sweet = 0.2;
    profile.sour = 0.3;
    profile.salty = 0.5;

    let total = profile.total_intensity();
    assert!((total - 1.0).abs() < 0.001);
}

#[test]
fn test_taste_profile_normalize() {
    let mut profile = TasteProfile::neutral();
    profile.sweet = 2.0;
    profile.sour = 2.0;
    profile.salty = 6.0;

    profile.normalize();

    let total = profile.total_intensity();
    assert!((total - 1.0).abs() < 0.001);
    assert!((profile.sweet - 0.2).abs() < 0.001);
    assert!((profile.sour - 0.2).abs() < 0.001);
    assert!((profile.salty - 0.6).abs() < 0.001);
}

#[test]
fn test_taste_profile_intensity_of() {
    let mut profile = TasteProfile::neutral();
    profile.umami = 0.9;

    assert_eq!(profile.intensity_of(TasteType::Sweet), 0.0);
    assert_eq!(profile.intensity_of(TasteType::Umami), 0.9);
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTES DE SALINITY
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_salinity_creation() {
    let salinity = Salinity::new(1000.0);
    assert_eq!(salinity.ppm, 1000.0);
}

#[test]
fn test_salinity_negative_clamp() {
    let salinity = Salinity::new(-500.0);
    assert_eq!(salinity.ppm, 0.0);
}

#[test]
fn test_salinity_water_types() {
    let fresh = Salinity::new(300.0);
    assert!(fresh.is_fresh_water());
    assert!(!fresh.is_brackish_water());
    assert!(!fresh.is_salt_water());

    let brackish = Salinity::new(10000.0);
    assert!(!brackish.is_fresh_water());
    assert!(brackish.is_brackish_water());
    assert!(!brackish.is_salt_water());

    let salt = Salinity::new(35000.0);
    assert!(!salt.is_fresh_water());
    assert!(!salt.is_brackish_water());
    assert!(salt.is_salt_water());
}

#[test]
fn test_salinity_normalized() {
    let fresh = Salinity::new(0.0);
    assert_eq!(fresh.normalized(), 0.0);

    let ocean = Salinity::new(35000.0);
    assert_eq!(ocean.normalized(), 1.0);

    let half = Salinity::new(17500.0);
    assert!((half.normalized() - 0.5).abs() < 0.001);
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTES DE TASTE DATA
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_taste_data_neutral() {
    let data = TasteData::neutral();
    assert_eq!(data.ph.value, 7.0);
    assert_eq!(data.profile.total_intensity(), 0.0);
    assert_eq!(data.salinity.ppm, 0.0);
}

#[test]
fn test_taste_data_new() {
    let ph = PhLevel::clamped(5.0);
    let profile = TasteProfile::single(TasteType::Sour, 0.6);
    let data = TasteData::new(ph, profile);

    assert_eq!(data.ph.value, 5.0);
    assert_eq!(data.profile.sour, 0.6);
}

#[test]
fn test_taste_data_full() {
    let ph = PhLevel::clamped(8.0);
    let profile = TasteProfile::single(TasteType::Bitter, 0.7);
    let salinity = Salinity::new(500.0);
    let data = TasteData::full(ph, profile, salinity, Some(1000.0), Some(500.0));

    assert_eq!(data.ph.value, 8.0);
    assert_eq!(data.profile.bitter, 0.7);
    assert_eq!(data.salinity.ppm, 500.0);
    assert_eq!(data.conductivity, Some(1000.0));
    assert_eq!(data.tds, Some(500.0));
}

#[test]
fn test_taste_data_dominant_taste() {
    let mut profile = TasteProfile::neutral();
    profile.sweet = 0.3;
    profile.umami = 0.8;

    let data = TasteData::new(PhLevel::default(), profile);
    assert_eq!(data.dominant_taste(), TasteType::Umami);
    assert_eq!(data.dominant_intensity(), 0.8);
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTES DE ERROR
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_gustatory_error_conversion() {
    use sil_core::traits::SensorError;

    let err = GustatoryError::NotReady;
    let sensor_err: SensorError = err.into();
    assert!(matches!(sensor_err, SensorError::NotInitialized));

    let err = GustatoryError::Timeout(5000);
    let sensor_err: SensorError = err.into();
    assert!(matches!(sensor_err, SensorError::Timeout(5000)));
}
