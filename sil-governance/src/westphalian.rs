//! # WestphalianDigital — Principios de Soberania Digital
//!
//! Baseado nos principios da Paz de Westfalia (1648), adaptados
//! para o dominio digital.
//!
//! ## Os 4 Principios
//!
//! 1. **Integridade Territorial**: O hash do estado e inviolavel
//! 2. **Nao-Intervencao**: Nenhum no altera outro sem permissao
//! 3. **Igualdade Soberana**: 1 no = 1 voz (base)
//! 4. **Auto-Determinacao**: Cada no define suas regras internas

use serde::{Deserialize, Serialize};
use std::fmt;

/// Hash de 256 bits para integridade territorial
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Hash256(pub [u8; 32]);

impl Hash256 {
    /// Hash zero (invalido)
    pub const ZERO: Self = Self([0u8; 32]);

    /// Cria novo hash
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Verifica se e zero
    pub fn is_zero(&self) -> bool {
        self.0 == [0u8; 32]
    }

    /// Cria de slice
    pub fn from_slice(slice: &[u8]) -> Option<Self> {
        if slice.len() != 32 {
            return None;
        }
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(slice);
        Some(Self(bytes))
    }
}

impl fmt::Display for Hash256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:016x}...", u64::from_be_bytes(self.0[0..8].try_into().unwrap()))
    }
}

impl Default for Hash256 {
    fn default() -> Self {
        Self::ZERO
    }
}

/// Identificador de no
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub String);

impl NodeId {
    /// Cria novo ID
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Gera ID aleatorio
    pub fn random() -> Self {
        Self(format!("node-{:016x}", rand::random::<u64>()))
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Poder de voto
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VotingPower {
    /// Sem direito a voto
    None,
    /// 1 voto (padrao)
    One,
    /// Multiplos votos (delegacao)
    Multiple(u64),
    /// Voto ponderado por stake
    Weighted(u64),
}

impl Default for VotingPower {
    fn default() -> Self {
        Self::One
    }
}

impl fmt::Display for VotingPower {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "0 votes"),
            Self::One => write!(f, "1 vote"),
            Self::Multiple(n) => write!(f, "{} votes", n),
            Self::Weighted(w) => write!(f, "weight={}", w),
        }
    }
}

/// Regras de governanca interna
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GovernanceRules {
    /// Threshold para aprovacao (0.0 - 1.0)
    pub approval_threshold: f64,

    /// Threshold para quorum (0.0 - 1.0)
    pub quorum_threshold: f64,

    /// Duracao maxima de votacao (segundos)
    pub voting_duration: u64,

    /// Permite delegacao?
    pub allow_delegation: bool,

    /// Permite veto?
    pub allow_veto: bool,

    /// Regras customizadas (key-value)
    pub custom_rules: Vec<(String, String)>,
}

impl GovernanceRules {
    /// Cria regras padrao
    pub fn new() -> Self {
        Self {
            approval_threshold: 0.5,
            quorum_threshold: 0.33,
            voting_duration: 300,
            allow_delegation: false,
            allow_veto: false,
            custom_rules: Vec::new(),
        }
    }

    /// Regras democraticas (maioria simples)
    pub fn democratic() -> Self {
        Self {
            approval_threshold: 0.5,
            quorum_threshold: 0.5,
            voting_duration: 600,
            allow_delegation: true,
            allow_veto: false,
            custom_rules: Vec::new(),
        }
    }

    /// Regras de consenso (unanimidade)
    pub fn consensus() -> Self {
        Self {
            approval_threshold: 1.0,
            quorum_threshold: 1.0,
            voting_duration: 3600,
            allow_delegation: false,
            allow_veto: true,
            custom_rules: Vec::new(),
        }
    }

    /// Regras de supermaioria
    pub fn supermajority() -> Self {
        Self {
            approval_threshold: 0.67,
            quorum_threshold: 0.5,
            voting_duration: 300,
            allow_delegation: true,
            allow_veto: false,
            custom_rules: Vec::new(),
        }
    }

    /// Adiciona regra customizada
    pub fn with_rule(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.custom_rules.push((key.into(), value.into()));
        self
    }
}

/// Trait para implementar os principios Westfalianos digitais
pub trait WestphalianDigital {
    /// Principio 1: Integridade Territorial
    /// Retorna o hash do estado interno (inviolavel).
    fn territorial_integrity(&self) -> Hash256;

    /// Principio 2: Nao-Intervencao
    /// Verifica se intervencao em outro no e permitida.
    fn non_intervention(&self, other: &NodeId) -> bool;

    /// Principio 3: Igualdade Soberana
    /// Retorna o poder de voto base do no.
    fn sovereign_equality(&self) -> VotingPower;

    /// Principio 4: Auto-Determinacao
    /// Retorna as regras de governanca interna.
    fn self_determination(&self) -> &GovernanceRules;

    /// Verifica se todos os principios estao sendo respeitados
    fn is_westphalian_compliant(&self) -> bool {
        // Hash deve ser valido
        if self.territorial_integrity().is_zero() {
            return false;
        }

        // Deve ter direito a voto
        if matches!(self.sovereign_equality(), VotingPower::None) {
            return false;
        }

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
    fn test_hash256() {
        let hash = Hash256::new([1u8; 32]);
        assert!(!hash.is_zero());

        let zero = Hash256::ZERO;
        assert!(zero.is_zero());
    }

    #[test]
    fn test_node_id() {
        let id = NodeId::new("test-node");
        assert_eq!(id.0, "test-node");

        let random = NodeId::random();
        assert!(random.0.starts_with("node-"));
    }

    #[test]
    fn test_voting_power() {
        assert_eq!(VotingPower::default(), VotingPower::One);
    }

    #[test]
    fn test_governance_rules() {
        let rules = GovernanceRules::democratic();
        assert_eq!(rules.approval_threshold, 0.5);
        assert!(rules.allow_delegation);

        let consensus = GovernanceRules::consensus();
        assert_eq!(consensus.approval_threshold, 1.0);
        assert!(consensus.allow_veto);
    }

    #[test]
    fn test_supermajority() {
        let rules = GovernanceRules::supermajority();
        assert!((rules.approval_threshold - 0.67).abs() < 0.01);
    }
}
