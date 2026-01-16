//! Fusão sensorial implementando o trait Processor (L7)
//!
//! O SensorFusion combina informações das camadas de percepção (L0-L4)
//! com dados ambientais (L7) para criar um contexto ambiental enriquecido.

use serde::{Deserialize, Serialize};
use sil_core::prelude::*;
use sil_core::traits::{SilComponent, Processor, ProcessorError};
use crate::error::{EnvironmentError, EnvironmentResult};
use crate::types::EnvironmentData;

/// Configuração do processador de fusão sensorial
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusionConfig {
    /// Pesos para cada camada sensorial (L0-L4)
    pub layer_weights: [f32; 5],
    /// Peso da camada ambiental (L7)
    pub environment_weight: f32,
    /// Limiar de confiança mínima
    pub confidence_threshold: f32,
    /// Aplicar normalização
    pub normalize: bool,
}

impl Default for FusionConfig {
    fn default() -> Self {
        Self {
            // Pesos iguais para todas as camadas de percepção
            layer_weights: [1.0, 1.0, 1.0, 1.0, 1.0],
            environment_weight: 1.5, // Ambiente tem peso maior
            confidence_threshold: 0.5,
            normalize: true,
        }
    }
}

/// Processador de fusão sensorial (L7)
///
/// O SensorFusion implementa o trait Processor e combina dados de
/// múltiplas camadas sensoriais (L0-L4) com informações ambientais (L7)
/// para produzir um estado contextualizado e enriquecido.
///
/// # Algoritmo
///
/// 1. Extrai dados das camadas L0-L4 do estado de entrada
/// 2. Calcula scores ponderados para cada camada
/// 3. Combina com dados ambientais (L7) usando fusão adaptativa
/// 4. Gera novo estado com L7 atualizado
#[derive(Debug, Clone)]
pub struct SensorFusion {
    config: FusionConfig,
    ready: bool,
    execution_count: u64,
    // Cache de dados ambientais
    cached_environment: Option<EnvironmentData>,
    // Histórico de fusões (para debug)
    fusion_history: Vec<FusionResult>,
}

/// Resultado de uma operação de fusão
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusionResult {
    /// Score de confiança da fusão [0.0, 1.0]
    pub confidence: f32,
    /// Número de camadas utilizadas
    pub layers_used: usize,
    /// Score de contexto resultante [-8, 7]
    pub context_score: i8,
    /// Timestamp da fusão
    pub timestamp: u64,
}

impl SensorFusion {
    /// Cria novo processador de fusão sensorial
    pub fn new() -> EnvironmentResult<Self> {
        Self::with_config(FusionConfig::default())
    }

    /// Cria com configuração específica
    pub fn with_config(config: FusionConfig) -> EnvironmentResult<Self> {
        // Validar configuração
        if config.layer_weights.iter().any(|&w| w < 0.0) {
            return Err(EnvironmentError::InvalidConfig(
                "Layer weights must be non-negative".into(),
            ));
        }

        if config.environment_weight < 0.0 {
            return Err(EnvironmentError::InvalidConfig(
                "Environment weight must be non-negative".into(),
            ));
        }

        if config.confidence_threshold < 0.0 || config.confidence_threshold > 1.0 {
            return Err(EnvironmentError::InvalidConfig(
                "Confidence threshold must be in [0, 1]".into(),
            ));
        }

        Ok(Self {
            config,
            ready: true,
            execution_count: 0,
            cached_environment: None,
            fusion_history: Vec::new(),
        })
    }

    /// Retorna contagem de execuções
    pub fn execution_count(&self) -> u64 {
        self.execution_count
    }

    /// Define dados ambientais para fusão
    pub fn set_environment(&mut self, env: EnvironmentData) {
        self.cached_environment = Some(env);
    }

    /// Retorna histórico de fusões
    pub fn fusion_history(&self) -> &[FusionResult] {
        &self.fusion_history
    }

    /// Limpa histórico
    pub fn clear_history(&mut self) {
        self.fusion_history.clear();
    }

    /// Extrai dados ambientais do estado L7
    fn extract_environment_from_state(&self, state: &SilState) -> EnvironmentData {
        let l7 = state.layers[7];

        // Decodificar ByteSil de volta para EnvironmentData
        // ρ [-8, 7] -> comfort_score [0, 1]
        let comfort = ((l7.rho as f32 + 8.0) / 15.0).clamp(0.0, 1.0);

        // θ [0, 255] -> AQI [0, 500]
        let aqi = (l7.theta as f32 / 255.0) * 500.0;

        // Estimar valores baseados em comfort e AQI
        // (valores aproximados - em produção viriam de sensores reais)
        let temperature = 22.0 + (comfort - 0.5) * 20.0; // 12-32°C
        let humidity = 50.0 + (1.0 - comfort) * 30.0;    // 50-80%
        let pressure = 1013.25; // Padrão

        EnvironmentData {
            temperature,
            humidity,
            pressure,
            air_quality: aqi,
            co2_ppm: 400.0 + aqi * 2.0,
            voc_ppb: 50.0 + aqi * 0.5,
            pm25: (aqi / 500.0) * 50.0,
            pm10: (aqi / 500.0) * 100.0,
        }
    }

    /// Calcula score ponderado das camadas sensoriais
    fn calculate_sensor_scores(&self, state: &SilState) -> Vec<(usize, f32)> {
        let mut scores = Vec::new();

        // Processar L0-L4 (camadas de percepção)
        for i in 0..5 {
            let layer = state.layers[i];
            let weight = self.config.layer_weights[i];

            if weight > 0.0 {
                // Normalizar ρ para [0, 1]
                let rho_norm = (layer.rho as f32 + 8.0) / 15.0;
                // Normalizar θ para [0, 1]
                let theta_norm = layer.theta as f32 / 255.0;

                // Score combinado ponderado
                let score = (rho_norm + theta_norm) / 2.0 * weight;
                scores.push((i, score));
            }
        }

        scores
    }

    /// Executa fusão sensorial
    fn fuse(&mut self, state: &SilState) -> EnvironmentResult<SilState> {
        // Obter ou usar dados ambientais cacheados
        let environment = if let Some(ref env) = self.cached_environment {
            env.clone()
        } else {
            self.extract_environment_from_state(state)
        };

        // Calcular scores das camadas sensoriais
        let sensor_scores = self.calculate_sensor_scores(state);

        // Calcular confiança baseada no número de sensores ativos
        let confidence = if sensor_scores.is_empty() {
            0.0
        } else {
            (sensor_scores.len() as f32 / 5.0).min(1.0)
        };

        // Verificar limiar de confiança
        if confidence < self.config.confidence_threshold {
            return Err(EnvironmentError::InsufficientData(format!(
                "Confidence {:.2} below threshold {:.2}",
                confidence, self.config.confidence_threshold
            )));
        }

        // Calcular score médio dos sensores
        let sensor_avg = if sensor_scores.is_empty() {
            0.5
        } else {
            let sum: f32 = sensor_scores.iter().map(|(_, s)| s).sum();
            sum / sensor_scores.len() as f32
        };

        // Combinar com score ambiental
        let env_comfort = environment.comfort_score();
        let combined_score = if self.config.normalize {
            (sensor_avg + env_comfort * self.config.environment_weight)
                / (1.0 + self.config.environment_weight)
        } else {
            sensor_avg * 0.5 + env_comfort * 0.5
        };

        // Mapear para ρ [-8, 7]
        let context_rho = ((combined_score * 15.0) - 8.0) as i8;

        // θ é o AQI normalizado
        let aqi = environment.composite_aqi();
        let context_theta = ((aqi / 500.0) * 255.0) as u8;

        // Criar novo estado com L7 atualizado
        let new_l7 = ByteSil::new(context_rho.clamp(-8, 7), context_theta);
        let mut new_state = state.clone();
        new_state.layers[7] = new_l7;

        // Registrar resultado
        let result = FusionResult {
            confidence,
            layers_used: sensor_scores.len(),
            context_score: context_rho.clamp(-8, 7),
            timestamp: Self::current_timestamp(),
        };
        self.fusion_history.push(result);

        // Limitar histórico a últimas 100 fusões
        if self.fusion_history.len() > 100 {
            self.fusion_history.remove(0);
        }

        Ok(new_state)
    }

    /// Timestamp atual em segundos
    fn current_timestamp() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }
}

impl Default for SensorFusion {
    fn default() -> Self {
        Self::new().expect("Default SensorFusion creation should not fail")
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// IMPLEMENTAÇÃO DOS TRAITS DO CORE
// ═══════════════════════════════════════════════════════════════════════════════

impl SilComponent for SensorFusion {
    fn name(&self) -> &str {
        "SensorFusion"
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

impl Processor for SensorFusion {
    fn execute(&mut self, state: &SilState) -> Result<SilState, ProcessorError> {
        let result = self.fuse(state).map_err(|e| ProcessorError::ExecutionFailed(e.to_string()))?;
        self.execution_count += 1;
        Ok(result)
    }

    fn latency_ms(&self) -> f32 {
        // Fusão é uma operação rápida
        1.0
    }

    fn supports_batch(&self) -> bool {
        true
    }

    fn execute_batch(&mut self, states: &[SilState]) -> Result<Vec<SilState>, ProcessorError> {
        states.iter().map(|s| self.execute(s)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_fusion() {
        let fusion = SensorFusion::new();
        assert!(fusion.is_ok());
    }

    #[test]
    fn test_fusion_ready_by_default() {
        let fusion = SensorFusion::new().unwrap();
        assert!(fusion.is_ready());
    }

    #[test]
    fn test_execute() {
        let mut fusion = SensorFusion::new().unwrap();
        let state = SilState::neutral();
        let result = fusion.execute(&state);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execution_count_increments() {
        let mut fusion = SensorFusion::new().unwrap();
        let state = SilState::neutral();

        assert_eq!(fusion.execution_count(), 0);
        fusion.execute(&state).unwrap();
        assert_eq!(fusion.execution_count(), 1);
        fusion.execute(&state).unwrap();
        assert_eq!(fusion.execution_count(), 2);
    }

    #[test]
    fn test_fusion_with_environment_data() {
        let mut fusion = SensorFusion::new().unwrap();
        let env = EnvironmentData::default_ideal();
        fusion.set_environment(env);

        let state = SilState::neutral();
        let result = fusion.execute(&state);
        assert!(result.is_ok());
    }

    #[test]
    fn test_fusion_result_recorded() {
        let mut fusion = SensorFusion::new().unwrap();
        let state = SilState::neutral();

        fusion.execute(&state).unwrap();
        assert_eq!(fusion.fusion_history().len(), 1);
    }

    #[test]
    fn test_fusion_history_limit() {
        let mut fusion = SensorFusion::new().unwrap();
        let state = SilState::neutral();

        // Executar 150 vezes
        for _ in 0..150 {
            fusion.execute(&state).unwrap();
        }

        // Histórico deve ser limitado a 100
        assert!(fusion.fusion_history().len() <= 100);
    }

    #[test]
    fn test_clear_history() {
        let mut fusion = SensorFusion::new().unwrap();
        let state = SilState::neutral();

        fusion.execute(&state).unwrap();
        assert!(!fusion.fusion_history().is_empty());

        fusion.clear_history();
        assert!(fusion.fusion_history().is_empty());
    }

    #[test]
    fn test_fusion_with_active_sensors() {
        let mut fusion = SensorFusion::new().unwrap();
        let mut state = SilState::neutral();

        // Ativar algumas camadas sensoriais
        state.layers[0] = ByteSil::new(5, 100);  // L0 - Fotônico
        state.layers[1] = ByteSil::new(3, 150);  // L1 - Acústico
        state.layers[4] = ByteSil::new(-2, 50);  // L4 - Dérmico

        let result = fusion.execute(&state).unwrap();
        assert_eq!(result.layers[7].rho >= -8 && result.layers[7].rho <= 7, true);
    }

    #[test]
    fn test_config_with_custom_weights() {
        let config = FusionConfig {
            layer_weights: [2.0, 1.0, 1.0, 0.5, 1.5],
            environment_weight: 2.0,
            ..Default::default()
        };
        let fusion = SensorFusion::with_config(config);
        assert!(fusion.is_ok());
    }

    #[test]
    fn test_config_invalid_weights() {
        let config = FusionConfig {
            layer_weights: [-1.0, 1.0, 1.0, 1.0, 1.0],
            ..Default::default()
        };
        let fusion = SensorFusion::with_config(config);
        assert!(fusion.is_err());
    }

    #[test]
    fn test_config_invalid_confidence() {
        let config = FusionConfig {
            confidence_threshold: 1.5,
            ..Default::default()
        };
        let fusion = SensorFusion::with_config(config);
        assert!(fusion.is_err());
    }

    #[test]
    fn test_low_confidence_config() {
        // Testar criação com threshold baixo
        let config = FusionConfig {
            confidence_threshold: 0.1,
            ..Default::default()
        };
        let fusion = SensorFusion::with_config(config);
        assert!(fusion.is_ok());
    }

    #[test]
    fn test_supports_batch() {
        let fusion = SensorFusion::new().unwrap();
        assert!(fusion.supports_batch());
    }

    #[test]
    fn test_execute_batch() {
        let mut fusion = SensorFusion::new().unwrap();
        let states = vec![
            SilState::neutral(),
            SilState::neutral(),
            SilState::neutral(),
        ];

        let results = fusion.execute_batch(&states);
        assert!(results.is_ok());
        assert_eq!(results.unwrap().len(), 3);
    }

    #[test]
    fn test_latency() {
        let fusion = SensorFusion::new().unwrap();
        assert!(fusion.latency_ms() > 0.0);
        assert!(fusion.latency_ms() < 10.0);
    }

    #[test]
    fn test_component_trait() {
        let fusion = SensorFusion::new().unwrap();
        assert_eq!(fusion.name(), "SensorFusion");
        assert_eq!(fusion.layers(), &[7]);
    }

    #[test]
    fn test_extract_environment_from_state() {
        let fusion = SensorFusion::new().unwrap();
        let mut state = SilState::neutral();
        state.layers[7] = ByteSil::new(5, 100);

        let env = fusion.extract_environment_from_state(&state);
        assert!(env.temperature > 0.0);
        assert!(env.humidity > 0.0);
    }

    #[test]
    fn test_normalize_enabled() {
        let config = FusionConfig {
            normalize: true,
            ..Default::default()
        };
        let mut fusion = SensorFusion::with_config(config).unwrap();
        let state = SilState::neutral();
        let result = fusion.execute(&state);
        assert!(result.is_ok());
    }

    #[test]
    fn test_normalize_disabled() {
        let config = FusionConfig {
            normalize: false,
            ..Default::default()
        };
        let mut fusion = SensorFusion::with_config(config).unwrap();
        let state = SilState::neutral();
        let result = fusion.execute(&state);
        assert!(result.is_ok());
    }

    #[test]
    fn test_fusion_result_confidence() {
        let mut fusion = SensorFusion::new().unwrap();
        let state = SilState::neutral();
        fusion.execute(&state).unwrap();

        let history = fusion.fusion_history();
        assert!(!history.is_empty());
        assert!(history[0].confidence >= 0.0);
        assert!(history[0].confidence <= 1.0);
    }

    #[test]
    fn test_fusion_result_context_score() {
        let mut fusion = SensorFusion::new().unwrap();
        let state = SilState::neutral();
        fusion.execute(&state).unwrap();

        let history = fusion.fusion_history();
        assert!(history[0].context_score >= -8);
        assert!(history[0].context_score <= 7);
    }
}
