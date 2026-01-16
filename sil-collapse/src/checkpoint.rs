//! Checkpoint storage e gerenciamento

use serde::{Deserialize, Serialize};
use sil_core::prelude::*;
use std::collections::VecDeque;

/// ID de checkpoint
pub type CheckpointId = u64;

/// Checkpoint de estado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    /// ID único
    pub id: CheckpointId,
    /// Estado salvo
    pub state: SilState,
    /// Timestamp de criação
    pub timestamp: u64,
    /// Descrição opcional
    pub description: Option<String>,
}

impl Checkpoint {
    /// Cria novo checkpoint
    pub fn new(id: CheckpointId, state: SilState) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            id,
            state,
            timestamp,
            description: None,
        }
    }

    /// Cria checkpoint com descrição
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Idade do checkpoint em segundos
    pub fn age(&self) -> u64 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        now.saturating_sub(self.timestamp)
    }
}

/// Storage de checkpoints
#[derive(Debug, Clone)]
pub struct CheckpointStorage {
    /// Checkpoints armazenados (VecDeque para O(1) pop_front)
    checkpoints: VecDeque<Checkpoint>,
    /// Próximo ID
    next_id: CheckpointId,
    /// Limite máximo de checkpoints
    max_checkpoints: usize,
}

impl CheckpointStorage {
    /// Cria novo storage
    pub fn new(max_checkpoints: usize) -> Self {
        Self {
            checkpoints: VecDeque::new(),
            next_id: 1,
            max_checkpoints,
        }
    }

    /// Adiciona checkpoint
    pub fn add(&mut self, state: SilState, description: Option<String>) -> CheckpointId {
        let id = self.next_id;
        self.next_id += 1;

        let mut checkpoint = Checkpoint::new(id, state);
        if let Some(desc) = description {
            checkpoint = checkpoint.with_description(desc);
        }

        self.checkpoints.push_back(checkpoint);

        // Remove checkpoints antigos se exceder limite (O(1) com VecDeque)
        while self.checkpoints.len() > self.max_checkpoints {
            self.checkpoints.pop_front();
        }

        id
    }

    /// Obtém checkpoint
    pub fn get(&self, id: CheckpointId) -> Option<&Checkpoint> {
        self.checkpoints.iter().find(|c| c.id == id)
    }

    /// Remove checkpoint
    pub fn remove(&mut self, id: CheckpointId) -> bool {
        if let Some(pos) = self.checkpoints.iter().position(|c| c.id == id) {
            self.checkpoints.remove(pos);
            true
        } else {
            false
        }
    }

    /// Lista todos os IDs
    pub fn list(&self) -> Vec<CheckpointId> {
        self.checkpoints.iter().map(|c| c.id).collect()
    }

    /// Número de checkpoints
    pub fn count(&self) -> usize {
        self.checkpoints.len()
    }

    /// Limpa todos os checkpoints
    pub fn clear(&mut self) {
        self.checkpoints.clear();
    }

    /// Checkpoint mais recente
    pub fn latest(&self) -> Option<&Checkpoint> {
        self.checkpoints.back()
    }

    /// Checkpoint mais antigo
    pub fn oldest(&self) -> Option<&Checkpoint> {
        self.checkpoints.front()
    }
}

impl Default for CheckpointStorage {
    fn default() -> Self {
        Self::new(10)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkpoint_creation() {
        let state = SilState::neutral().with_layer(0, ByteSil { rho: 3, theta: 4 });
        let checkpoint = Checkpoint::new(1, state);
        assert_eq!(checkpoint.id, 1);
    }

    #[test]
    fn test_storage_add() {
        let mut storage = CheckpointStorage::new(5);
        let state = SilState::neutral();
        let id = storage.add(state, None);
        assert_eq!(storage.count(), 1);
        assert!(storage.get(id).is_some());
    }

    #[test]
    fn test_storage_limit() {
        let mut storage = CheckpointStorage::new(3);
        for i in 0..5 {
            let state = SilState::neutral().with_layer(0, ByteSil { rho: i, theta: 0 });
            storage.add(state, None);
        }
        assert_eq!(storage.count(), 3);
    }
}
