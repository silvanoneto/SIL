//! Peer registry e identificadores
//!
//! PeerId é um hash de 64 bits derivado da chave pública Ed25519.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::fmt;
use std::time::{Instant, Duration};
use serde::{Deserialize, Serialize};

/// Identificador único de peer (hash de 64 bits da chave pública)
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PeerId(pub u64);

impl PeerId {
    /// Cria novo PeerId a partir de bytes de chave pública
    pub fn from_public_key(pubkey: &[u8; 32]) -> Self {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(pubkey);
        let hash = hasher.finalize();
        let id = u64::from_le_bytes(hash[0..8].try_into().unwrap());
        Self(id)
    }

    /// Cria PeerId para testes/desenvolvimento
    pub fn random() -> Self {
        Self(rand::random())
    }

    /// Valor numérico interno
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl fmt::Debug for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Peer({:016x})", self.0)
    }
}

impl fmt::Display for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

/// Estado de conexão com peer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeerState {
    /// Descoberto mas não conectado
    Discovered,
    /// Conectando (handshake em progresso)
    Connecting,
    /// Conectado e ativo
    Connected,
    /// Desconectado (pode reconectar)
    Disconnected,
}

/// Informações detalhadas de um peer
#[derive(Debug, Clone)]
pub struct PeerEntry {
    pub id: PeerId,
    pub addr: SocketAddr,
    pub state: PeerState,
    pub last_seen: Instant,
    pub latency_ms: Option<f32>,
    pub messages_sent: u64,
    pub messages_received: u64,
}

impl PeerEntry {
    pub fn new(id: PeerId, addr: SocketAddr) -> Self {
        Self {
            id,
            addr,
            state: PeerState::Discovered,
            last_seen: Instant::now(),
            latency_ms: None,
            messages_sent: 0,
            messages_received: 0,
        }
    }

    /// Peer está ativo (visto recentemente)?
    pub fn is_active(&self, timeout: Duration) -> bool {
        self.last_seen.elapsed() < timeout
    }

    /// Atualiza timestamp de última atividade
    pub fn touch(&mut self) {
        self.last_seen = Instant::now();
    }
}

/// Registry de peers conhecidos
#[derive(Debug)]
pub struct PeerRegistry {
    peers: HashMap<PeerId, PeerEntry>,
    timeout: Duration,
}

impl PeerRegistry {
    pub fn new() -> Self {
        Self {
            peers: HashMap::new(),
            timeout: Duration::from_secs(30),
        }
    }

    /// Define timeout para considerar peer inativo
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Adiciona ou atualiza peer
    pub fn upsert(&mut self, id: PeerId, addr: SocketAddr) -> &mut PeerEntry {
        self.peers
            .entry(id)
            .and_modify(|e| {
                e.addr = addr;
                e.touch();
            })
            .or_insert_with(|| PeerEntry::new(id, addr))
    }

    /// Obtém peer por ID
    pub fn get(&self, id: &PeerId) -> Option<&PeerEntry> {
        self.peers.get(id)
    }

    /// Obtém peer mutável
    pub fn get_mut(&mut self, id: &PeerId) -> Option<&mut PeerEntry> {
        self.peers.get_mut(id)
    }

    /// Remove peer
    pub fn remove(&mut self, id: &PeerId) -> Option<PeerEntry> {
        self.peers.remove(id)
    }

    /// Lista todos os peers ativos
    pub fn active_peers(&self) -> Vec<&PeerEntry> {
        self.peers
            .values()
            .filter(|p| p.is_active(self.timeout) && p.state == PeerState::Connected)
            .collect()
    }

    /// Lista todos os peers (incluindo inativos)
    pub fn all_peers(&self) -> Vec<&PeerEntry> {
        self.peers.values().collect()
    }

    /// Contagem de peers conectados
    pub fn connected_count(&self) -> usize {
        self.peers
            .values()
            .filter(|p| p.state == PeerState::Connected)
            .count()
    }

    /// Remove peers inativos
    pub fn prune_inactive(&mut self) -> usize {
        let before = self.peers.len();
        self.peers.retain(|_, p| p.is_active(self.timeout));
        before - self.peers.len()
    }

    /// Endereços de todos os peers conectados
    pub fn connected_addrs(&self) -> Vec<SocketAddr> {
        self.peers
            .values()
            .filter(|p| p.state == PeerState::Connected)
            .map(|p| p.addr)
            .collect()
    }
}

impl Default for PeerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    fn test_addr(port: u16) -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port)
    }

    #[test]
    fn peer_id_from_public_key() {
        let key = [0u8; 32];
        let id = PeerId::from_public_key(&key);
        // Determinístico
        let id2 = PeerId::from_public_key(&key);
        assert_eq!(id, id2);
    }

    #[test]
    fn registry_upsert_and_get() {
        let mut reg = PeerRegistry::new();
        let id = PeerId::random();
        let addr = test_addr(8000);

        reg.upsert(id, addr);
        assert!(reg.get(&id).is_some());
        assert_eq!(reg.get(&id).unwrap().addr, addr);
    }

    #[test]
    fn registry_prune_inactive() {
        let mut reg = PeerRegistry::new().with_timeout(Duration::from_millis(1));
        let id = PeerId::random();
        reg.upsert(id, test_addr(8000));

        std::thread::sleep(Duration::from_millis(10));
        let pruned = reg.prune_inactive();
        assert_eq!(pruned, 1);
        assert!(reg.get(&id).is_none());
    }
}
