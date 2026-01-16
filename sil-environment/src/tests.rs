//! Testes integrados do módulo sil-environment

use crate::*;
use sil_core::traits::{Sensor, Processor, SilComponent};

// ═══════════════════════════════════════════════════════════════════════════════
// TESTES DE INTEGRAÇÃO - ClimateSensor + SensorFusion
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_full_pipeline_sensor_to_fusion() {
    // Criar sensor climático
    let mut climate = ClimateSensor::new().unwrap();
    climate.calibrate().unwrap();

    // Capturar dados
    let update = climate.sense().unwrap();
    assert_eq!(update.layer, 7);

    // Criar estado e aplicar atualização
    let state = SilState::neutral();
    let state_with_climate = update.apply(&state);

    // Processar com fusão
    let mut fusion = SensorFusion::new().unwrap();
    let enriched = fusion.execute(&state_with_climate).unwrap();

    // Verificar que L7 foi processado
    assert!(enriched.layers[7].rho >= -8 && enriched.layers[7].rho <= 7);
}

#[test]
fn test_sensor_fusion_with_multiple_sensors() {
    // Criar estado com múltiplos sensores ativos
    let mut state = SilState::neutral();
    state.layers[0] = ByteSil::new(5, 100);   // L0 - Fotônico
    state.layers[1] = ByteSil::new(3, 80);    // L1 - Acústico
    state.layers[2] = ByteSil::new(-1, 120);  // L2 - Olfativo
    state.layers[3] = ByteSil::new(2, 60);    // L3 - Gustativo
    state.layers[4] = ByteSil::new(4, 90);    // L4 - Dérmico

    // Processar fusão
    let mut fusion = SensorFusion::new().unwrap();
    let result = fusion.execute(&state).unwrap();

    // Verificar que fusão utilizou múltiplos sensores
    let history = fusion.fusion_history();
    assert_eq!(history.len(), 1);
    assert!(history[0].layers_used > 0);
}

#[test]
fn test_climate_sensor_multiple_readings() {
    let mut climate = ClimateSensor::new().unwrap();
    climate.calibrate().unwrap();

    let mut readings = Vec::new();
    for _ in 0..10 {
        let update = climate.sense().unwrap();
        readings.push(update);
    }

    assert_eq!(readings.len(), 10);
    // Verificar que todas as leituras estão em range válido
    for reading in readings {
        assert!(reading.byte.rho >= -8 && reading.byte.rho <= 7);
        assert!(reading.byte.theta <= 255);
    }
}

#[test]
fn test_fusion_with_environment_data() {
    let mut fusion = SensorFusion::new().unwrap();

    // Criar dados ambientais extremos
    let hot_polluted = EnvironmentData {
        temperature: 38.0,
        humidity: 85.0,
        pressure: 1005.0,
        air_quality: 180.0,
        co2_ppm: 2500.0,
        voc_ppb: 300.0,
        pm25: 75.0,
        pm10: 150.0,
    };

    fusion.set_environment(hot_polluted);

    let state = SilState::neutral();
    let result = fusion.execute(&state).unwrap();

    // Ambiente ruim deve resultar em score baixo (ρ baixo)
    assert!(result.layers[7].rho < 5);
    // Verificar que L7 foi atualizado
    assert!(result.layers[7].theta <= 255);
}

#[test]
fn test_fusion_batch_processing() {
    let mut fusion = SensorFusion::new().unwrap();

    // Criar múltiplos estados
    let states = vec![
        SilState::neutral(),
        SilState::vacuum(),
        SilState::maximum(),
    ];

    let results = fusion.execute_batch(&states).unwrap();
    assert_eq!(results.len(), 3);

    // Verificar que todos foram processados
    for result in results {
        assert!(result.layers[7].rho >= -8 && result.layers[7].rho <= 7);
    }
}

#[test]
fn test_climate_configuration_changes() {
    let config = ClimateConfig {
        sample_rate: 10.0,
        enable_smoothing: false,
        ..Default::default()
    };

    let mut climate = ClimateSensor::with_config(config).unwrap();
    assert_eq!(climate.sample_rate(), 10.0);

    climate.calibrate().unwrap();
    let update = climate.sense().unwrap();
    assert!(update.layer == 7);
}

#[test]
fn test_fusion_confidence_threshold() {
    // Testar que configuração de threshold funciona
    let config = FusionConfig {
        confidence_threshold: 0.5,
        ..Default::default()
    };

    let fusion = SensorFusion::with_config(config);
    assert!(fusion.is_ok());
}

#[test]
fn test_fusion_custom_weights() {
    let config = FusionConfig {
        layer_weights: [2.0, 2.0, 1.0, 1.0, 1.0],
        environment_weight: 3.0,
        ..Default::default()
    };

    let mut fusion = SensorFusion::with_config(config).unwrap();
    let mut state = SilState::neutral();
    state.layers[0] = ByteSil::new(7, 200);
    state.layers[1] = ByteSil::new(6, 180);

    let result = fusion.execute(&state).unwrap();
    assert!(result.layers[7].rho > 0);
}

#[test]
fn test_environment_data_alerts() {
    let mut data = EnvironmentData::default();
    assert!(!data.has_alerts());

    // Adicionar condições de alerta
    data.temperature = 40.0;
    data.co2_ppm = 2500.0;

    assert!(data.has_alerts());
    let alerts = data.get_alerts();
    assert!(alerts.len() >= 2);
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTES DE EDGE CASES
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_climate_sensor_stress() {
    let mut climate = ClimateSensor::new().unwrap();
    climate.calibrate().unwrap();

    // Realizar muitas leituras
    for _ in 0..1000 {
        let update = climate.sense().unwrap();
        assert!(update.byte.rho >= -8 && update.byte.rho <= 7);
    }

    assert_eq!(climate.sample_count(), 1000);
}

#[test]
fn test_fusion_empty_history() {
    let fusion = SensorFusion::new().unwrap();
    assert!(fusion.fusion_history().is_empty());
}

#[test]
fn test_fusion_history_overflow() {
    let mut fusion = SensorFusion::new().unwrap();
    let state = SilState::neutral();

    // Executar mais de 100 vezes para testar limite de histórico
    for _ in 0..150 {
        fusion.execute(&state).unwrap();
    }

    // Histórico deve estar limitado a 100
    assert!(fusion.fusion_history().len() <= 100);
    assert_eq!(fusion.execution_count(), 150);
}

#[test]
fn test_climate_smoothing_window() {
    let config = ClimateConfig {
        enable_smoothing: true,
        smoothing_window: 3,
        ..Default::default()
    };

    let mut climate = ClimateSensor::with_config(config).unwrap();
    climate.calibrate().unwrap();

    // Fazer leituras suficientes para preencher janela
    for _ in 0..5 {
        climate.sense().unwrap();
    }

    // Verificar que está funcionando
    assert!(climate.sample_count() == 5);
}

#[test]
fn test_environment_data_composite_aqi() {
    let mut data = EnvironmentData::default();

    // Testar diferentes níveis de poluição
    data.co2_ppm = 2000.0;
    data.voc_ppb = 500.0;
    data.pm25 = 75.0;

    let aqi = data.composite_aqi();
    assert!(aqi > 100.0, "High pollution should result in high AQI");
}

#[test]
fn test_climate_without_calibration() {
    let mut climate = ClimateSensor::new().unwrap();
    assert!(!climate.is_ready());

    let result = climate.read();
    assert!(result.is_err());
}

#[test]
fn test_fusion_with_zero_weights() {
    let config = FusionConfig {
        layer_weights: [0.0, 0.0, 0.0, 0.0, 0.0],
        environment_weight: 1.0,
        confidence_threshold: 0.0,
        ..Default::default()
    };

    let mut fusion = SensorFusion::with_config(config).unwrap();
    let state = SilState::neutral();
    let result = fusion.execute(&state);

    // Deve funcionar mesmo sem sensores, usando apenas ambiente
    assert!(result.is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTES DE CONFORMIDADE COM TRAITS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_climate_sensor_trait_implementations() {
    let climate = ClimateSensor::new().unwrap();

    // SilComponent
    assert_eq!(climate.name(), "ClimateSensor");
    assert_eq!(climate.layers(), &[7]);
    assert!(!climate.version().is_empty());

    // Sensor
    assert_eq!(climate.target_layer(), 7);
    assert!(climate.sample_rate() >= 0.0);
}

#[test]
fn test_sensor_fusion_trait_implementations() {
    let fusion = SensorFusion::new().unwrap();

    // SilComponent
    assert_eq!(fusion.name(), "SensorFusion");
    assert_eq!(fusion.layers(), &[7]);
    assert!(!fusion.version().is_empty());
    assert!(fusion.is_ready());

    // Processor
    assert!(fusion.latency_ms() > 0.0);
    assert!(fusion.supports_batch());
}

#[test]
fn test_sensor_sense_method() {
    let mut climate = ClimateSensor::new().unwrap();
    climate.calibrate().unwrap();

    let update = climate.sense().unwrap();
    assert_eq!(update.layer, 7);
    assert!(update.confidence >= 0.0 && update.confidence <= 1.0);
    assert!(update.timestamp > 0);
}

#[test]
fn test_processor_execute_method() {
    let mut fusion = SensorFusion::new().unwrap();
    let state = SilState::neutral();

    let result = fusion.execute(&state);
    assert!(result.is_ok());

    let new_state = result.unwrap();
    // L7 deve ter sido modificado
    assert!(new_state.layers[7] != state.layers[7] || state.layers[7].rho == 0);
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTES DE SERIALIZAÇÃO
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_environment_data_serialization() {
    let data = EnvironmentData::default();
    let json = serde_json::to_string(&data).unwrap();
    let deserialized: EnvironmentData = serde_json::from_str(&json).unwrap();
    assert_eq!(data, deserialized);
}

#[test]
fn test_climate_config_serialization() {
    let config = ClimateConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: ClimateConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(config.sample_rate, deserialized.sample_rate);
}

#[test]
fn test_fusion_config_serialization() {
    let config = FusionConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: FusionConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(config.environment_weight, deserialized.environment_weight);
}

#[test]
fn test_fusion_result_serialization() {
    let result = FusionResult {
        confidence: 0.95,
        layers_used: 5,
        context_score: 3,
        timestamp: 1234567890,
    };
    let json = serde_json::to_string(&result).unwrap();
    let deserialized: FusionResult = serde_json::from_str(&json).unwrap();
    assert_eq!(result.confidence, deserialized.confidence);
    assert_eq!(result.layers_used, deserialized.layers_used);
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTES DE CONVERSÃO E MAPEAMENTO
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_environment_to_bytesil_ideal() {
    let climate = ClimateSensor::new().unwrap();
    let data = EnvironmentData::default_ideal();
    let byte = climate.to_byte_sil(&data);

    // Ambiente ideal deve ter valores favoráveis
    assert!(byte.rho > 0);
    assert!(byte.theta < 128);
}

#[test]
fn test_environment_to_bytesil_extreme_cold() {
    let climate = ClimateSensor::new().unwrap();
    let mut data = EnvironmentData::default();
    data.temperature = -10.0;
    data.humidity = 90.0;

    let byte = climate.to_byte_sil(&data);
    assert!(byte.rho < 0);
}

#[test]
fn test_environment_to_bytesil_extreme_hot() {
    let climate = ClimateSensor::new().unwrap();
    let mut data = EnvironmentData::default();
    data.temperature = 45.0;
    data.air_quality = 250.0;

    let byte = climate.to_byte_sil(&data);
    assert!(byte.rho <= 2); // Muito quente = score muito baixo
    assert!(byte.theta <= 255); // Verificar range válido
}

#[test]
fn test_pm25_to_aqi_ranges() {
    // Good (0-50 AQI)
    let good_pm25 = 10.0;
    let aqi = EnvironmentData::pm25_to_aqi(good_pm25);
    assert!(aqi < 50.0);

    // Moderate (51-100 AQI)
    let moderate_pm25 = 25.0;
    let aqi = EnvironmentData::pm25_to_aqi(moderate_pm25);
    assert!(aqi > 50.0 && aqi < 100.0);

    // Unhealthy (151-200 AQI)
    let unhealthy_pm25 = 60.0;
    let aqi = EnvironmentData::pm25_to_aqi(unhealthy_pm25);
    assert!(aqi > 150.0);
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTES DE PERFORMANCE E LATÊNCIA
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_fusion_latency() {
    let fusion = SensorFusion::new().unwrap();
    let latency = fusion.latency_ms();
    assert!(latency > 0.0);
    assert!(latency < 100.0, "Fusion should be fast");
}

#[test]
fn test_climate_rapid_readings() {
    let mut climate = ClimateSensor::new().unwrap();
    climate.calibrate().unwrap();

    use std::time::Instant;
    let start = Instant::now();

    for _ in 0..100 {
        climate.sense().unwrap();
    }

    let duration = start.elapsed();
    // 100 leituras devem ser rápidas (< 1 segundo)
    assert!(duration.as_secs() < 1, "Readings should be fast");
}

#[test]
fn test_fusion_rapid_processing() {
    let mut fusion = SensorFusion::new().unwrap();
    let state = SilState::neutral();

    use std::time::Instant;
    let start = Instant::now();

    for _ in 0..100 {
        fusion.execute(&state).unwrap();
    }

    let duration = start.elapsed();
    // 100 processamentos devem ser rápidos (< 1 segundo)
    assert!(duration.as_secs() < 1, "Processing should be fast");
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTES DE VALIDAÇÃO E LIMITES
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_environment_limits_validation() {
    let limits = EnvironmentLimits::default();
    assert!(limits.temp_min < limits.temp_max);
    assert!(limits.humidity_max > 0.0);
    assert!(limits.aqi_max > 0.0);
}

#[test]
fn test_climate_invalid_config() {
    let config = ClimateConfig {
        sample_rate: -1.0,
        ..Default::default()
    };
    let result = ClimateSensor::with_config(config);
    assert!(result.is_err());
}

#[test]
fn test_fusion_invalid_config() {
    let config = FusionConfig {
        layer_weights: [-1.0, 1.0, 1.0, 1.0, 1.0],
        ..Default::default()
    };
    let result = SensorFusion::with_config(config);
    assert!(result.is_err());
}

#[test]
fn test_bytesil_range_validation() {
    let mut climate = ClimateSensor::new().unwrap();
    climate.calibrate().unwrap();

    // Testar 100 leituras para garantir que sempre estão no range
    for _ in 0..100 {
        let update = climate.sense().unwrap();
        assert!(update.byte.rho >= -8 && update.byte.rho <= 7,
            "rho={} out of range", update.byte.rho);
        assert!(update.byte.theta <= 255,
            "theta={} out of range", update.byte.theta);
    }
}

#[test]
fn test_comfort_score_range() {
    let data = EnvironmentData::default();
    let score = data.comfort_score();
    assert!(score >= 0.0 && score <= 1.0,
        "Comfort score {} out of [0, 1] range", score);
}
