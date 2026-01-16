//! Sistema de votação

use crate::proposal::ProposalId;
use std::collections::HashMap;
use std::time::{Instant, Duration};
use sil_core::traits::Vote;

/// Contagem de votos
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VoteCount {
    /// Votos SIM
    pub yes: u64,
    /// Votos NÃO
    pub no: u64,
    /// Votos ABSTENÇÃO
    pub abstain: u64,
}

impl VoteCount {
    pub fn new() -> Self {
        Self {
            yes: 0,
            no: 0,
            abstain: 0,
        }
    }

    pub fn total(&self) -> u64 {
        self.yes + self.no + self.abstain
    }

    pub fn register(&mut self, vote: Vote) {
        match vote {
            Vote::Yes => self.yes += 1,
            Vote::No => self.no += 1,
            Vote::Abstain => self.abstain += 1,
        }
    }

    /// Percentual de SIM entre votos válidos (não-abstenção)
    pub fn approval_ratio(&self) -> f32 {
        let valid = self.yes + self.no;
        if valid == 0 {
            0.0
        } else {
            self.yes as f32 / valid as f32
        }
    }

    /// Percentual de participação (votos/total_elegíveis)
    pub fn participation_ratio(&self, eligible: u64) -> f32 {
        if eligible == 0 {
            0.0
        } else {
            self.total() as f32 / eligible as f32
        }
    }
}

impl Default for VoteCount {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for VoteCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Votes: Yes={}, No={}, Abstain={} (Ratio: {:.1}%)",
            self.yes,
            self.no,
            self.abstain,
            self.approval_ratio() * 100.0
        )
    }
}

/// Registro de votação de uma proposta
#[derive(Debug)]
pub struct VotingRecord {
    /// ID da proposta
    pub proposal_id: ProposalId,
    /// Votos por nó
    voters: HashMap<String, Vote>,
    /// Contagem
    pub count: VoteCount,
    /// Timestamp de início
    started_at: Instant,
    /// Duração máxima
    duration: Duration,
}

impl VotingRecord {
    /// Cria novo registro
    pub fn new(proposal_id: ProposalId, duration_secs: u64) -> Self {
        Self {
            proposal_id,
            voters: HashMap::new(),
            count: VoteCount::new(),
            started_at: Instant::now(),
            duration: Duration::from_secs(duration_secs),
        }
    }

    /// Registra voto de um nó
    pub fn vote(&mut self, voter_id: &str, vote: Vote) -> Result<(), String> {
        // Verifica timeout
        if self.started_at.elapsed() > self.duration {
            return Err("Voting period expired".into());
        }

        // Verifica se já votou
        if self.voters.contains_key(voter_id) {
            return Err("Already voted".into());
        }

        // Registra
        self.voters.insert(voter_id.to_string(), vote);
        self.count.register(vote);

        Ok(())
    }

    /// Quem votou
    pub fn voters(&self) -> impl Iterator<Item = (&str, Vote)> {
        self.voters
            .iter()
            .map(|(k, v)| (k.as_str(), *v))
    }

    /// Votação expirou?
    pub fn is_expired(&self) -> bool {
        self.started_at.elapsed() > self.duration
    }

    /// Tempo restante
    pub fn time_remaining(&self) -> Option<Duration> {
        let elapsed = self.started_at.elapsed();
        if elapsed >= self.duration {
            None
        } else {
            Some(self.duration - elapsed)
        }
    }

    /// Total de votantes
    pub fn voter_count(&self) -> usize {
        self.voters.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vote_count_creation() {
        let vc = VoteCount::new();
        assert_eq!(vc.yes, 0);
        assert_eq!(vc.no, 0);
        assert_eq!(vc.abstain, 0);
        assert_eq!(vc.total(), 0);
    }

    #[test]
    fn vote_count_register() {
        let mut vc = VoteCount::new();
        vc.register(Vote::Yes);
        vc.register(Vote::Yes);
        vc.register(Vote::No);
        vc.register(Vote::Abstain);

        assert_eq!(vc.yes, 2);
        assert_eq!(vc.no, 1);
        assert_eq!(vc.abstain, 1);
        assert_eq!(vc.total(), 4);
    }

    #[test]
    fn vote_count_approval_ratio() {
        let mut vc = VoteCount::new();
        vc.yes = 3;
        vc.no = 1;
        vc.abstain = 1;

        assert_eq!(vc.approval_ratio(), 0.75); // 3 de 4
    }

    #[test]
    fn vote_count_participation_ratio() {
        let vc = VoteCount {
            yes: 2,
            no: 1,
            abstain: 1,
        };

        assert_eq!(vc.participation_ratio(4), 1.0); // 4 de 4
        assert_eq!(vc.participation_ratio(10), 0.4); // 4 de 10
    }

    #[test]
    fn voting_record_creation() {
        let id = ProposalId(1);
        let record = VotingRecord::new(id, 300);
        assert_eq!(record.voter_count(), 0);
        assert_eq!(record.count.total(), 0);
    }

    #[test]
    fn voting_record_vote() {
        let id = ProposalId(1);
        let mut record = VotingRecord::new(id, 300);

        assert!(record.vote("node-1", Vote::Yes).is_ok());
        assert_eq!(record.voter_count(), 1);
        assert_eq!(record.count.yes, 1);

        // Não permite revoto
        assert!(record.vote("node-1", Vote::No).is_err());
    }

    #[test]
    fn voting_record_expiration() {
        let id = ProposalId(1);
        let mut record = VotingRecord::new(id, 1);

        // Voto imediato OK
        assert!(record.vote("node-1", Vote::Yes).is_ok());

        // Aguarda expiração
        std::thread::sleep(Duration::from_millis(1100));

        // Novo voto falha
        assert!(record.vote("node-2", Vote::Yes).is_err());
    }
}
