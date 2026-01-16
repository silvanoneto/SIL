//! # Bell States — Estados de Bell e Teleportação
//!
//! Implementa os quatro estados de Bell maximamente emaranhados
//! e protocolo de teleportação quântica para SIL.
//!
//! ## Estados de Bell
//!
//! ```text
//! |Φ+⟩ = (|00⟩ + |11⟩) / √2
//! |Φ-⟩ = (|00⟩ - |11⟩) / √2
//! |Ψ+⟩ = (|01⟩ + |10⟩) / √2
//! |Ψ-⟩ = (|01⟩ - |10⟩) / √2
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;
use sil_core::{ByteSil, SilState};

/// Tipo de correlação (interpretação theta de LE)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum CorrelationType {
    /// Sem correlação
    None = 0,
    /// Correlação clássica
    Classical = 2,
    /// Correlação quântica fraca
    #[default]
    Weak = 4,
    /// Correlação quântica forte
    Strong = 6,
    /// Estado de Bell Φ+
    BellPhiPlus = 8,
    /// Estado de Bell Φ-
    BellPhiMinus = 10,
    /// Estado de Bell Ψ+
    BellPsiPlus = 12,
    /// Estado de Bell Ψ-
    BellPsiMinus = 14,
}

impl CorrelationType {
    /// Cria CorrelationType a partir de theta (0-15)
    pub fn from_theta(theta: u8) -> Self {
        match theta & 0b1110 {
            0 => Self::None,
            2 => Self::Classical,
            4 => Self::Weak,
            6 => Self::Strong,
            8 => Self::BellPhiPlus,
            10 => Self::BellPhiMinus,
            12 => Self::BellPsiPlus,
            14 => Self::BellPsiMinus,
            _ => Self::Weak,
        }
    }

    /// Converte para theta
    pub fn to_theta(self) -> u8 {
        self as u8
    }

    /// Verifica se é estado de Bell
    pub fn is_bell_state(&self) -> bool {
        matches!(
            self,
            Self::BellPhiPlus | Self::BellPhiMinus | Self::BellPsiPlus | Self::BellPsiMinus
        )
    }

    /// Verifica se é emaranhado
    pub fn is_entangled(&self) -> bool {
        !matches!(self, Self::None | Self::Classical)
    }

    /// Força da correlação (0.0-1.0)
    pub fn strength(&self) -> f64 {
        match self {
            Self::None => 0.0,
            Self::Classical => 0.25,
            Self::Weak => 0.5,
            Self::Strong => 0.75,
            Self::BellPhiPlus | Self::BellPhiMinus | Self::BellPsiPlus | Self::BellPsiMinus => 1.0,
        }
    }

    /// Nome descritivo
    pub fn name(&self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Classical => "Classical",
            Self::Weak => "Weak",
            Self::Strong => "Strong",
            Self::BellPhiPlus => "Φ+",
            Self::BellPhiMinus => "Φ-",
            Self::BellPsiPlus => "Ψ+",
            Self::BellPsiMinus => "Ψ-",
        }
    }
}

impl fmt::Display for CorrelationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Estado de Bell
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BellState {
    /// |Φ+⟩ = (|00⟩ + |11⟩) / √2
    PhiPlus,
    /// |Φ-⟩ = (|00⟩ - |11⟩) / √2
    PhiMinus,
    /// |Ψ+⟩ = (|01⟩ + |10⟩) / √2
    PsiPlus,
    /// |Ψ-⟩ = (|01⟩ - |10⟩) / √2
    PsiMinus,
}

impl BellState {
    /// Cria a partir de CorrelationType
    pub fn from_correlation(corr: CorrelationType) -> Option<Self> {
        match corr {
            CorrelationType::BellPhiPlus => Some(Self::PhiPlus),
            CorrelationType::BellPhiMinus => Some(Self::PhiMinus),
            CorrelationType::BellPsiPlus => Some(Self::PsiPlus),
            CorrelationType::BellPsiMinus => Some(Self::PsiMinus),
            _ => None,
        }
    }

    /// Converte para CorrelationType
    pub fn to_correlation(&self) -> CorrelationType {
        match self {
            Self::PhiPlus => CorrelationType::BellPhiPlus,
            Self::PhiMinus => CorrelationType::BellPhiMinus,
            Self::PsiPlus => CorrelationType::BellPsiPlus,
            Self::PsiMinus => CorrelationType::BellPsiMinus,
        }
    }

    /// Resultado da medição de Alice (0 ou 1)
    /// dado o resultado de Bob (0 ou 1)
    pub fn correlate(&self, bob_result: bool) -> bool {
        match self {
            // Φ+: mesmo resultado
            Self::PhiPlus => bob_result,
            // Φ-: mesmo resultado
            Self::PhiMinus => bob_result,
            // Ψ+: resultado oposto
            Self::PsiPlus => !bob_result,
            // Ψ-: resultado oposto
            Self::PsiMinus => !bob_result,
        }
    }

    /// Fase relativa (-1 ou +1)
    pub fn phase(&self) -> i8 {
        match self {
            Self::PhiPlus | Self::PsiPlus => 1,
            Self::PhiMinus | Self::PsiMinus => -1,
        }
    }

    /// Paridade (mesmo ou diferente)
    pub fn parity(&self) -> bool {
        match self {
            // Phi: mesma paridade (00 ou 11)
            Self::PhiPlus | Self::PhiMinus => true,
            // Psi: paridade diferente (01 ou 10)
            Self::PsiPlus | Self::PsiMinus => false,
        }
    }

    /// Nome em notação bra-ket
    pub fn name(&self) -> &'static str {
        match self {
            Self::PhiPlus => "|Φ+⟩",
            Self::PhiMinus => "|Φ-⟩",
            Self::PsiPlus => "|Ψ+⟩",
            Self::PsiMinus => "|Ψ-⟩",
        }
    }
}

impl Default for BellState {
    fn default() -> Self {
        Self::PhiPlus
    }
}

impl fmt::Display for BellState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Par emaranhado
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntangledPair {
    /// ID do par
    pub id: u64,
    /// Estado de Bell
    pub bell_state: BellState,
    /// Estado de Alice
    pub alice_state: SilState,
    /// Estado de Bob
    pub bob_state: SilState,
    /// Correlação mantida?
    pub is_coherent: bool,
}

impl EntangledPair {
    /// Cria novo par emaranhado
    pub fn new(id: u64, bell_state: BellState) -> Self {
        // Cria estados correlacionados
        let alice_state = SilState::neutral();
        let bob_state = SilState::neutral();

        Self {
            id,
            bell_state,
            alice_state,
            bob_state,
            is_coherent: true,
        }
    }

    /// Cria par a partir de dois estados
    pub fn from_states(id: u64, alice: SilState, bob: SilState, bell_state: BellState) -> Self {
        Self {
            id,
            bell_state,
            alice_state: alice,
            bob_state: bob,
            is_coherent: true,
        }
    }

    /// Mede estado de Alice, retorna resultado
    pub fn measure_alice(&mut self, layer: usize) -> u8 {
        let byte = self.alice_state.layer(layer);
        let result = byte.rho as u8;

        // Colapsa correlação em Bob
        if self.is_coherent {
            let bob_result = if self.bell_state.parity() {
                result
            } else {
                result.wrapping_neg()
            };

            let bob_byte = ByteSil::new(bob_result as i8, self.bob_state.layer(layer).theta);
            self.bob_state = self.bob_state.with_layer(layer, bob_byte);
            self.is_coherent = false;
        }

        result
    }

    /// Mede estado de Bob, retorna resultado
    pub fn measure_bob(&mut self, layer: usize) -> u8 {
        let byte = self.bob_state.layer(layer);
        let result = byte.rho as u8;

        // Colapsa correlação em Alice
        if self.is_coherent {
            let alice_result = if self.bell_state.parity() {
                result
            } else {
                result.wrapping_neg()
            };

            let alice_byte = ByteSil::new(alice_result as i8, self.alice_state.layer(layer).theta);
            self.alice_state = self.alice_state.with_layer(layer, alice_byte);
            self.is_coherent = false;
        }

        result
    }

    /// Decoerência - quebra emaranhamento
    pub fn decohere(&mut self) {
        self.is_coherent = false;
    }
}

/// Teleporta estado de uma camada
///
/// Protocolo simplificado de teleportação quântica:
/// 1. Alice e Bob compartilham par emaranhado
/// 2. Alice faz medição de Bell no estado a teleportar + seu qubit
/// 3. Alice envia resultado clássico (2 bits)
/// 4. Bob aplica correção baseado nos 2 bits
pub fn teleport(
    source_state: &SilState,
    layer: usize,
    pair: &mut EntangledPair,
) -> Option<ByteSil> {
    if !pair.is_coherent {
        return None;
    }

    // Passo 1: Medição de Bell (simplificada)
    let source_byte = source_state.layer(layer);
    let alice_byte = pair.alice_state.layer(layer);

    // Resultado da medição (2 bits clássicos)
    let bit1 = (source_byte.rho ^ alice_byte.rho) & 1;
    let bit2 = (source_byte.theta ^ alice_byte.theta) & 1;

    // Passo 2: Bob aplica correção
    let bob_byte = pair.bob_state.layer(layer);
    let corrected_rho = if bit1 != 0 {
        bob_byte.rho.wrapping_neg()
    } else {
        bob_byte.rho
    };
    let corrected_theta = if bit2 != 0 {
        bob_byte.theta.wrapping_add(8) & 0xF
    } else {
        bob_byte.theta
    };

    // Combina com estado fonte
    let result = ByteSil::new(
        source_byte.rho.wrapping_add(corrected_rho),
        source_byte.theta.wrapping_add(corrected_theta) & 0xF,
    );

    // Emaranhamento consumido
    pair.is_coherent = false;

    Some(result)
}

/// Teleporta estado completo (todas as camadas)
pub fn teleport_full(
    source_state: &SilState,
    pairs: &mut [EntangledPair],
) -> Option<SilState> {
    if pairs.len() < 16 {
        return None;
    }

    let mut result = SilState::neutral();

    for (layer, pair) in pairs.iter_mut().enumerate().take(16) {
        if let Some(byte) = teleport(source_state, layer, pair) {
            result = result.with_layer(layer, byte);
        } else {
            return None; // Falha em alguma camada
        }
    }

    Some(result)
}

// =============================================================================
// Testes
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correlation_type_from_theta() {
        assert_eq!(CorrelationType::from_theta(0), CorrelationType::None);
        assert_eq!(CorrelationType::from_theta(4), CorrelationType::Weak);
        assert_eq!(CorrelationType::from_theta(8), CorrelationType::BellPhiPlus);
    }

    #[test]
    fn test_correlation_type_roundtrip() {
        for theta in (0..16).step_by(2) {
            let corr = CorrelationType::from_theta(theta);
            assert_eq!(corr.to_theta(), theta);
        }
    }

    #[test]
    fn test_bell_state_correlation() {
        // PhiPlus: mesma medição
        assert_eq!(BellState::PhiPlus.correlate(true), true);
        assert_eq!(BellState::PhiPlus.correlate(false), false);

        // PsiPlus: medição oposta
        assert_eq!(BellState::PsiPlus.correlate(true), false);
        assert_eq!(BellState::PsiPlus.correlate(false), true);
    }

    #[test]
    fn test_bell_state_parity() {
        assert!(BellState::PhiPlus.parity()); // 00 ou 11
        assert!(BellState::PhiMinus.parity());
        assert!(!BellState::PsiPlus.parity()); // 01 ou 10
        assert!(!BellState::PsiMinus.parity());
    }

    #[test]
    fn test_bell_state_phase() {
        assert_eq!(BellState::PhiPlus.phase(), 1);
        assert_eq!(BellState::PhiMinus.phase(), -1);
        assert_eq!(BellState::PsiPlus.phase(), 1);
        assert_eq!(BellState::PsiMinus.phase(), -1);
    }

    #[test]
    fn test_entangled_pair_creation() {
        let pair = EntangledPair::new(1, BellState::PhiPlus);

        assert!(pair.is_coherent);
        assert_eq!(pair.bell_state, BellState::PhiPlus);
    }

    #[test]
    fn test_measurement_collapses_entanglement() {
        let mut pair = EntangledPair::new(1, BellState::PhiPlus);

        assert!(pair.is_coherent);
        let _ = pair.measure_alice(0);
        assert!(!pair.is_coherent);
    }

    #[test]
    fn test_decohere() {
        let mut pair = EntangledPair::new(1, BellState::PhiPlus);

        pair.decohere();
        assert!(!pair.is_coherent);
    }

    #[test]
    fn test_teleport_single_layer() {
        let source = SilState::neutral();
        let mut pair = EntangledPair::new(1, BellState::PhiPlus);

        let result = teleport(&source, 0, &mut pair);
        assert!(result.is_some());
        assert!(!pair.is_coherent); // Emaranhamento consumido
    }

    #[test]
    fn test_teleport_requires_coherence() {
        let source = SilState::neutral();
        let mut pair = EntangledPair::new(1, BellState::PhiPlus);

        pair.decohere();

        let result = teleport(&source, 0, &mut pair);
        assert!(result.is_none());
    }

    #[test]
    fn test_is_bell_state() {
        assert!(CorrelationType::BellPhiPlus.is_bell_state());
        assert!(CorrelationType::BellPsiMinus.is_bell_state());
        assert!(!CorrelationType::Strong.is_bell_state());
        assert!(!CorrelationType::None.is_bell_state());
    }

    #[test]
    fn test_correlation_strength() {
        assert_eq!(CorrelationType::None.strength(), 0.0);
        assert_eq!(CorrelationType::BellPhiPlus.strength(), 1.0);
        assert!(CorrelationType::Strong.strength() > CorrelationType::Weak.strength());
    }
}
