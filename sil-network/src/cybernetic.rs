//! # CyberneticState — Estado da Camada L8
//!
//! Estado completo da camada cibernética, integrando modo de controle,
//! PID e feedback loops.

use serde::{Deserialize, Serialize};
use sil_core::{ByteSil, SilState};

use crate::{ControlMode, PidController, PidConfig};

/// Índice da camada cibernética
pub const CYBERNETIC_LAYER: usize = 0x8;

/// Estatísticas de controle
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ControlStats {
    /// Número de ciclos de controle
    pub cycles: u64,
    /// Erro médio
    pub mean_error: f64,
    /// Erro máximo absoluto
    pub max_error: f64,
    /// Tempo em estado estável (ciclos)
    pub steady_state_cycles: u64,
    /// Overshoots detectados
    pub overshoots: u64,
}

impl ControlStats {
    /// Atualiza estatísticas com novo erro
    pub fn update(&mut self, error: f64, is_steady: bool) {
        self.cycles += 1;

        // Média móvel exponencial
        let alpha = 0.1;
        self.mean_error = alpha * error.abs() + (1.0 - alpha) * self.mean_error;

        // Máximo
        if error.abs() > self.max_error {
            self.max_error = error.abs();
        }

        // Contagem de estado estável
        if is_steady {
            self.steady_state_cycles += 1;
        }
    }

    /// Registra overshoot
    pub fn record_overshoot(&mut self) {
        self.overshoots += 1;
    }

    /// Reseta estatísticas
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// Estado da camada cibernética
#[derive(Debug)]
pub struct CyberneticState {
    /// Modo de controle ativo (theta)
    pub control_mode: ControlMode,

    /// Nível de feedback (derivado de rho, -8 a +7)
    pub feedback_level: i8,

    /// Controlador PID principal (não serializado)
    pid: PidController,

    /// Setpoint atual
    pub setpoint: f64,

    /// Valor do processo
    pub process_variable: f64,

    /// Última saída de controle
    pub control_output: f64,

    /// Estatísticas
    pub stats: ControlStats,

    /// Threshold para estado estável
    pub steady_threshold: f64,
}

// Implementação manual de Serialize/Deserialize para pular pid
impl Serialize for CyberneticState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("CyberneticState", 6)?;
        state.serialize_field("control_mode", &self.control_mode)?;
        state.serialize_field("feedback_level", &self.feedback_level)?;
        state.serialize_field("setpoint", &self.setpoint)?;
        state.serialize_field("process_variable", &self.process_variable)?;
        state.serialize_field("control_output", &self.control_output)?;
        state.serialize_field("stats", &self.stats)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for CyberneticState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct CyberneticStateHelper {
            control_mode: ControlMode,
            feedback_level: i8,
            setpoint: f64,
            process_variable: f64,
            control_output: f64,
            stats: ControlStats,
        }

        let helper = CyberneticStateHelper::deserialize(deserializer)?;
        let mut state = CyberneticState::new(helper.control_mode);
        state.feedback_level = helper.feedback_level;
        state.setpoint = helper.setpoint;
        state.process_variable = helper.process_variable;
        state.control_output = helper.control_output;
        state.stats = helper.stats;
        Ok(state)
    }
}

impl CyberneticState {
    /// Cria novo estado cibernético
    pub fn new(control_mode: ControlMode) -> Self {
        let pid_config = Self::config_for_mode(control_mode);
        let mut pid = PidController::new(pid_config);
        pid.set_setpoint(0.0);

        Self {
            control_mode,
            feedback_level: 0,
            pid,
            setpoint: 0.0,
            process_variable: 0.0,
            control_output: 0.0,
            stats: ControlStats::default(),
            steady_threshold: 0.01,
        }
    }

    /// Cria estado a partir de ByteSil (8 bits)
    pub fn from_byte_sil(byte: ByteSil) -> Self {
        let control_mode = ControlMode::from_theta(byte.theta);
        let feedback_level = byte.rho;

        let mut state = Self::new(control_mode);
        state.feedback_level = feedback_level;

        // Ajusta ganhos baseado no feedback_level
        let gain_multiplier = 1.0 + (feedback_level as f64 * 0.1);
        let kp = state.pid.config().kp * gain_multiplier;
        let ki = state.pid.config().ki * gain_multiplier.sqrt();
        let kd = state.pid.config().kd * gain_multiplier.sqrt();
        state.pid.set_gains(kp, ki, kd);

        state
    }

    /// Extrai estado cibernético de um SilState completo
    pub fn from_sil_state(state: &SilState) -> Self {
        let byte = state.layer(CYBERNETIC_LAYER);
        Self::from_byte_sil(byte)
    }

    /// Converte para ByteSil
    pub fn to_byte_sil(&self) -> ByteSil {
        let theta = self.control_mode.to_theta();
        ByteSil::new(self.feedback_level, theta)
    }

    /// Atualiza camada L8 em um SilState
    pub fn apply_to(&self, state: &SilState) -> SilState {
        state.with_layer(CYBERNETIC_LAYER, self.to_byte_sil())
    }

    /// Retorna configuração PID para modo de controle
    fn config_for_mode(mode: ControlMode) -> PidConfig {
        match mode {
            ControlMode::Manual => PidConfig {
                kp: 0.0,
                ki: 0.0,
                kd: 0.0,
                ..Default::default()
            },
            ControlMode::OpenLoop => PidConfig {
                kp: 1.0,
                ki: 0.0,
                kd: 0.0,
                ..Default::default()
            },
            ControlMode::ClosedLoop => PidConfig::default(),
            ControlMode::Adaptive => PidConfig::aggressive(),
            ControlMode::Predictive => PidConfig {
                kp: 1.5,
                ki: 0.3,
                kd: 0.2,
                ..Default::default()
            },
            ControlMode::Learning => PidConfig::conservative(),
            ControlMode::Emergent => PidConfig {
                kp: 0.3,
                ki: 0.1,
                kd: 0.05,
                ..Default::default()
            },
            ControlMode::Autonomous => PidConfig::aggressive(),
        }
    }

    /// Define setpoint
    pub fn set_setpoint(&mut self, setpoint: f64) {
        self.setpoint = setpoint;
        self.pid.set_setpoint(setpoint);
    }

    /// Atualiza estado com novo valor do processo
    pub fn update(&mut self, process_variable: f64, dt: f64) -> f64 {
        self.process_variable = process_variable;

        // Calcula controle
        self.control_output = match self.control_mode {
            ControlMode::Manual => 0.0,
            ControlMode::OpenLoop => self.setpoint, // Passthrough
            _ => self.pid.update_with_dt(process_variable, dt),
        };

        // Atualiza estatísticas
        let error = self.setpoint - process_variable;
        let is_steady = error.abs() < self.steady_threshold;
        self.stats.update(error, is_steady);

        // Detecta overshoot
        let prev_error = self.pid.error();
        if prev_error.signum() != error.signum() && error.abs() > self.steady_threshold {
            self.stats.record_overshoot();
        }

        self.control_output
    }

    /// Define modo de controle
    pub fn with_control_mode(mut self, mode: ControlMode) -> Self {
        self.control_mode = mode;
        let config = Self::config_for_mode(mode);
        self.pid = PidController::new(config);
        self.pid.set_setpoint(self.setpoint);
        self
    }

    /// Verifica se está em estado estável
    pub fn is_steady(&self) -> bool {
        let error = self.setpoint - self.process_variable;
        error.abs() < self.steady_threshold
    }

    /// Retorna erro atual
    pub fn error(&self) -> f64 {
        self.setpoint - self.process_variable
    }

    /// Retorna componentes PID
    pub fn pid_components(&self) -> (f64, f64, f64) {
        self.pid.components()
    }

    /// Reseta controlador
    pub fn reset(&mut self) {
        self.pid.reset();
        self.stats.reset();
        self.control_output = 0.0;
    }
}

impl Default for CyberneticState {
    fn default() -> Self {
        Self::new(ControlMode::ClosedLoop)
    }
}

// Implementação do trait SilComponent
impl sil_core::SilComponent for CyberneticState {
    fn name(&self) -> &'static str {
        "CyberneticState"
    }

    fn layers(&self) -> &[u8] {
        &[CYBERNETIC_LAYER as u8]
    }

    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    fn is_ready(&self) -> bool {
        true
    }
}

// =============================================================================
// Testes
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_creation() {
        let state = CyberneticState::new(ControlMode::ClosedLoop);
        assert_eq!(state.control_mode, ControlMode::ClosedLoop);
        assert_eq!(state.setpoint, 0.0);
    }

    #[test]
    fn test_from_byte_sil() {
        let byte = ByteSil::new(3, 4); // feedback_level=3, ClosedLoop
        let state = CyberneticState::from_byte_sil(byte);

        assert_eq!(state.control_mode, ControlMode::ClosedLoop);
        assert_eq!(state.feedback_level, 3);
    }

    #[test]
    fn test_roundtrip() {
        let original = CyberneticState::new(ControlMode::Adaptive);
        let byte = original.to_byte_sil();
        let recovered = CyberneticState::from_byte_sil(byte);

        assert_eq!(original.control_mode, recovered.control_mode);
    }

    #[test]
    fn test_update() {
        let mut state = CyberneticState::new(ControlMode::ClosedLoop);
        state.set_setpoint(10.0);

        let output = state.update(0.0, 0.1);
        assert!(output > 0.0); // Deve tentar aumentar para alcançar setpoint
    }

    #[test]
    fn test_manual_mode() {
        let mut state = CyberneticState::new(ControlMode::Manual);
        state.set_setpoint(10.0);

        let output = state.update(0.0, 0.1);
        assert_eq!(output, 0.0); // Manual não gera saída
    }

    #[test]
    fn test_steady_state() {
        let mut state = CyberneticState::new(ControlMode::ClosedLoop);
        state.set_setpoint(5.0);
        state.process_variable = 5.001;

        assert!(state.is_steady());
    }

    #[test]
    fn test_stats_update() {
        let mut state = CyberneticState::new(ControlMode::ClosedLoop);
        state.set_setpoint(10.0);

        for _ in 0..10 {
            state.update(5.0, 0.1);
        }

        assert_eq!(state.stats.cycles, 10);
        assert!(state.stats.mean_error > 0.0);
    }

    #[test]
    fn test_sil_state_integration() {
        let sil_state = SilState::neutral();
        let cyber = CyberneticState::new(ControlMode::Predictive);

        let updated = cyber.apply_to(&sil_state);

        let byte = updated.layer(CYBERNETIC_LAYER);
        assert_eq!(byte.theta, ControlMode::Predictive.to_theta());
    }
}
