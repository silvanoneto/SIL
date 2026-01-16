//! # Finality — Semântica de Colapso L(F)
//!
//! Implementa os tipos de colapso e threshold semântico da camada L(F).
//!
//! ## Significado de L(F)
//!
//! A camada L(F) representa a "medição" ou "decisão" que finaliza
//! um ciclo de processamento. O valor de rho indica proximidade
//! ao colapso, e theta indica o tipo de colapso.

use serde::{Deserialize, Serialize};
use std::fmt;
use sil_core::{ByteSil, SilState};

/// Tipo de colapso (interpretação theta de LF)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum CollapseType {
    /// Nenhum colapso (continua)
    None = 0,
    /// Colapso suave (gradual)
    Soft = 2,
    /// Colapso por medição
    #[default]
    Measurement = 4,
    /// Colapso por decoerência
    Decoherence = 6,
    /// Colapso por threshold
    Threshold = 8,
    /// Colapso por timeout
    Timeout = 10,
    /// Colapso forçado (hard reset)
    Forced = 12,
    /// Colapso de emergência (panic)
    Emergency = 14,
}

impl CollapseType {
    /// Cria CollapseType a partir de theta (0-15)
    pub fn from_theta(theta: u8) -> Self {
        match theta & 0b1110 {
            0 => Self::None,
            2 => Self::Soft,
            4 => Self::Measurement,
            6 => Self::Decoherence,
            8 => Self::Threshold,
            10 => Self::Timeout,
            12 => Self::Forced,
            14 => Self::Emergency,
            _ => Self::Measurement,
        }
    }

    /// Converte para theta
    pub fn to_theta(self) -> u8 {
        self as u8
    }

    /// Verifica se é colapso irrecuperável
    pub fn is_hard(&self) -> bool {
        matches!(self, Self::Forced | Self::Emergency)
    }

    /// Verifica se preserva estado
    pub fn preserves_state(&self) -> bool {
        matches!(self, Self::None | Self::Soft | Self::Measurement)
    }

    /// Prioridade do colapso (maior = mais urgente)
    pub fn priority(&self) -> u8 {
        match self {
            Self::None => 0,
            Self::Soft => 1,
            Self::Measurement => 2,
            Self::Decoherence => 3,
            Self::Threshold => 4,
            Self::Timeout => 5,
            Self::Forced => 6,
            Self::Emergency => 7,
        }
    }

    /// Nome descritivo
    pub fn name(&self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Soft => "Soft",
            Self::Measurement => "Measurement",
            Self::Decoherence => "Decoherence",
            Self::Threshold => "Threshold",
            Self::Timeout => "Timeout",
            Self::Forced => "Forced",
            Self::Emergency => "Emergency",
        }
    }
}

impl fmt::Display for CollapseType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Índice da camada de colapso
pub const COLLAPSE_LAYER: usize = 0xF;

/// Threshold padrão para colapso
pub const DEFAULT_COLLAPSE_THRESHOLD: i8 = 6;

/// Configuração de threshold
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CollapseConfig {
    /// Threshold de rho para disparar colapso
    pub rho_threshold: i8,
    /// Tipo de colapso preferido
    pub collapse_type: CollapseType,
    /// Permitir colapso de emergência?
    pub allow_emergency: bool,
    /// Timeout em ciclos
    pub timeout_cycles: u64,
    /// Contador de ciclos atual
    current_cycles: u64,
}

impl Default for CollapseConfig {
    fn default() -> Self {
        Self {
            rho_threshold: DEFAULT_COLLAPSE_THRESHOLD,
            collapse_type: CollapseType::Measurement,
            allow_emergency: true,
            timeout_cycles: 10000,
            current_cycles: 0,
        }
    }
}

impl CollapseConfig {
    /// Cria nova configuração
    pub fn new() -> Self {
        Self::default()
    }

    /// Define threshold
    pub fn with_threshold(mut self, threshold: i8) -> Self {
        self.rho_threshold = threshold;
        self
    }

    /// Define tipo de colapso
    pub fn with_collapse_type(mut self, collapse_type: CollapseType) -> Self {
        self.collapse_type = collapse_type;
        self
    }

    /// Define timeout
    pub fn with_timeout(mut self, cycles: u64) -> Self {
        self.timeout_cycles = cycles;
        self
    }

    /// Configuração estrita (colapso rápido)
    pub fn strict() -> Self {
        Self {
            rho_threshold: 3,
            collapse_type: CollapseType::Threshold,
            allow_emergency: true,
            timeout_cycles: 1000,
            current_cycles: 0,
        }
    }

    /// Configuração permissiva (colapso lento)
    pub fn permissive() -> Self {
        Self {
            rho_threshold: 7,
            collapse_type: CollapseType::Soft,
            allow_emergency: false,
            timeout_cycles: 100000,
            current_cycles: 0,
        }
    }

    /// Incrementa contador de ciclos
    pub fn tick(&mut self) {
        self.current_cycles = self.current_cycles.saturating_add(1);
    }

    /// Reseta contador
    pub fn reset_cycles(&mut self) {
        self.current_cycles = 0;
    }

    /// Verifica se atingiu timeout
    pub fn is_timed_out(&self) -> bool {
        self.current_cycles >= self.timeout_cycles
    }
}

/// Resultado da verificação de colapso
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CollapseDecision {
    /// Continuar execução
    Continue,
    /// Colapso iminente (preparar)
    Imminent(CollapseType),
    /// Colapsar agora
    Collapse(CollapseType),
}

/// Verifica condição de colapso em um estado
pub fn check_collapse(state: &SilState, config: &CollapseConfig) -> CollapseDecision {
    let collapse_byte = state.layer(COLLAPSE_LAYER);
    let rho = collapse_byte.rho;
    let theta = collapse_byte.theta;

    // Tipo de colapso indicado pelo estado
    let indicated_type = CollapseType::from_theta(theta);

    // Verifica emergência
    if indicated_type == CollapseType::Emergency {
        if config.allow_emergency {
            return CollapseDecision::Collapse(CollapseType::Emergency);
        } else {
            return CollapseDecision::Collapse(CollapseType::Forced);
        }
    }

    // Verifica timeout
    if config.is_timed_out() {
        return CollapseDecision::Collapse(CollapseType::Timeout);
    }

    // Verifica threshold
    if rho >= config.rho_threshold {
        return CollapseDecision::Collapse(indicated_type);
    }

    // Verifica iminência (próximo ao threshold)
    if rho >= config.rho_threshold - 2 {
        return CollapseDecision::Imminent(indicated_type);
    }

    CollapseDecision::Continue
}

/// Calcula "finalidade" de um estado (0.0 = não final, 1.0 = colapso)
pub fn finality(state: &SilState) -> f64 {
    let collapse_byte = state.layer(COLLAPSE_LAYER);

    // Normaliza rho para [0, 1] onde 7 = 1.0
    let rho_norm = (collapse_byte.rho as f64 + 8.0) / 15.0;

    // Penaliza tipos de colapso mais severos
    let type_factor = CollapseType::from_theta(collapse_byte.theta).priority() as f64 / 7.0;

    // Combina (média ponderada)
    0.7 * rho_norm + 0.3 * type_factor
}

/// Prepara estado para colapso (define camada LF)
pub fn prepare_collapse(state: &SilState, collapse_type: CollapseType, urgency: i8) -> SilState {
    let byte = ByteSil::new(urgency, collapse_type.to_theta());
    state.with_layer(COLLAPSE_LAYER, byte)
}

/// Reseta camada de colapso
pub fn reset_collapse(state: &SilState) -> SilState {
    let byte = ByteSil::new(-8, CollapseType::None.to_theta());
    state.with_layer(COLLAPSE_LAYER, byte)
}

/// Estatísticas de colapso
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CollapseStats {
    /// Total de colapsos
    pub total_collapses: u64,
    /// Colapsos por tipo
    pub by_type: [u64; 8],
    /// Média de ciclos até colapso
    pub mean_cycles_to_collapse: f64,
    /// Máximo de ciclos até colapso
    pub max_cycles_to_collapse: u64,
}

impl CollapseStats {
    /// Registra colapso
    pub fn record(&mut self, collapse_type: CollapseType, cycles: u64) {
        self.total_collapses += 1;

        let idx = (collapse_type.to_theta() / 2) as usize;
        if idx < 8 {
            self.by_type[idx] += 1;
        }

        // Atualiza média móvel
        let alpha = 0.1;
        self.mean_cycles_to_collapse = alpha * cycles as f64
            + (1.0 - alpha) * self.mean_cycles_to_collapse;

        if cycles > self.max_cycles_to_collapse {
            self.max_cycles_to_collapse = cycles;
        }
    }

    /// Tipo mais comum
    pub fn most_common_type(&self) -> CollapseType {
        let max_idx = self.by_type
            .iter()
            .enumerate()
            .max_by_key(|(_, count)| *count)
            .map(|(idx, _)| idx)
            .unwrap_or(2);

        CollapseType::from_theta((max_idx * 2) as u8)
    }
}

// =============================================================================
// Testes
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collapse_type_from_theta() {
        assert_eq!(CollapseType::from_theta(0), CollapseType::None);
        assert_eq!(CollapseType::from_theta(4), CollapseType::Measurement);
        assert_eq!(CollapseType::from_theta(14), CollapseType::Emergency);
    }

    #[test]
    fn test_collapse_type_roundtrip() {
        for theta in (0..16).step_by(2) {
            let ct = CollapseType::from_theta(theta);
            assert_eq!(ct.to_theta(), theta);
        }
    }

    #[test]
    fn test_collapse_priority() {
        assert!(CollapseType::Emergency.priority() > CollapseType::Soft.priority());
        assert!(CollapseType::Forced.priority() > CollapseType::Measurement.priority());
    }

    #[test]
    fn test_is_hard() {
        assert!(CollapseType::Forced.is_hard());
        assert!(CollapseType::Emergency.is_hard());
        assert!(!CollapseType::Measurement.is_hard());
        assert!(!CollapseType::Soft.is_hard());
    }

    #[test]
    fn test_collapse_config_default() {
        let config = CollapseConfig::default();
        assert_eq!(config.rho_threshold, DEFAULT_COLLAPSE_THRESHOLD);
        assert!(!config.is_timed_out());
    }

    #[test]
    fn test_collapse_config_timeout() {
        let mut config = CollapseConfig::new().with_timeout(100);

        for _ in 0..100 {
            config.tick();
        }

        assert!(config.is_timed_out());
    }

    #[test]
    fn test_check_collapse_continue() {
        let state = SilState::neutral();
        let config = CollapseConfig::default();

        let decision = check_collapse(&state, &config);
        assert_eq!(decision, CollapseDecision::Continue);
    }

    #[test]
    fn test_check_collapse_threshold() {
        let byte = ByteSil::new(7, CollapseType::Measurement.to_theta());
        let state = SilState::neutral().with_layer(COLLAPSE_LAYER, byte);
        let config = CollapseConfig::default();

        let decision = check_collapse(&state, &config);
        assert!(matches!(decision, CollapseDecision::Collapse(_)));
    }

    #[test]
    fn test_check_collapse_timeout() {
        let state = SilState::neutral();
        let mut config = CollapseConfig::new().with_timeout(10);

        for _ in 0..20 {
            config.tick();
        }

        let decision = check_collapse(&state, &config);
        assert_eq!(decision, CollapseDecision::Collapse(CollapseType::Timeout));
    }

    #[test]
    fn test_finality() {
        let neutral = SilState::neutral();
        assert!(finality(&neutral) < 0.5);

        let high_rho = ByteSil::new(7, CollapseType::Emergency.to_theta());
        let urgent = SilState::neutral().with_layer(COLLAPSE_LAYER, high_rho);
        assert!(finality(&urgent) > 0.8);
    }

    #[test]
    fn test_prepare_collapse() {
        let state = SilState::neutral();
        let prepared = prepare_collapse(&state, CollapseType::Measurement, 5);

        let byte = prepared.layer(COLLAPSE_LAYER);
        assert_eq!(byte.rho, 5);
        assert_eq!(CollapseType::from_theta(byte.theta), CollapseType::Measurement);
    }

    #[test]
    fn test_reset_collapse() {
        let byte = ByteSil::new(7, CollapseType::Emergency.to_theta());
        let state = SilState::neutral().with_layer(COLLAPSE_LAYER, byte);

        let reset = reset_collapse(&state);
        let byte = reset.layer(COLLAPSE_LAYER);

        assert_eq!(byte.rho, -8);
        assert_eq!(CollapseType::from_theta(byte.theta), CollapseType::None);
    }

    #[test]
    fn test_collapse_stats() {
        let mut stats = CollapseStats::default();

        stats.record(CollapseType::Measurement, 100);
        stats.record(CollapseType::Measurement, 200);
        stats.record(CollapseType::Timeout, 150);

        assert_eq!(stats.total_collapses, 3);
        assert_eq!(stats.most_common_type(), CollapseType::Measurement);
    }

    #[test]
    fn test_collapse_imminent() {
        let byte = ByteSil::new(4, CollapseType::Soft.to_theta());
        let state = SilState::neutral().with_layer(COLLAPSE_LAYER, byte);
        let config = CollapseConfig::default(); // threshold = 6

        let decision = check_collapse(&state, &config);
        assert!(matches!(decision, CollapseDecision::Imminent(_)));
    }
}
