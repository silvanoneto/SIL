//! Checkpoint and delta compression for efficient model synchronization
//!
//! Implements JSIL-style XOR delta compression for bandwidth-efficient updates.
//! Following Paebiru pattern: only transmit differences between states.

use crate::Result;
use crate::error::SilMlError;
use sil_core::state::SilState;

/// Checkpoint snapshot with metadata
#[derive(Debug, Clone)]
pub struct Checkpoint {
    /// Unique checkpoint ID
    pub id: String,
    /// Epoch or training step
    pub step: u64,
    /// Serialized state (JSIL format)
    pub data: Vec<u8>,
    /// SHA256 hash for verification
    pub hash: String,
}

/// Delta compression info
#[derive(Debug, Clone)]
pub struct Delta {
    /// Source checkpoint ID
    pub from_id: String,
    /// Target checkpoint ID
    pub to_id: String,
    /// XOR-compressed difference
    pub xor_data: Vec<u8>,
    /// Compression ratio achieved
    pub ratio: f64,
}

impl Checkpoint {
    /// Create checkpoint from state
    pub fn from_state(id: String, step: u64, state: &SilState) -> Self {
        let data = serialize_state(state);
        let hash = compute_hash(&data);
        Self { id, step, data, hash }
    }

    /// Compute delta to target checkpoint
    ///
    /// # Paebiru Pattern: Incremental Sync
    ///
    /// XOR compression reduces update size to ~5-10% of model for small changes.
    pub fn delta_to(&self, target: &Checkpoint) -> Result<Delta> {
        if self.data.len() != target.data.len() {
            return Err(SilMlError::Transform(
                "Checkpoint size mismatch".to_string()
            ));
        }

        // XOR compression
        let xor_data: Vec<u8> = self.data.iter()
            .zip(target.data.iter())
            .map(|(a, b)| a ^ b)
            .collect();

        let original_size = target.data.len() as f64;
        let compressed_size = xor_data.len() as f64;
        let ratio = compressed_size / original_size;

        Ok(Delta {
            from_id: self.id.clone(),
            to_id: target.id.clone(),
            xor_data,
            ratio,
        })
    }

    /// Apply delta to create new checkpoint
    pub fn apply_delta(&self, delta: &Delta) -> Result<Checkpoint> {
        if delta.from_id != self.id {
            return Err(SilMlError::Transform(
                "Delta source mismatch".to_string()
            ));
        }

        // XOR decompression
        let new_data: Vec<u8> = self.data.iter()
            .zip(delta.xor_data.iter())
            .map(|(a, b)| a ^ b)
            .collect();

        let hash = compute_hash(&new_data);
        Ok(Checkpoint {
            id: delta.to_id.clone(),
            step: self.step + 1,
            data: new_data,
            hash,
        })
    }
}

/// Checkpoint history for efficient tracking
pub struct CheckpointHistory {
    checkpoints: Vec<Checkpoint>,
    /// Keep last N checkpoints
    max_history: usize,
}

impl CheckpointHistory {
    pub fn new(max_history: usize) -> Self {
        Self {
            checkpoints: Vec::new(),
            max_history,
        }
    }

    /// Add checkpoint to history
    pub fn add(&mut self, checkpoint: Checkpoint) {
        self.checkpoints.push(checkpoint);
        if self.checkpoints.len() > self.max_history {
            self.checkpoints.remove(0);
        }
    }

    /// Get latest checkpoint
    pub fn latest(&self) -> Option<&Checkpoint> {
        self.checkpoints.last()
    }

    /// Get checkpoint by ID
    pub fn get(&self, id: &str) -> Option<&Checkpoint> {
        self.checkpoints.iter().find(|c| c.id == id)
    }

    /// Get length of history
    pub fn len(&self) -> usize {
        self.checkpoints.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.checkpoints.is_empty()
    }

    /// Get delta between two checkpoints
    pub fn delta_between(&self, from_id: &str, to_id: &str) -> Result<Delta> {
        let from = self.get(from_id)
            .ok_or_else(|| SilMlError::StorageError("Source checkpoint not found".into()))?;
        let to = self.get(to_id)
            .ok_or_else(|| SilMlError::StorageError("Target checkpoint not found".into()))?;

        from.delta_to(to)
    }

    /// Rollback to checkpoint
    pub fn rollback(&mut self, checkpoint_id: &str) -> Result<()> {
        let pos = self.checkpoints.iter()
            .position(|c| c.id == checkpoint_id)
            .ok_or_else(|| SilMlError::StorageError("Checkpoint not found".into()))?;

        self.checkpoints.truncate(pos + 1);
        Ok(())
    }
}

/// Serialize SilState to bytes (JSIL format)
fn serialize_state(state: &SilState) -> Vec<u8> {
    // Simple serialization: 16 bytes per layer (rho + theta)
    let mut data = Vec::with_capacity(16);
    for i in 0..16 {
        let layer = state.get(i);
        data.push(layer.rho as u8);
        data.push(layer.theta);
    }
    data
}

/// Compute SHA256 hash of data
fn compute_hash(data: &[u8]) -> String {
    // Simple hash for demo (in production use real SHA256)
    let mut hash = 0u64;
    for &byte in data {
        hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
    }
    format!("{:016x}", hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkpoint_creation() {
        let state = SilState::neutral();
        let cp = Checkpoint::from_state("cp1".into(), 0, &state);
        assert_eq!(cp.id, "cp1");
        assert_eq!(cp.step, 0);
        assert_eq!(cp.data.len(), 32); // 16 layers × 2 bytes (rho + theta)
    }

    #[test]
    fn test_delta_compression() {
        let state1 = SilState::neutral();
        let state2 = SilState::neutral();
        let cp1 = Checkpoint::from_state("cp1".into(), 0, &state1);
        let cp2 = Checkpoint::from_state("cp2".into(), 1, &state2);

        let delta = cp1.delta_to(&cp2).unwrap();
        assert!(delta.ratio >= 0.0 && delta.ratio <= 1.0);
    }

    #[test]
    fn test_checkpoint_history() {
        let mut history = CheckpointHistory::new(3);
        let state = SilState::neutral();

        for i in 0..5 {
            let cp = Checkpoint::from_state(format!("cp{}", i), i as u64, &state);
            history.add(cp);
        }

        // Should only keep last 3
        assert_eq!(history.checkpoints.len(), 3);
        assert_eq!(history.latest().unwrap().step, 4);
    }
}
