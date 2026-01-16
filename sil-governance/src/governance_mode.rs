//! # GovernanceMode — Regimes de Governanca
//!
//! Os 8 regimes de governanca para L9 (Geopolitico).

use serde::{Deserialize, Serialize};
use std::fmt;

/// Regimes de governanca (theta de L9)
///
/// Cada regime representa um modo diferente de organizacao
/// e tomada de decisao em um territorio.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum GovernanceMode {
    /// Auto-governanca plena
    /// O no toma todas as decisoes internamente.
    Autonomous = 0,

    /// Federacao com delegacao
    /// Algumas decisoes sao delegadas a um nivel superior.
    Federated = 2,

    /// Alianca com parceiros
    /// Decisoes coordenadas entre aliados.
    Allied = 4,

    /// Neutralidade declarada
    /// O no nao participa de conflitos ou aliancas.
    Neutral = 6,

    /// Territorio em disputa
    /// Soberania contestada por multiplas partes.
    Disputed = 8,

    /// Sob controle externo
    /// Soberania limitada por forca externa.
    Occupied = 10,

    /// Regime transitorio
    /// Governanca em processo de mudanca.
    Transitional = 12,

    /// Estado falido
    /// Sem governanca efetiva.
    Collapsed = 14,
}

impl GovernanceMode {
    /// Cria GovernanceMode a partir de valor theta (0-15)
    pub fn from_theta(theta: u8) -> Self {
        match theta & 0x0E {
            0 => Self::Autonomous,
            2 => Self::Federated,
            4 => Self::Allied,
            6 => Self::Neutral,
            8 => Self::Disputed,
            10 => Self::Occupied,
            12 => Self::Transitional,
            _ => Self::Collapsed,
        }
    }

    /// Retorna o valor theta correspondente
    pub fn to_theta(&self) -> u8 {
        *self as u8
    }

    /// Nome do regime
    pub fn name(&self) -> &'static str {
        match self {
            Self::Autonomous => "Autonomous",
            Self::Federated => "Federated",
            Self::Allied => "Allied",
            Self::Neutral => "Neutral",
            Self::Disputed => "Disputed",
            Self::Occupied => "Occupied",
            Self::Transitional => "Transitional",
            Self::Collapsed => "Collapsed",
        }
    }

    /// Descricao do regime
    pub fn description(&self) -> &'static str {
        match self {
            Self::Autonomous => "Full self-governance",
            Self::Federated => "Delegation to higher level",
            Self::Allied => "Coordinated with allies",
            Self::Neutral => "Non-aligned, no conflicts",
            Self::Disputed => "Sovereignty contested",
            Self::Occupied => "Limited sovereignty",
            Self::Transitional => "Governance in flux",
            Self::Collapsed => "No effective governance",
        }
    }

    /// Nivel de soberania (0.0 = nenhuma, 1.0 = plena)
    pub fn sovereignty_level(&self) -> f64 {
        match self {
            Self::Autonomous => 1.0,
            Self::Federated => 0.8,
            Self::Allied => 0.7,
            Self::Neutral => 0.9,
            Self::Disputed => 0.4,
            Self::Occupied => 0.2,
            Self::Transitional => 0.5,
            Self::Collapsed => 0.0,
        }
    }

    /// Verifica se o regime e estavel
    pub fn is_stable(&self) -> bool {
        matches!(
            self,
            Self::Autonomous | Self::Federated | Self::Allied | Self::Neutral
        )
    }

    /// Verifica se o regime esta em crise
    pub fn is_in_crisis(&self) -> bool {
        matches!(
            self,
            Self::Disputed | Self::Occupied | Self::Transitional | Self::Collapsed
        )
    }

    /// Pode fazer aliancas?
    pub fn can_form_alliances(&self) -> bool {
        matches!(
            self,
            Self::Autonomous | Self::Federated | Self::Allied | Self::Neutral
        )
    }

    /// Pode aceitar propostas?
    pub fn can_accept_proposals(&self) -> bool {
        !matches!(self, Self::Collapsed | Self::Occupied)
    }
}

impl Default for GovernanceMode {
    fn default() -> Self {
        Self::Autonomous
    }
}

impl fmt::Display for GovernanceMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (sov={})", self.name(), self.sovereignty_level())
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
        assert_eq!(GovernanceMode::from_theta(0), GovernanceMode::Autonomous);
        assert_eq!(GovernanceMode::from_theta(2), GovernanceMode::Federated);
        assert_eq!(GovernanceMode::from_theta(14), GovernanceMode::Collapsed);
    }

    #[test]
    fn test_roundtrip() {
        for theta in (0..16).step_by(2) {
            let mode = GovernanceMode::from_theta(theta);
            assert_eq!(mode.to_theta(), theta);
        }
    }

    #[test]
    fn test_sovereignty_ordering() {
        assert!(GovernanceMode::Autonomous.sovereignty_level() >
                GovernanceMode::Federated.sovereignty_level());
        assert!(GovernanceMode::Federated.sovereignty_level() >
                GovernanceMode::Occupied.sovereignty_level());
    }

    #[test]
    fn test_stability() {
        assert!(GovernanceMode::Autonomous.is_stable());
        assert!(!GovernanceMode::Disputed.is_stable());
        assert!(GovernanceMode::Disputed.is_in_crisis());
    }
}
