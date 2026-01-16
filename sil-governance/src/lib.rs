//! # L9-LA Governança Distribuída — Consenso e Soberania
//!
//! Camada de governança: propostas, votações, consenso distribuído.
//!
//! ## Responsabilidades
//! - Criar e rastrear propostas
//! - Gerenciar votações
//! - Atingir consenso (quórum, maioria)
//! - Executar decisões aprovadas
//!
//! ## Uso
//!
//! ```rust,no_run
//! use sil_governance::{Governance, ProposalData};
//! use sil_core::traits::{Governor, SilComponent};
//!
//! let mut gov = Governance::new().unwrap();
//!
//! // Criar proposta
//! let proposal = ProposalData::new("Aumentar quórum para 75%");
//! let id = gov.propose(proposal).unwrap();
//!
//! // Votar
//! use sil_core::traits::Vote;
//! gov.vote(&id, Vote::Yes).unwrap();
//! ```

pub mod proposal;
pub mod voting;
pub mod territory;
pub mod governance_mode;
pub mod westphalian;

pub use proposal::{ProposalData, ProposalId, ProposalState};
pub use voting::{VotingRecord, VoteCount};
pub use territory::{Territory, TerritoryId, Border, BorderProtocol, Resource, ResourceType};
pub use governance_mode::GovernanceMode;
pub use westphalian::{WestphalianDigital, Hash256, NodeId, VotingPower, GovernanceRules};

// Re-exporta Vote do core para facilitar uso
pub use sil_core::traits::Vote;

// Alias para facilitar uso do CLI
pub type Proposal = ProposalData;

use std::collections::HashMap;
use sil_core::traits::{
    Governor, GovernanceError, LayerId, SilComponent, ProposalStatus,
};
use sil_core::SilState;

/// Erro de governança específico
#[derive(Debug, thiserror::Error)]
pub enum GovernanceError2 {
    #[error("Proposal not found")]
    NotFound,
    #[error("Already voted")]
    AlreadyVoted,
    #[error("Invalid proposal")]
    Invalid(String),
    #[error("Quorum not reached")]
    NoQuorum,
    #[error("Not authorized")]
    NotAuthorized,
}

/// Configuração de governança
#[derive(Debug, Clone)]
pub struct GovernanceConfig {
    /// Percentual mínimo para aprovação (0.0 - 1.0)
    pub approval_threshold: f32,
    /// Percentual mínimo de participação (0.0 - 1.0)
    pub quorum_threshold: f32,
    /// Tempo máximo de votação em segundos
    pub voting_duration_secs: u64,
    /// ID deste nó (para tracking de quem votou)
    pub node_id: String,
}

impl Default for GovernanceConfig {
    fn default() -> Self {
        Self {
            approval_threshold: 0.5, // Maioria simples
            quorum_threshold: 0.33,  // 1/3 de quórum
            voting_duration_secs: 300,
            node_id: format!("node-{:08x}", rand::random::<u32>()),
        }
    }
}

/// Sistema de governança distribuída
#[derive(Debug)]
pub struct Governance {
    /// Configuração
    config: GovernanceConfig,
    /// Propostas ativas
    proposals: HashMap<ProposalId, (ProposalData, ProposalState)>,
    /// Registro de votações por proposta
    votes: HashMap<ProposalId, VotingRecord>,
    /// Próximo ID de proposta
    next_id: u64,
    /// Estado local
    #[allow(dead_code)]
    local_state: SilState,
}

impl Governance {
    /// Cria novo sistema de governança
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Self::with_config(GovernanceConfig::default())
    }

    /// Cria com configuração customizada
    pub fn with_config(config: GovernanceConfig) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            config,
            proposals: HashMap::new(),
            votes: HashMap::new(),
            next_id: 1,
            local_state: SilState::default(),
        })
    }

    /// Retorna configuração
    pub fn config(&self) -> &GovernanceConfig {
        &self.config
    }

    /// Lista propostas ativas
    pub fn active_proposals(&self) -> Vec<ProposalId> {
        self.proposals
            .keys()
            .filter(|id| {
                if let Some((_, state)) = self.proposals.get(id) {
                    matches!(state, ProposalState::Voting)
                } else {
                    false
                }
            })
            .copied()
            .collect()
    }

    /// Obtém detalhes de proposta
    pub fn get_proposal(&self, id: &ProposalId) -> Option<&ProposalData> {
        self.proposals.get(id).map(|(p, _)| p)
    }

    /// Obtém estado de proposta
    pub fn get_proposal_state(&self, id: &ProposalId) -> Option<ProposalState> {
        self.proposals.get(id).map(|(_, s)| *s)
    }

    /// Calcula resultado de votação
    fn calculate_result(&self, votes: &VoteCount) -> ProposalState {
        let total = votes.yes + votes.no + votes.abstain;
        if total == 0 {
            return ProposalState::Voting;
        }

        let participation = total as f32;
        let quorum_needed = (self.config.quorum_threshold * 100.0) as u64;

        // Verifica quórum
        if participation < quorum_needed as f32 {
            return ProposalState::Voting; // Ainda aguarda
        }

        // Verifica aprovação
        let yes_ratio = votes.yes as f32 / total as f32;
        if yes_ratio >= self.config.approval_threshold {
            ProposalState::Approved
        } else {
            ProposalState::Rejected
        }
    }

    /// Atualiza estado de proposta baseado em votação
    fn update_proposal_state(&mut self, id: ProposalId) {
        if let Some(voting) = self.votes.get(&id) {
            let new_state = self.calculate_result(&voting.count);
            if let Some((_, state)) = self.proposals.get_mut(&id) {
                *state = new_state;
            }
        }
    }

    /// Executa proposta aprovada
    pub fn execute(&mut self, id: &ProposalId) -> Result<(), GovernanceError2> {
        let proposal = self
            .proposals
            .get(id)
            .ok_or(GovernanceError2::NotFound)?;

        match proposal.1 {
            ProposalState::Approved => {
                // Marca como executada
                if let Some((_, state)) = self.proposals.get_mut(id) {
                    *state = ProposalState::Executed;
                }
                Ok(())
            }
            _ => Err(GovernanceError2::Invalid("Not approved".into())),
        }
    }
}

impl Default for Governance {
    fn default() -> Self {
        Self::new().expect("Failed to create Governance")
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Implementação de SilComponent
// ═══════════════════════════════════════════════════════════════════════════════

impl SilComponent for Governance {
    fn name(&self) -> &str {
        "governance"
    }

    fn layers(&self) -> &[LayerId] {
        // L9 (Geopolítico), LA (Cosmopolítico)
        &[9, 10]
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn is_ready(&self) -> bool {
        true
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Implementação de Governor
// ═══════════════════════════════════════════════════════════════════════════════

impl Governor for Governance {
    type ProposalId = ProposalId;
    type Proposal = ProposalData;

    fn propose(&mut self, proposal: Self::Proposal) -> Result<Self::ProposalId, GovernanceError> {
        let id = ProposalId(self.next_id);
        self.next_id += 1;

        // Inicializa votação
        self.proposals
            .insert(id, (proposal, ProposalState::Voting));
        self.votes
            .insert(id, VotingRecord::new(id, self.config.voting_duration_secs));

        Ok(id)
    }

    fn vote(&mut self, id: &Self::ProposalId, vote: Vote) -> Result<(), GovernanceError> {
        // Verifica proposta existe
        if !self.proposals.contains_key(id) {
            return Err(GovernanceError::Rejected("Proposal not found".into()));
        }

        // Verifica ainda em votação
        if let Some((_, state)) = self.proposals.get(id) {
            if !matches!(state, ProposalState::Voting) {
                return Err(GovernanceError::Rejected(
                    "Voting period ended".into(),
                ));
            }
        }

        // Registra voto
        if let Some(voting) = self.votes.get_mut(id) {
            voting.vote(&self.config.node_id, vote)
                .map_err(|e| GovernanceError::Rejected(e))?;
        }

        // Atualiza estado da proposta
        self.update_proposal_state(*id);

        Ok(())
    }

    fn status(&self, id: &Self::ProposalId) -> Option<ProposalStatus> {
        self.proposals.get(id).map(|(_, state)| match state {
            ProposalState::Voting => ProposalStatus::Pending,
            ProposalState::Approved => ProposalStatus::Approved,
            ProposalState::Rejected => ProposalStatus::Rejected,
            ProposalState::Executed => ProposalStatus::Approved, // ou outro estado
            ProposalState::Expired => ProposalStatus::Expired,
        })
    }

    fn active_proposals(&self) -> Vec<Self::ProposalId> {
        self.active_proposals()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn governance_creation() {
        let gov = Governance::new();
        assert!(gov.is_ok());
    }

    #[test]
    fn governance_propose() {
        let mut gov = Governance::new().unwrap();
        let proposal = ProposalData::new("Test proposal");
        let id = gov.propose(proposal).unwrap();
        assert_eq!(id.0, 1);
    }

    #[test]
    fn governance_vote() {
        let mut gov = Governance::new().unwrap();
        let proposal = ProposalData::new("Test proposal");
        let id = gov.propose(proposal).unwrap();

        let result = gov.vote(&id, Vote::Yes);
        assert!(result.is_ok());
    }

    #[test]
    fn governance_implements_sil_component() {
        let gov = Governance::new().unwrap();
        assert_eq!(gov.name(), "governance");
        assert!(gov.layers().contains(&9));
        assert!(gov.is_ready());
    }
}
