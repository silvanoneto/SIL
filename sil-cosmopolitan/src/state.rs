//! # CosmopoliticalState — Estado da Camada LA
//!
//! Estado completo da camada cosmopolitica, integrando etica,
//! especies, direitos e hospitalidade.

use serde::{Deserialize, Serialize};
use sil_core::{ByteSil, SilState};

use crate::{
    AgentSpecies, EthicalMode, HospitalityPhase, HospitalitySession,
    MoralCircle, RightsRegistry,
};

/// Indice da camada cosmopolitica
pub const COSMOPOLITICAL_LAYER: usize = 0xA;

/// Estado da camada cosmopolitica
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CosmopoliticalState {
    /// Framework etico ativo (theta)
    pub ethical_mode: EthicalMode,

    /// Circulo moral atual (derivado de rho)
    pub moral_circle: MoralCircle,

    /// Especie do agente
    pub species: AgentSpecies,

    /// Registro de direitos e deveres
    #[serde(skip)]
    pub rights_registry: RightsRegistry,

    /// Sessao de hospitalidade ativa (se houver)
    pub hospitality_session: Option<HospitalitySession>,

    /// Nivel de confianca geral
    pub trust_level: f64,

    /// Escopo etico (derivado de rho, 0.0 - 1.0)
    pub scope: f64,
}

impl CosmopoliticalState {
    /// Cria novo estado cosmopolitico
    pub fn new(species: AgentSpecies) -> Self {
        Self {
            ethical_mode: EthicalMode::default(),
            moral_circle: MoralCircle::default(),
            species,
            rights_registry: RightsRegistry::with_fundamentals(),
            hospitality_session: None,
            trust_level: 0.5,
            scope: 0.5,
        }
    }

    /// Cria estado a partir de ByteSil (8 bits)
    pub fn from_byte_sil(byte: ByteSil, species: AgentSpecies) -> Self {
        let ethical_mode = EthicalMode::from_theta(byte.theta);
        let moral_circle = MoralCircle::from_rho(byte.rho);
        let scope = (byte.rho as f64 + 8.0) / 15.0;

        Self {
            ethical_mode,
            moral_circle,
            species,
            rights_registry: RightsRegistry::with_fundamentals(),
            hospitality_session: None,
            trust_level: 0.5,
            scope,
        }
    }

    /// Extrai estado cosmopolitico de um SilState completo
    pub fn from_sil_state(state: &SilState, species: AgentSpecies) -> Self {
        let byte = state.layer(COSMOPOLITICAL_LAYER);
        Self::from_byte_sil(byte, species)
    }

    /// Converte para ByteSil
    pub fn to_byte_sil(&self) -> ByteSil {
        let theta = self.ethical_mode.to_theta();
        let rho = self.moral_circle.to_rho();
        ByteSil::new(rho, theta)
    }

    /// Atualiza camada LA em um SilState
    pub fn apply_to(&self, state: &SilState) -> SilState {
        state.with_layer(COSMOPOLITICAL_LAYER, self.to_byte_sil())
    }

    /// Define framework etico
    pub fn with_ethical_mode(mut self, mode: EthicalMode) -> Self {
        self.ethical_mode = mode;
        self
    }

    /// Define circulo moral
    pub fn with_moral_circle(mut self, circle: MoralCircle) -> Self {
        self.moral_circle = circle;
        // Atualiza scope baseado no circulo
        self.scope = (circle.index() as f64) / 8.0;
        self
    }

    /// Define especie
    pub fn with_species(mut self, species: AgentSpecies) -> Self {
        self.species = species;
        self
    }

    /// Inicia sessao de hospitalidade
    pub fn start_hospitality(&mut self) {
        self.hospitality_session = Some(HospitalitySession::new());
    }

    /// Finaliza sessao de hospitalidade
    pub fn end_hospitality(&mut self) {
        self.hospitality_session = None;
    }

    /// Verifica se ha sessao de hospitalidade ativa
    pub fn is_hosting(&self) -> bool {
        self.hospitality_session
            .as_ref()
            .map(|s: &HospitalitySession| s.is_active())
            .unwrap_or(false)
    }

    /// Fase atual da hospitalidade
    pub fn hospitality_phase(&self) -> HospitalityPhase {
        self.hospitality_session
            .as_ref()
            .map(|s: &HospitalitySession| s.phase)
            .unwrap_or(HospitalityPhase::None)
    }

    /// Verifica se o framework etico e compativel com a especie
    pub fn is_compatible(&self) -> bool {
        self.ethical_mode.is_compatible_with(&self.species)
    }

    /// Expande o circulo moral
    pub fn expand_circle(&mut self) -> bool {
        if let Some(expanded) = self.moral_circle.expand() {
            self.moral_circle = expanded;
            self.scope = (expanded.index() as f64) / 8.0;
            true
        } else {
            false
        }
    }

    /// Contrai o circulo moral
    pub fn contract_circle(&mut self) -> bool {
        if let Some(contracted) = self.moral_circle.contract() {
            self.moral_circle = contracted;
            self.scope = (contracted.index() as f64) / 8.0;
            true
        } else {
            false
        }
    }

    /// Sugere circulo moral para o framework etico atual
    pub fn suggested_circle(&self) -> MoralCircle {
        MoralCircle::suggested_for(&self.ethical_mode)
    }

    /// Verifica se o circulo atual e adequado ao framework
    pub fn circle_matches_framework(&self) -> bool {
        self.moral_circle >= self.suggested_circle()
    }

    /// Atualiza nivel de confianca
    pub fn update_trust(&mut self, delta: f64) {
        self.trust_level = (self.trust_level + delta).clamp(0.0, 1.0);
    }

    /// Lista direitos aplicaveis a especie
    pub fn applicable_rights(&self) -> Vec<&crate::Right> {
        self.rights_registry.rights_for(&self.species)
    }

    /// Lista deveres aplicaveis a especie
    pub fn applicable_duties(&self) -> Vec<&crate::Duty> {
        self.rights_registry.duties_for(&self.species)
    }
}

impl Default for CosmopoliticalState {
    fn default() -> Self {
        Self::new(AgentSpecies::Unknown)
    }
}

// Implementacao do trait SilComponent
impl sil_core::SilComponent for CosmopoliticalState {
    fn name(&self) -> &'static str {
        "CosmopoliticalState"
    }

    fn layers(&self) -> &[u8] {
        &[COSMOPOLITICAL_LAYER as u8]
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
        let state = CosmopoliticalState::new(AgentSpecies::Human);

        assert_eq!(state.species, AgentSpecies::Human);
        assert!(state.is_compatible());
    }

    #[test]
    fn test_from_byte_sil() {
        // Criar byte com Ubuntu (theta=10) e circulo amplo (rho=7)
        let byte = ByteSil::new(7, 10);
        let state = CosmopoliticalState::from_byte_sil(byte, AgentSpecies::Human);

        assert_eq!(state.ethical_mode, EthicalMode::Ubuntu);
        assert!(state.scope > 0.9); // rho=7 -> escopo alto
    }

    #[test]
    fn test_roundtrip() {
        let original = CosmopoliticalState::new(AgentSpecies::Digital)
            .with_ethical_mode(EthicalMode::Gaian)
            .with_moral_circle(MoralCircle::Biosphere);

        let byte = original.to_byte_sil();
        let recovered = CosmopoliticalState::from_byte_sil(byte, AgentSpecies::Digital);

        assert_eq!(original.ethical_mode, recovered.ethical_mode);
    }

    #[test]
    fn test_circle_expansion() {
        let mut state = CosmopoliticalState::new(AgentSpecies::Human)
            .with_moral_circle(MoralCircle::Humanity);

        assert!(state.expand_circle());
        assert_eq!(state.moral_circle, MoralCircle::Sentient);

        // Scope deve aumentar
        assert!(state.scope > 0.5);
    }

    #[test]
    fn test_hospitality_lifecycle() {
        let mut state = CosmopoliticalState::new(AgentSpecies::Human);

        assert!(!state.is_hosting());
        assert_eq!(state.hospitality_phase(), HospitalityPhase::None);

        state.start_hospitality();
        // Sessao criada mas ainda em fase None
        assert!(!state.is_hosting());

        // Avancar para Acolhimento
        if let Some(ref mut session) = state.hospitality_session {
            session.advance().unwrap();
        }
        assert!(state.is_hosting());

        state.end_hospitality();
        assert!(!state.is_hosting());
    }

    #[test]
    fn test_framework_circle_matching() {
        let state = CosmopoliticalState::new(AgentSpecies::Human)
            .with_ethical_mode(EthicalMode::Gaian)
            .with_moral_circle(MoralCircle::Humanity);

        // Gaian sugere Biosphere, Humanity e menor
        assert!(!state.circle_matches_framework());

        let state2 = state.with_moral_circle(MoralCircle::Biosphere);
        assert!(state2.circle_matches_framework());
    }

    #[test]
    fn test_sil_state_integration() {
        let sil_state = SilState::neutral();
        let cosmo = CosmopoliticalState::new(AgentSpecies::Human)
            .with_ethical_mode(EthicalMode::Ubuntu);

        let updated = cosmo.apply_to(&sil_state);

        // Verificar que a camada LA foi atualizada
        let byte = updated.layer(COSMOPOLITICAL_LAYER);
        assert_eq!(byte.theta, EthicalMode::Ubuntu.to_theta());
    }
}
