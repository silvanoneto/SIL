//! # ControlMode — Modos de Controle L8
//!
//! Interpretação theta da camada cibernética.
//!
//! ## Os 8 Modos de Controle
//!
//! 0. **Manual**: Controle direto, sem automação
//! 1. **OpenLoop**: Controle aberto, sem feedback
//! 2. **ClosedLoop**: Controle fechado com feedback (PID)
//! 3. **Adaptive**: Ajusta parâmetros baseado em performance
//! 4. **Predictive**: Usa modelo preditivo (MPC)
//! 5. **Learning**: Aprende política via RL
//! 6. **Emergent**: Controle emerge do swarm
//! 7. **Autonomous**: Totalmente autônomo

use serde::{Deserialize, Serialize};
use std::fmt;

/// Modo de controle cibernético
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum ControlMode {
    /// Controle manual direto
    Manual = 0,
    /// Controle de malha aberta (sem feedback)
    OpenLoop = 2,
    /// Controle de malha fechada (PID)
    #[default]
    ClosedLoop = 4,
    /// Controle adaptativo (ajusta parâmetros)
    Adaptive = 6,
    /// Controle preditivo (Model Predictive Control)
    Predictive = 8,
    /// Controle por aprendizado (Reinforcement Learning)
    Learning = 10,
    /// Controle emergente (swarm-based)
    Emergent = 12,
    /// Controle totalmente autônomo
    Autonomous = 14,
}

impl ControlMode {
    /// Cria ControlMode a partir de theta (0-15)
    pub fn from_theta(theta: u8) -> Self {
        match theta & 0b1110 {
            0 => Self::Manual,
            2 => Self::OpenLoop,
            4 => Self::ClosedLoop,
            6 => Self::Adaptive,
            8 => Self::Predictive,
            10 => Self::Learning,
            12 => Self::Emergent,
            14 => Self::Autonomous,
            _ => Self::ClosedLoop,
        }
    }

    /// Converte para theta
    pub fn to_theta(self) -> u8 {
        self as u8
    }

    /// Retorna se usa feedback
    pub fn uses_feedback(&self) -> bool {
        !matches!(self, Self::Manual | Self::OpenLoop)
    }

    /// Retorna se é adaptativo
    pub fn is_adaptive(&self) -> bool {
        matches!(
            self,
            Self::Adaptive | Self::Predictive | Self::Learning | Self::Autonomous
        )
    }

    /// Retorna se requer aprendizado
    pub fn requires_learning(&self) -> bool {
        matches!(self, Self::Learning | Self::Autonomous)
    }

    /// Retorna se é emergente
    pub fn is_emergent(&self) -> bool {
        matches!(self, Self::Emergent | Self::Autonomous)
    }

    /// Nível de autonomia (0-3)
    pub fn autonomy_level(&self) -> u8 {
        match self {
            Self::Manual => 0,
            Self::OpenLoop | Self::ClosedLoop => 1,
            Self::Adaptive | Self::Predictive => 2,
            Self::Learning | Self::Emergent | Self::Autonomous => 3,
        }
    }

    /// Nome descritivo
    pub fn name(&self) -> &'static str {
        match self {
            Self::Manual => "Manual",
            Self::OpenLoop => "Open Loop",
            Self::ClosedLoop => "Closed Loop",
            Self::Adaptive => "Adaptive",
            Self::Predictive => "Predictive",
            Self::Learning => "Learning",
            Self::Emergent => "Emergent",
            Self::Autonomous => "Autonomous",
        }
    }
}

impl fmt::Display for ControlMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

// =============================================================================
// Testes
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_theta() {
        assert_eq!(ControlMode::from_theta(0), ControlMode::Manual);
        assert_eq!(ControlMode::from_theta(4), ControlMode::ClosedLoop);
        assert_eq!(ControlMode::from_theta(14), ControlMode::Autonomous);
    }

    #[test]
    fn test_roundtrip() {
        for theta in (0..16).step_by(2) {
            let mode = ControlMode::from_theta(theta);
            assert_eq!(mode.to_theta(), theta);
        }
    }

    #[test]
    fn test_feedback() {
        assert!(!ControlMode::Manual.uses_feedback());
        assert!(!ControlMode::OpenLoop.uses_feedback());
        assert!(ControlMode::ClosedLoop.uses_feedback());
        assert!(ControlMode::Adaptive.uses_feedback());
    }

    #[test]
    fn test_adaptive() {
        assert!(!ControlMode::Manual.is_adaptive());
        assert!(!ControlMode::ClosedLoop.is_adaptive());
        assert!(ControlMode::Adaptive.is_adaptive());
        assert!(ControlMode::Autonomous.is_adaptive());
    }

    #[test]
    fn test_autonomy_level() {
        assert_eq!(ControlMode::Manual.autonomy_level(), 0);
        assert_eq!(ControlMode::ClosedLoop.autonomy_level(), 1);
        assert_eq!(ControlMode::Adaptive.autonomy_level(), 2);
        assert_eq!(ControlMode::Autonomous.autonomy_level(), 3);
    }
}
