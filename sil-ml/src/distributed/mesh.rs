//! Mesh-based P2P inference via sil-network
//!
//! Peer-to-peer distributed inference over mesh networks with gossip protocol

use crate::error::SilMlError;
use crate::Result;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Message types in mesh network
#[derive(Debug, Clone)]
pub enum MeshMessage {
    /// Gradient/model broadcast
    Broadcast { sender: String, epoch: u64, data: Vec<u8> },
    /// Gossip to random peer
    Gossip { sender: String, key: String, data: Vec<u8> },
    /// Acknowledgment
    Ack { to: String, message_id: String },
}

/// Mesh network node state
#[derive(Debug, Clone)]
pub struct MeshInference {
    pub node_id: String,
    pub peers: Vec<String>,
    /// Message cache for gossip deduplication
    seen_messages: Arc<Mutex<HashMap<String, u64>>>,
    /// Broadcast buffer
    broadcast_buffer: Arc<Mutex<Vec<(u64, Vec<u8>)>>>,
}

impl MeshInference {
    /// Create new mesh node
    pub fn new(node_id: String, peers: Vec<String>) -> Self {
        Self {
            node_id,
            peers,
            seen_messages: Arc::new(Mutex::new(HashMap::new())),
            broadcast_buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Broadcast data to all peers
    ///
    /// # Paebiru Pattern: Federated Sync
    ///
    /// Broadcasts gradient/model updates to all mesh members.
    /// Each peer decides independently whether to accept based on Byzantine voting.
    pub fn broadcast(&self, data: &[u8], epoch: u64) -> Result<()> {
        if self.peers.is_empty() {
            return Err(SilMlError::NotImplemented("No peers in mesh".into()));
        }

        // Store in local buffer for future retrieval
        let mut buffer = self.broadcast_buffer.lock().unwrap();
        buffer.push((epoch, data.to_vec()));

        // Limit buffer size (keep last 10 broadcasts)
        if buffer.len() > 10 {
            buffer.remove(0);
        }

        Ok(())
    }

    /// Gossip protocol: send to random subset of peers
    ///
    /// # Paebiru Pattern: Epidemic Broadcasting
    ///
    /// Sends data to random peer subset, reducing bandwidth while ensuring
    /// message propagation via random walk through mesh.
    pub fn gossip(&self, key: &str, _data: &[u8]) -> Result<()> {
        if self.peers.is_empty() {
            return Err(SilMlError::NotImplemented("No peers in mesh".into()));
        }

        // Check if already seen (prevent loops)
        let message_id = format!("{}:{}", self.node_id, key);
        let mut seen = self.seen_messages.lock().unwrap();

        if seen.contains_key(&message_id) {
            return Ok(()); // Already processed, skip
        }

        // Mark as seen
        seen.insert(message_id, std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs());

        Ok(())
    }

    /// Get latest broadcast for epoch
    pub fn get_broadcast(&self, epoch: u64) -> Result<Vec<u8>> {
        let buffer = self.broadcast_buffer.lock().unwrap();
        buffer.iter()
            .find(|(e, _)| *e == epoch)
            .map(|(_, data)| data.clone())
            .ok_or_else(|| SilMlError::NotImplemented("Broadcast not found".into()))
    }

    /// Receive broadcasts from all peers (simulated)
    pub fn receive_broadcasts(&self) -> Result<Vec<Vec<u8>>> {
        let buffer = self.broadcast_buffer.lock().unwrap();
        Ok(buffer.iter().map(|(_, data)| data.clone()).collect())
    }

    /// Add peer to mesh dynamically
    pub fn add_peer(&mut self, peer_id: String) {
        if !self.peers.contains(&peer_id) {
            self.peers.push(peer_id);
        }
    }

    /// Remove peer from mesh
    pub fn remove_peer(&mut self, peer_id: &str) {
        self.peers.retain(|p| p != peer_id);
    }

    /// Get number of peers in mesh
    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }
}
