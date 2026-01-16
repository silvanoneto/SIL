//! Sincronização Distribuída via Entanglement
//!
//! Implementa o protocolo de entanglement para sincronização de estados
//! SIL entre nós distribuídos usando a camada L(E) - Entanglement.
//!
//! # Conceito
//!
//! Dois estados SIL "entangled" mantêm correlação mesmo em nós remotos:
//! - Mudança em um nó propaga instantaneamente para o par
//! - Operações XOR preservam entanglement
//! - Colapso quebra o entanglement
//!
//! # Protocolo
//!
//! ```text
//! ┌────────────────┐                    ┌────────────────┐
//! │     Nó A       │                    │     Nó B       │
//! │                │    ENTANGLE        │                │
//! │  State_A  ─────┼───────────────────►│  State_B       │
//! │    │           │                    │    │           │
//! │    │ XOR       │                    │    │ XOR       │
//! │    ▼           │    SYNC            │    ▼           │
//! │  State_A' ─────┼───────────────────►│  State_B'      │
//! │                │                    │                │
//! └────────────────┘                    └────────────────┘
//! ```

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::state::{ByteSil, SilState};

// Contador global para garantir IDs únicos
static NODE_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

// ═══════════════════════════════════════════════════════════════════════════════
// NODE IDENTITY
// ═══════════════════════════════════════════════════════════════════════════════

/// ID único de nó
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub u64);

impl NodeId {
    pub fn new() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let counter = NODE_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self((t as u64) ^ (std::process::id() as u64) ^ (counter << 32))
    }
    
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut arr = [0u8; 8];
        arr.copy_from_slice(&bytes[..8.min(bytes.len())]);
        Self(u64::from_le_bytes(arr))
    }
    
    pub fn to_bytes(&self) -> [u8; 8] {
        self.0.to_le_bytes()
    }
}

impl Default for NodeId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:016X}", self.0)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ENTANGLEMENT PAIR
// ═══════════════════════════════════════════════════════════════════════════════

/// ID de par entangled
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PairId(pub u64);

impl PairId {
    pub fn new() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        Self((t as u64) ^ rand_u64())
    }
}

impl Default for PairId {
    fn default() -> Self {
        Self::new()
    }
}

fn rand_u64() -> u64 {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    RandomState::new().build_hasher().finish()
}

/// Estado do entanglement
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntanglementState {
    /// Ativo - correlação mantida
    Active,
    /// Suspenso - temporariamente sem sync
    Suspended,
    /// Quebrado - colapso ou timeout
    Broken,
}

/// Par entangled
#[derive(Debug, Clone)]
pub struct EntangledPair {
    /// ID do par
    pub id: PairId,
    /// Nó local
    pub local_node: NodeId,
    /// Nó remoto
    pub remote_node: NodeId,
    /// Estado local
    pub local_state: SilState,
    /// Estado correlacionado
    pub correlation: SilState,
    /// Estado do entanglement
    pub state: EntanglementState,
    /// Última sincronização
    pub last_sync: Instant,
    /// Contador de versão
    pub version: u64,
    /// Camada de entanglement (L(E) = camada 14)
    pub layer: u8,
}

impl EntangledPair {
    pub fn new(local: NodeId, remote: NodeId, initial_state: SilState) -> Self {
        Self {
            id: PairId::new(),
            local_node: local,
            remote_node: remote,
            local_state: initial_state.clone(),
            correlation: initial_state,
            state: EntanglementState::Active,
            last_sync: Instant::now(),
            version: 0,
            layer: 14, // L(E)
        }
    }
    
    /// Aplica operação XOR mantendo correlação
    pub fn xor(&mut self, other: &SilState) {
        // XOR preserva entanglement
        for i in 0..16 {
            let local = ByteSil::from(self.local_state.layers[i]);
            let other_byte = ByteSil::from(other.layers[i]);
            let result = local ^ other_byte;
            
            // Update correlation
            let corr = ByteSil::from(self.correlation.layers[i]);
            let new_corr = corr ^ other_byte;
            
            self.local_state.set_layer(i, result.into());
            self.correlation.set_layer(i, new_corr.into());
        }
        
        self.version += 1;
        self.last_sync = Instant::now();
    }
    
    /// Colapso - quebra o entanglement
    pub fn collapse(&mut self) -> SilState {
        self.state = EntanglementState::Broken;
        
        // Collapse via XOR of local and correlation
        let mut result = SilState::neutral();
        for i in 0..16 {
            let local = ByteSil::from(self.local_state.layers[i]);
            let corr = ByteSil::from(self.correlation.layers[i]);
            result.set_layer(i, (local ^ corr).into());
        }
        
        result
    }
    
    /// Verifica se está sincronizado
    pub fn is_synced(&self) -> bool {
        self.state == EntanglementState::Active
    }
    
    /// Verifica timeout
    pub fn check_timeout(&mut self, timeout: Duration) -> bool {
        if self.last_sync.elapsed() > timeout {
            self.state = EntanglementState::Broken;
            true
        } else {
            false
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SYNC MESSAGE
// ═══════════════════════════════════════════════════════════════════════════════

/// Tipo de mensagem de sync
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncMessageType {
    /// Pedido de entanglement
    EntangleRequest,
    /// Resposta de entanglement
    EntangleResponse,
    /// Sincronização de estado
    StateSync,
    /// Confirmação de sync
    SyncAck,
    /// Notificação de colapso
    Collapse,
    /// Heartbeat
    Heartbeat,
    /// Desconexão
    Disconnect,
}

/// Mensagem de sincronização
#[derive(Debug, Clone)]
pub struct SyncMessage {
    /// Tipo
    pub msg_type: SyncMessageType,
    /// Origem
    pub from: NodeId,
    /// Destino
    pub to: NodeId,
    /// ID do par
    pub pair_id: PairId,
    /// Versão
    pub version: u64,
    /// Payload (estado serializado)
    pub payload: Vec<u8>,
    /// Timestamp
    pub timestamp: u64,
}

impl SyncMessage {
    pub fn new(msg_type: SyncMessageType, from: NodeId, to: NodeId, pair_id: PairId) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        Self {
            msg_type,
            from,
            to,
            pair_id,
            version: 0,
            payload: Vec::new(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        }
    }
    
    pub fn with_state(mut self, state: &SilState) -> Self {
        self.payload = state_to_bytes(state);
        self
    }
    
    pub fn with_version(mut self, version: u64) -> Self {
        self.version = version;
        self
    }
    
    /// Serializa para bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        // Header
        bytes.push(self.msg_type as u8);
        bytes.extend_from_slice(&self.from.to_bytes());
        bytes.extend_from_slice(&self.to.to_bytes());
        bytes.extend_from_slice(&self.pair_id.0.to_le_bytes());
        bytes.extend_from_slice(&self.version.to_le_bytes());
        bytes.extend_from_slice(&self.timestamp.to_le_bytes());
        bytes.extend_from_slice(&(self.payload.len() as u32).to_le_bytes());
        
        // Payload
        bytes.extend_from_slice(&self.payload);
        
        bytes
    }
    
    /// Deserializa de bytes
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 45 {
            return None;
        }
        
        let msg_type = match bytes[0] {
            0 => SyncMessageType::EntangleRequest,
            1 => SyncMessageType::EntangleResponse,
            2 => SyncMessageType::StateSync,
            3 => SyncMessageType::SyncAck,
            4 => SyncMessageType::Collapse,
            5 => SyncMessageType::Heartbeat,
            6 => SyncMessageType::Disconnect,
            _ => return None,
        };
        
        let from = NodeId::from_bytes(&bytes[1..9]);
        let to = NodeId::from_bytes(&bytes[9..17]);
        let pair_id = PairId(u64::from_le_bytes(bytes[17..25].try_into().ok()?));
        let version = u64::from_le_bytes(bytes[25..33].try_into().ok()?);
        let timestamp = u64::from_le_bytes(bytes[33..41].try_into().ok()?);
        let payload_len = u32::from_le_bytes(bytes[41..45].try_into().ok()?) as usize;
        
        let payload = if payload_len > 0 && bytes.len() >= 45 + payload_len {
            bytes[45..45 + payload_len].to_vec()
        } else {
            Vec::new()
        };
        
        Some(Self {
            msg_type,
            from,
            to,
            pair_id,
            version,
            payload,
            timestamp,
        })
    }
}

fn state_to_bytes(state: &SilState) -> Vec<u8> {
    state.layers.iter().map(|b| u8::from(*b)).collect()
}

fn bytes_to_state(bytes: &[u8]) -> Option<SilState> {
    if bytes.len() != 16 {
        return None;
    }
    
    let mut layers = [ByteSil::NULL; 16];
    for (i, &b) in bytes.iter().enumerate() {
        layers[i] = ByteSil::from_u8(b);
    }
    Some(SilState::from_layers(layers))
}

// ═══════════════════════════════════════════════════════════════════════════════
// ENTANGLEMENT MANAGER
// ═══════════════════════════════════════════════════════════════════════════════

/// Callback para envio de mensagem
pub type SendCallback = Box<dyn Fn(&SyncMessage) -> bool + Send + Sync>;

/// Gerenciador de entanglements
pub struct EntanglementManager {
    /// ID deste nó
    node_id: NodeId,
    /// Pares entangled
    pairs: HashMap<PairId, EntangledPair>,
    /// Pares por nó remoto
    pairs_by_node: HashMap<NodeId, Vec<PairId>>,
    /// Timeout para sync
    sync_timeout: Duration,
    /// Callback de envio
    send_callback: Option<SendCallback>,
    /// Mensagens pendentes
    pending_messages: Vec<SyncMessage>,
}

impl EntanglementManager {
    pub fn new(node_id: NodeId) -> Self {
        Self {
            node_id,
            pairs: HashMap::new(),
            pairs_by_node: HashMap::new(),
            sync_timeout: Duration::from_secs(30),
            send_callback: None,
            pending_messages: Vec::new(),
        }
    }
    
    /// Define callback de envio
    pub fn on_send<F>(&mut self, callback: F)
    where
        F: Fn(&SyncMessage) -> bool + Send + Sync + 'static,
    {
        self.send_callback = Some(Box::new(callback));
    }
    
    /// Define timeout
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.sync_timeout = timeout;
    }
    
    /// Solicita entanglement com nó remoto
    pub fn request_entangle(&mut self, remote: NodeId, initial_state: SilState) -> PairId {
        let pair = EntangledPair::new(self.node_id, remote, initial_state.clone());
        let pair_id = pair.id;
        
        self.pairs.insert(pair_id, pair);
        self.pairs_by_node
            .entry(remote)
            .or_default()
            .push(pair_id);
        
        // Enviar request
        let msg = SyncMessage::new(
            SyncMessageType::EntangleRequest,
            self.node_id,
            remote,
            pair_id,
        ).with_state(&initial_state);
        
        self.send_message(&msg);
        
        pair_id
    }
    
    /// Processa mensagem recebida
    pub fn receive_message(&mut self, msg: &SyncMessage) -> Option<SyncMessage> {
        match msg.msg_type {
            SyncMessageType::EntangleRequest => {
                self.handle_entangle_request(msg)
            }
            SyncMessageType::EntangleResponse => {
                self.handle_entangle_response(msg)
            }
            SyncMessageType::StateSync => {
                self.handle_state_sync(msg)
            }
            SyncMessageType::SyncAck => {
                self.handle_sync_ack(msg);
                None
            }
            SyncMessageType::Collapse => {
                self.handle_collapse(msg);
                None
            }
            SyncMessageType::Heartbeat => {
                self.handle_heartbeat(msg)
            }
            SyncMessageType::Disconnect => {
                self.handle_disconnect(msg);
                None
            }
        }
    }
    
    fn handle_entangle_request(&mut self, msg: &SyncMessage) -> Option<SyncMessage> {
        // Criar par local
        let initial_state = bytes_to_state(&msg.payload)?;
        let pair = EntangledPair::new(self.node_id, msg.from, initial_state.clone());
        
        self.pairs.insert(msg.pair_id, pair);
        self.pairs_by_node
            .entry(msg.from)
            .or_default()
            .push(msg.pair_id);
        
        // Responder
        Some(SyncMessage::new(
            SyncMessageType::EntangleResponse,
            self.node_id,
            msg.from,
            msg.pair_id,
        ).with_state(&initial_state))
    }
    
    fn handle_entangle_response(&mut self, msg: &SyncMessage) -> Option<SyncMessage> {
        if let Some(pair) = self.pairs.get_mut(&msg.pair_id) {
            pair.state = EntanglementState::Active;
            pair.last_sync = Instant::now();
        }
        None
    }
    
    fn handle_state_sync(&mut self, msg: &SyncMessage) -> Option<SyncMessage> {
        let new_state = bytes_to_state(&msg.payload)?;
        
        if let Some(pair) = self.pairs.get_mut(&msg.pair_id) {
            if msg.version > pair.version {
                // Apply sync
                pair.correlation = new_state;
                pair.version = msg.version;
                pair.last_sync = Instant::now();
                
                // Ack
                return Some(SyncMessage::new(
                    SyncMessageType::SyncAck,
                    self.node_id,
                    msg.from,
                    msg.pair_id,
                ).with_version(msg.version));
            }
        }
        None
    }
    
    fn handle_sync_ack(&mut self, msg: &SyncMessage) {
        if let Some(pair) = self.pairs.get_mut(&msg.pair_id) {
            pair.last_sync = Instant::now();
        }
    }
    
    fn handle_collapse(&mut self, msg: &SyncMessage) {
        if let Some(pair) = self.pairs.get_mut(&msg.pair_id) {
            pair.state = EntanglementState::Broken;
        }
    }
    
    fn handle_heartbeat(&mut self, msg: &SyncMessage) -> Option<SyncMessage> {
        if let Some(pair) = self.pairs.get_mut(&msg.pair_id) {
            pair.last_sync = Instant::now();
        }
        None
    }
    
    fn handle_disconnect(&mut self, msg: &SyncMessage) {
        if let Some(pair) = self.pairs.get_mut(&msg.pair_id) {
            pair.state = EntanglementState::Broken;
        }
    }
    
    /// Sincroniza estado com par
    pub fn sync(&mut self, pair_id: PairId) -> bool {
        let pair = match self.pairs.get_mut(&pair_id) {
            Some(p) if p.state == EntanglementState::Active => p,
            _ => return false,
        };
        
        pair.version += 1;
        
        let msg = SyncMessage::new(
            SyncMessageType::StateSync,
            self.node_id,
            pair.remote_node,
            pair_id,
        )
        .with_state(&pair.correlation)
        .with_version(pair.version);
        
        self.send_message(&msg)
    }
    
    /// Aplica operação em par entangled
    pub fn apply_xor(&mut self, pair_id: PairId, other: &SilState) -> bool {
        if let Some(pair) = self.pairs.get_mut(&pair_id) {
            pair.xor(other);
            // Auto-sync - release borrow before sync
            let _ = pair;
            return self.sync(pair_id);
        }
        false
    }
    
    /// Colapsa par entangled
    pub fn collapse(&mut self, pair_id: PairId) -> Option<SilState> {
        let pair = self.pairs.get_mut(&pair_id)?;
        let result = pair.collapse();
        
        // Notificar remoto
        let msg = SyncMessage::new(
            SyncMessageType::Collapse,
            self.node_id,
            pair.remote_node,
            pair_id,
        );
        self.send_message(&msg);
        
        Some(result)
    }
    
    /// Verifica timeouts
    pub fn check_timeouts(&mut self) {
        for pair in self.pairs.values_mut() {
            pair.check_timeout(self.sync_timeout);
        }
    }
    
    /// Retorna par por ID
    pub fn get_pair(&self, pair_id: PairId) -> Option<&EntangledPair> {
        self.pairs.get(&pair_id)
    }
    
    /// Lista todos os pares
    pub fn list_pairs(&self) -> Vec<&EntangledPair> {
        self.pairs.values().collect()
    }
    
    /// Lista pares ativos
    pub fn active_pairs(&self) -> Vec<&EntangledPair> {
        self.pairs.values()
            .filter(|p| p.state == EntanglementState::Active)
            .collect()
    }
    
    /// Remove pares quebrados
    pub fn cleanup(&mut self) {
        let broken: Vec<PairId> = self.pairs.iter()
            .filter(|(_, p)| p.state == EntanglementState::Broken)
            .map(|(id, _)| *id)
            .collect();
        
        for id in broken {
            self.pairs.remove(&id);
        }
    }
    
    fn send_message(&mut self, msg: &SyncMessage) -> bool {
        if let Some(callback) = &self.send_callback {
            callback(msg)
        } else {
            self.pending_messages.push(msg.clone());
            true
        }
    }
    
    /// Drena mensagens pendentes
    pub fn drain_pending(&mut self) -> Vec<SyncMessage> {
        std::mem::take(&mut self.pending_messages)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// THREAD-SAFE WRAPPER
// ═══════════════════════════════════════════════════════════════════════════════

/// Wrapper thread-safe para EntanglementManager
pub struct SharedEntanglementManager(Arc<RwLock<EntanglementManager>>);

impl SharedEntanglementManager {
    pub fn new(node_id: NodeId) -> Self {
        Self(Arc::new(RwLock::new(EntanglementManager::new(node_id))))
    }
    
    pub fn request_entangle(&self, remote: NodeId, initial_state: SilState) -> Option<PairId> {
        self.0.write().ok().map(|mut m| m.request_entangle(remote, initial_state))
    }
    
    pub fn receive_message(&self, msg: &SyncMessage) -> Option<SyncMessage> {
        self.0.write().ok().and_then(|mut m| m.receive_message(msg))
    }
    
    pub fn sync(&self, pair_id: PairId) -> bool {
        self.0.write().ok().map(|mut m| m.sync(pair_id)).unwrap_or(false)
    }
    
    pub fn collapse(&self, pair_id: PairId) -> Option<SilState> {
        self.0.write().ok().and_then(|mut m| m.collapse(pair_id))
    }
    
    pub fn clone_inner(&self) -> Arc<RwLock<EntanglementManager>> {
        Arc::clone(&self.0)
    }
}

impl Clone for SharedEntanglementManager {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// VSP INTEGRATION
// ═══════════════════════════════════════════════════════════════════════════════

use super::{VspResult, VspError};

/// Executa opcode ENTANGLE
pub fn execute_entangle(
    manager: &mut EntanglementManager,
    remote_node: NodeId,
    state: &SilState,
) -> VspResult<PairId> {
    let pair_id = manager.request_entangle(remote_node, state.clone());
    Ok(pair_id)
}

/// Executa opcode SYNC
pub fn execute_sync(
    manager: &mut EntanglementManager,
    pair_id: PairId,
) -> VspResult<()> {
    if manager.sync(pair_id) {
        Ok(())
    } else {
        Err(VspError::EntanglementBroken)
    }
}

/// Executa opcode BROADCAST
pub fn execute_broadcast(
    manager: &mut EntanglementManager,
    state: &SilState,
) -> VspResult<u32> {
    let pair_ids: Vec<PairId> = manager.active_pairs().iter().map(|p| p.id).collect();
    let count = pair_ids.len() as u32;
    
    for pair_id in pair_ids {
        manager.apply_xor(pair_id, state);
    }
    
    Ok(count)
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTES
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_node_id() {
        let id1 = NodeId::new();
        let id2 = NodeId::new();
        assert_ne!(id1, id2);
        
        let bytes = id1.to_bytes();
        let id1_restored = NodeId::from_bytes(&bytes);
        assert_eq!(id1, id1_restored);
    }
    
    #[test]
    fn test_entangle_pair() {
        let node_a = NodeId::new();
        let node_b = NodeId::new();
        let state = SilState::neutral();
        
        let pair = EntangledPair::new(node_a, node_b, state);
        assert_eq!(pair.state, EntanglementState::Active);
        assert!(pair.is_synced());
    }
    
    #[test]
    fn test_xor_preserves_entanglement() {
        let node_a = NodeId::new();
        let node_b = NodeId::new();
        let state = SilState::neutral();
        
        let mut pair = EntangledPair::new(node_a, node_b, state);
        
        let other = SilState::from_layers([ByteSil::from_u8(0xFF); 16]);
        pair.xor(&other);
        
        // Entanglement should still be active
        assert_eq!(pair.state, EntanglementState::Active);
        assert_eq!(pair.version, 1);
    }
    
    #[test]
    fn test_collapse_breaks_entanglement() {
        let node_a = NodeId::new();
        let node_b = NodeId::new();
        let state = SilState::neutral();
        
        let mut pair = EntangledPair::new(node_a, node_b, state);
        let _result = pair.collapse();
        
        assert_eq!(pair.state, EntanglementState::Broken);
    }
    
    #[test]
    fn test_sync_message_roundtrip() {
        let from = NodeId::new();
        let to = NodeId::new();
        let pair_id = PairId::new();
        let state = SilState::from_layers([ByteSil::from_u8(0x42); 16]);
        
        let msg = SyncMessage::new(
            SyncMessageType::StateSync,
            from,
            to,
            pair_id,
        ).with_state(&state).with_version(42);
        
        let bytes = msg.to_bytes();
        let restored = SyncMessage::from_bytes(&bytes).unwrap();
        
        assert_eq!(restored.msg_type, SyncMessageType::StateSync);
        assert_eq!(restored.from, from);
        assert_eq!(restored.to, to);
        assert_eq!(restored.version, 42);
    }
    
    #[test]
    fn test_entanglement_manager_local() {
        let mut manager_a = EntanglementManager::new(NodeId::new());
        let mut manager_b = EntanglementManager::new(NodeId::new());
        
        let state = SilState::neutral();
        let _pair_id = manager_a.request_entangle(manager_b.node_id, state);
        
        // Simulate message passing
        let pending = manager_a.drain_pending();
        assert_eq!(pending.len(), 1);
        
        let response = manager_b.receive_message(&pending[0]);
        assert!(response.is_some());
    }
}
