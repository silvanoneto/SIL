//! Testes do módulo sil-olfactory

use super::*;
use sil_core::traits::{Sensor, SilComponent};

// ═══════════════════════════════════════════════════════════════════════════════
// TESTES DE GAS SENSOR
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_gas_sensor_creation() {
    let sensor = GasSensor::new();
    assert!(sensor.is_ok());

    let sensor = sensor.unwrap();
    assert_eq!(sensor.name(), "GasSensor");
    assert_eq!(sensor.layers(), &[2]);
    assert!(sensor.is_ready());
}

#[test]
fn test_gas_sensor_with_config() {
    let config = GasSensorConfig {
        sample_rate: 5.0,
        operating_temperature: 30.0,
        warmup_time: 3000,
        target_gases: vec![GasType::CO, GasType::CO2],
        sensitivity: 0.8,
    };

    let sensor = GasSensor::with_config(config).unwrap();
    assert_eq!(sensor.get_sample_rate(), 5.0);
}

#[test]
fn test_gas_sensor_invalid_config() {
    let config = GasSensorConfig {
        sample_rate: -1.0, // Inválido
        operating_temperature: 25.0,
        warmup_time: 5000,
        target_gases: vec![],
        sensitivity: 0.5,
    };

    let result = GasSensor::with_config(config);
    assert!(result.is_err());
}

#[test]
fn test_gas_sensor_invalid_sensitivity() {
    let config = GasSensorConfig {
        sample_rate: 1.0,
        operating_temperature: 25.0,
        warmup_time: 5000,
        target_gases: vec![],
        sensitivity: 1.5, // Inválido
    };

    let result = GasSensor::with_config(config);
    assert!(result.is_err());
}

#[test]
fn test_gas_sensor_read() {
    let mut sensor = GasSensor::new().unwrap();
    let result = sensor.read();

    assert!(result.is_ok());
    let data = result.unwrap();

    // Verifica estrutura básica
    assert!(data.temperature >= -40.0 && data.temperature <= 85.0);
    assert!(data.humidity >= 0.0 && data.humidity <= 100.0);
    assert!(!data.profile.compounds.is_empty());
}

#[test]
fn test_gas_sensor_to_byte_sil() {
    let sensor = GasSensor::new().unwrap();

    // Cria dados mock com ar limpo
    let data = GasData::clean_air(20.0, 50.0);
    let byte = sensor.to_byte_sil(&data);

    // Ar limpo deve ter rho próximo de -8
    assert!(byte.rho <= -6);
}

#[test]
fn test_gas_sensor_to_byte_sil_with_gases() {
    let sensor = GasSensor::new().unwrap();

    // Cria dados com gases perigosos
    let compounds = vec![
        GasConcentration::new(GasType::CO, 100.0), // Muito perigoso
        GasConcentration::new(GasType::NO2, 5.0),
    ];
    let data = GasData::from_compounds(compounds, 25.0, 60.0);
    let byte = sensor.to_byte_sil(&data);

    // Gases perigosos devem ter rho alto
    assert!(byte.rho > 0);
}

#[test]
fn test_gas_sensor_sense() {
    let mut sensor = GasSensor::new().unwrap();
    let result = sensor.sense();

    assert!(result.is_ok());
    let update = result.unwrap();

    // Deve ser para L2
    assert_eq!(update.layer, 2);
    assert!(update.byte.rho >= -8 && update.byte.rho <= 7);
}

#[test]
fn test_gas_sensor_sample_count() {
    let mut sensor = GasSensor::new().unwrap();
    assert_eq!(sensor.sample_count(), 0);

    sensor.read().unwrap();
    assert_eq!(sensor.sample_count(), 1);

    sensor.read().unwrap();
    sensor.read().unwrap();
    assert_eq!(sensor.sample_count(), 3);
}

#[test]
fn test_gas_sensor_calibrate() {
    let mut sensor = GasSensor::new().unwrap();
    sensor.read().unwrap(); // Faz uma leitura

    let result = sensor.calibrate();
    assert!(result.is_ok());
    assert_eq!(sensor.sample_count(), 0); // Reset após calibração
    assert!(sensor.baseline() >= 0.0);
}

#[test]
fn test_gas_sensor_last_reading() {
    let mut sensor = GasSensor::new().unwrap();
    assert!(sensor.last_reading().is_none());

    sensor.read().unwrap();
    assert!(sensor.last_reading().is_some());
}

#[test]
fn test_gas_sensor_add_target_gas() {
    let mut sensor = GasSensor::new().unwrap();
    sensor.add_target_gas(GasType::O3);

    // Deve incluir O3 nas próximas leituras
    let data = sensor.read().unwrap();
    let has_o3 = data.profile.compounds.iter().any(|c| c.gas_type == GasType::O3);
    assert!(has_o3);
}

#[test]
fn test_gas_sensor_set_sensitivity() {
    let mut sensor = GasSensor::new().unwrap();

    let result = sensor.set_sensitivity(0.9);
    assert!(result.is_ok());

    let invalid = sensor.set_sensitivity(1.5);
    assert!(invalid.is_err());
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTES DE TYPES
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_gas_type() {
    assert_eq!(GasType::CO.id(), 0);
    assert_eq!(GasType::CO.name(), "Carbon Monoxide");

    let (min, max) = GasType::CO.typical_range();
    assert!(min < max);

    assert_eq!(GasType::CO.danger_threshold(), 50.0);
}

#[test]
fn test_gas_concentration() {
    let co = GasConcentration::new(GasType::CO, 30.0);
    assert_eq!(co.gas_type, GasType::CO);
    assert_eq!(co.ppm, 30.0);

    let normalized = co.normalized();
    assert!(normalized >= 0.0 && normalized <= 1.0);

    assert!(!co.is_hazardous()); // 30 < 50
    assert!(co.danger_level() < 1.0);
}

#[test]
fn test_gas_concentration_hazardous() {
    let co_danger = GasConcentration::new(GasType::CO, 80.0);
    assert!(co_danger.is_hazardous()); // 80 > 50
    assert!(co_danger.danger_level() > 1.0);
}

#[test]
fn test_gas_concentration_negative() {
    // Deve clampar valores negativos para 0
    let gas = GasConcentration::new(GasType::CO2, -100.0);
    assert_eq!(gas.ppm, 0.0);
}

#[test]
fn test_odor_class() {
    assert_eq!(OdorClass::Pleasant.id(), 0);
    assert_eq!(OdorClass::Neutral.id(), 1);
    assert_eq!(OdorClass::Unpleasant.id(), 2);
    assert_eq!(OdorClass::Hazardous.id(), 3);
}

#[test]
fn test_odor_class_from_gas() {
    let safe = GasConcentration::new(GasType::CO2, 400.0);
    assert_eq!(OdorClass::from_gas(&safe), OdorClass::Neutral);

    let danger = GasConcentration::new(GasType::CO, 75.0);
    assert_eq!(OdorClass::from_gas(&danger), OdorClass::Hazardous);

    // NH3 at 30 PPM is above danger_level() > 0.5, so it's Hazardous (30 > 25 threshold)
    let unpleasant = GasConcentration::new(GasType::NH3, 30.0);
    assert_eq!(OdorClass::from_gas(&unpleasant), OdorClass::Hazardous);
}

#[test]
fn test_odor_profile_clean_air() {
    let profile = OdorProfile::clean_air();
    assert!(profile.compounds.is_empty());
    assert_eq!(profile.classification, OdorClass::Neutral);
    assert_eq!(profile.air_quality_index, 0);
}

#[test]
fn test_odor_profile_from_compounds() {
    let compounds = vec![
        GasConcentration::new(GasType::CO2, 400.0),
        GasConcentration::new(GasType::VOC, 50.0),
    ];

    let profile = OdorProfile::from_compounds(compounds);
    assert_eq!(profile.compounds.len(), 2);
    assert_eq!(profile.classification, OdorClass::Neutral);
    assert!(profile.air_quality_index < 100);
}

#[test]
fn test_odor_profile_hazardous() {
    let compounds = vec![
        GasConcentration::new(GasType::CO, 100.0), // Perigoso
        GasConcentration::new(GasType::NO2, 5.0),  // Perigoso
    ];

    let profile = OdorProfile::from_compounds(compounds);
    assert_eq!(profile.classification, OdorClass::Hazardous);
    assert!(profile.air_quality_index > 50);
}

#[test]
fn test_odor_profile_dominant_compound() {
    let compounds = vec![
        GasConcentration::new(GasType::CO, 10.0),
        GasConcentration::new(GasType::CO2, 2000.0), // Dominante
        GasConcentration::new(GasType::VOC, 50.0),
    ];

    let profile = OdorProfile::from_compounds(compounds);
    let dominant = profile.dominant_compound();

    assert!(dominant.is_some());
    assert_eq!(dominant.unwrap().gas_type, GasType::CO2);
}

#[test]
fn test_odor_profile_add_compound() {
    let mut profile = OdorProfile::clean_air();
    assert_eq!(profile.compounds.len(), 0);

    profile.add_compound(GasConcentration::new(GasType::CO, 20.0));
    assert_eq!(profile.compounds.len(), 1);
}

#[test]
fn test_chemical_signature() {
    let sig = ChemicalSignature::new(0b1111, 0.5);
    assert_eq!(sig.pattern, 0b1111);
    assert_eq!(sig.intensity, 0.5);
}

#[test]
fn test_chemical_signature_from_profile() {
    let compounds = vec![
        GasConcentration::new(GasType::CO, 10.0),   // Bit 0
        GasConcentration::new(GasType::CO2, 400.0), // Bit 1
    ];
    let profile = OdorProfile::from_compounds(compounds);
    let sig = ChemicalSignature::from_profile(&profile);

    // Bits 0 e 1 devem estar setados
    assert!(sig.pattern & 0b11 == 0b11);
    assert!(sig.intensity > 0.0);
}

#[test]
fn test_chemical_signature_similarity() {
    let sig1 = ChemicalSignature::new(0b1111, 0.5);
    let sig2 = ChemicalSignature::new(0b1111, 0.5);
    assert_eq!(sig1.similarity(&sig2), 1.0);

    let sig3 = ChemicalSignature::new(0b1100, 0.3);
    let similarity = sig1.similarity(&sig3);
    assert!(similarity > 0.0 && similarity < 1.0);

    let sig4 = ChemicalSignature::new(0b0000, 0.0);
    assert!(sig1.similarity(&sig4) < 0.5);
}

#[test]
fn test_gas_data_clean_air() {
    let data = GasData::clean_air(20.0, 50.0);
    assert_eq!(data.temperature, 20.0);
    assert_eq!(data.humidity, 50.0);
    assert!(!data.is_hazardous());
    assert_eq!(data.profile.classification, OdorClass::Neutral);
}

#[test]
fn test_gas_data_from_compounds() {
    let compounds = vec![
        GasConcentration::new(GasType::CO2, 400.0),
        GasConcentration::new(GasType::VOC, 50.0),
    ];

    let data = GasData::from_compounds(compounds, 22.0, 55.0);
    assert_eq!(data.profile.compounds.len(), 2);
    assert_eq!(data.temperature, 22.0);
    assert_eq!(data.humidity, 55.0);
}

#[test]
fn test_gas_data_dominant_compound() {
    let compounds = vec![
        GasConcentration::new(GasType::CO, 10.0),
        GasConcentration::new(GasType::CO2, 1000.0), // Dominante
    ];

    let data = GasData::from_compounds(compounds, 25.0, 60.0);
    let dominant = data.dominant_compound();

    assert!(dominant.is_some());
    assert_eq!(dominant.unwrap().gas_type, GasType::CO2);
}

#[test]
fn test_gas_data_composite_concentration() {
    let data = GasData::clean_air(20.0, 50.0);
    let rho = data.composite_concentration();
    assert!(rho >= -8.0 && rho <= 7.0);
    assert!(rho < -5.0); // Ar limpo deve ter rho baixo

    let compounds = vec![GasConcentration::new(GasType::CO, 100.0)];
    let danger_data = GasData::from_compounds(compounds, 25.0, 60.0);
    let danger_rho = danger_data.composite_concentration();
    // CO at 100 PPM is hazardous (> 50 threshold), so rho should be higher than clean air
    assert!(danger_rho > -5.0); // Gases perigosos devem ter rho maior que ar limpo
}

#[test]
fn test_gas_data_dominant_signature() {
    let compounds = vec![GasConcentration::new(GasType::CH4, 100.0)];
    let data = GasData::from_compounds(compounds, 25.0, 60.0);

    let signature = data.dominant_signature();
    assert_eq!(signature, GasType::CH4.id());
}

#[test]
fn test_gas_data_is_hazardous() {
    let safe = GasData::clean_air(20.0, 50.0);
    assert!(!safe.is_hazardous());

    let compounds = vec![GasConcentration::new(GasType::CO, 100.0)];
    let danger = GasData::from_compounds(compounds, 25.0, 60.0);
    assert!(danger.is_hazardous());
}

#[test]
fn test_gas_sensor_air_quality_index() {
    let mut sensor = GasSensor::new().unwrap();
    sensor.read().unwrap();

    let aqi = sensor.air_quality_index();
    assert!(aqi.is_some());
    assert!(aqi.unwrap() <= 500);
}

#[test]
fn test_gas_sensor_has_hazardous_gases() {
    let mut sensor = GasSensor::new().unwrap();
    sensor.read().unwrap();

    // Com mock data, normalmente não deve ter gases perigosos
    // mas verifica que o método funciona
    let _ = sensor.has_hazardous_gases();
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTES DE INTEGRAÇÃO
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_full_sensing_pipeline() {
    let mut sensor = GasSensor::new().unwrap();

    // Calibra
    sensor.calibrate().unwrap();

    // Lê múltiplas amostras
    for _ in 0..5 {
        let update = sensor.sense().unwrap();
        assert_eq!(update.layer, 2);
        assert!(update.byte.rho >= -8 && update.byte.rho <= 7);
    }

    assert_eq!(sensor.sample_count(), 5);
}

#[test]
fn test_sensor_reconfiguration() {
    let mut sensor = GasSensor::new().unwrap();

    let new_config = GasSensorConfig {
        sample_rate: 10.0,
        operating_temperature: 30.0,
        warmup_time: 3000,
        target_gases: vec![GasType::CH4, GasType::H2],
        sensitivity: 0.7,
    };

    let result = sensor.configure(new_config);
    assert!(result.is_ok());

    assert_eq!(sensor.get_sample_rate(), 10.0);
}
