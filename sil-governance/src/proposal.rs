//! Propostas de governança

use serde::{Deserialize, Serialize};

/// ID único de proposta
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProposalId(pub u64);

impl std::fmt::Display for ProposalId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Prop({})", self.0)
    }
}

/// Estados de uma proposta
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProposalState {
    /// Em votação
    Voting,
    /// Aprovada
    Approved,
    /// Rejeitada
    Rejected,
    /// Executada
    Executed,
    /// Expirada (timeout)
    Expired,
}

impl std::fmt::Display for ProposalState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProposalState::Voting => write!(f, "Voting"),
            ProposalState::Approved => write!(f, "Approved"),
            ProposalState::Rejected => write!(f, "Rejected"),
            ProposalState::Executed => write!(f, "Executed"),
            ProposalState::Expired => write!(f, "Expired"),
        }
    }
}

/// Dados de uma proposta
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalData {
    /// Descrição/título
    pub description: String,
    /// Payload customizado (JSON ou binário)
    pub payload: Vec<u8>,
    /// Criador da proposta
    pub proposer: Option<String>,
    /// Timestamp de criação
    pub created_at: u64,
    /// Quórum necessário para aprovação (0.0-1.0)
    pub required_quorum: f64,
    /// Timestamp de expiração
    pub expires_at: std::time::SystemTime,
}

impl ProposalData {
    /// Cria nova proposta
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            payload: Vec::new(),
            proposer: None,
            created_at: now_secs(),
            required_quorum: 0.5, // Default: maioria simples
            expires_at: std::time::SystemTime::now() + std::time::Duration::from_secs(3600), // Default: 1 hora
        }
    }

    /// Com payload customizado
    pub fn with_payload(mut self, payload: Vec<u8>) -> Self {
        self.payload = payload;
        self
    }

    /// Com informação de proposer
    pub fn with_proposer(mut self, proposer: String) -> Self {
        self.proposer = Some(proposer);
        self
    }

    /// Alias para new() - compatibilidade com CLI
    pub fn from_data(data: ProposalData) -> Self {
        data
    }
}

impl std::fmt::Display for ProposalData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Proposal: {} (by {})",
            self.description,
            self.proposer.as_deref().unwrap_or("unknown")
        )
    }
}

/// Timestamp em segundos
fn now_secs() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proposal_id_display() {
        let id = ProposalId(42);
        assert_eq!(id.to_string(), "Prop(42)");
    }

    #[test]
    fn proposal_state_display() {
        assert_eq!(ProposalState::Voting.to_string(), "Voting");
        assert_eq!(ProposalState::Approved.to_string(), "Approved");
    }

    #[test]
    fn proposal_data_creation() {
        let p = ProposalData::new("Test proposal");
        assert_eq!(p.description, "Test proposal");
        assert!(p.created_at > 0);
    }

    #[test]
    fn proposal_with_payload() {
        let p = ProposalData::new("Test")
            .with_payload(vec![1, 2, 3]);
        assert_eq!(p.payload, vec![1, 2, 3]);
    }
}
