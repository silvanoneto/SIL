//! SilNode — Implementação do trait NetworkNode
//!
//! Nó de rede SIL que implementa o trait `NetworkNode` de `sil-core`.

use std::collections::VecDeque;
use std::fmt::Debug;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use sil_core::traits::{
    LayerId, NetworkError as CoreNetworkError, NetworkNode, PeerInfo, SilComponent, Timestamp,
};
use sil_core::SilState;

use crate::message::{ControlMessage, DiscoveryMessage, SilMessage, SilPayload};
use crate::peer::{PeerId, PeerRegistry, PeerState};
use crate::transport::{Transport, UdpTransport, DEFAULT_PORT};
use crate::NetworkError;

/// Configuração do nó
#[derive(Debug, Clone)]
pub struct NodeConfig {
    /// Porta de escuta
    pub port: u16,
    /// Nome do nó (identificação)
    pub name: String,
    /// Timeout para peers inativos
    pub peer_timeout: Duration,
    /// Habilitar discovery multicast
    pub enable_discovery: bool,
    /// Máximo de peers
    pub max_peers: usize,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            port: DEFAULT_PORT,
            name: format!("sil-node-{:08x}", rand::random::<u32>()),
            peer_timeout: Duration::from_secs(30),
            enable_discovery: true,
            max_peers: 64,
        }
    }
}

impl NodeConfig {
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    // Aliases para compatibilidade com CLI
    pub fn with_node_id(self, id: impl Into<String>) -> Self {
        self.with_name(id)
    }

    pub fn with_listen_port(self, port: u16) -> Self {
        self.with_port(port)
    }

    pub fn with_enable_multicast(mut self, enable: bool) -> Self {
        self.enable_discovery = enable;
        self
    }
}

/// Nó SIL que implementa NetworkNode
#[derive(Debug)]
pub struct SilNode {
    /// ID único do nó
    id: PeerId,
    /// Nome do nó
    name: String,
    /// Registry de peers
    peers: PeerRegistry,
    /// Transporte UDP
    transport: UdpTransport,
    /// Fila de mensagens recebidas
    inbox: VecDeque<(PeerId, SilMessage)>,
    /// Sequence number para mensagens
    seq: u64,
    /// Estado local
    local_state: SilState,
    /// Configuração
    config: NodeConfig,
    /// Momento de criação
    created_at: Instant,
}

impl SilNode {
    /// Cria novo nó com configuração
    pub fn new(config: NodeConfig) -> Result<Self, NetworkError> {
        let mut transport = UdpTransport::bind(config.port)?;

        if config.enable_discovery {
            let _ = transport.join_multicast(); // Ignora erro se multicast não disponível
        }

        // Gera ID baseado em timestamp + random
        let id = PeerId::random();

        Ok(Self {
            id,
            name: config.name.clone(),
            peers: PeerRegistry::new().with_timeout(config.peer_timeout),
            transport,
            inbox: VecDeque::new(),
            seq: 0,
            local_state: SilState::default(),
            config,
            created_at: Instant::now(),
        })
    }

    /// Cria nó com configuração padrão
    pub fn default_node() -> Result<Self, NetworkError> {
        Self::new(NodeConfig::default())
    }

    /// ID do nó
    pub fn id(&self) -> PeerId {
        self.id
    }

    /// Tempo desde a criação do nó
    pub fn uptime(&self) -> std::time::Duration {
        self.created_at.elapsed()
    }

    /// Estado local atual
    pub fn local_state(&self) -> &SilState {
        &self.local_state
    }

    /// Atualiza estado local
    pub fn set_local_state(&mut self, state: SilState) {
        self.local_state = state;
    }

    /// Número de peers conectados
    pub fn connected_count(&self) -> usize {
        self.peers.connected_count()
    }

    /// Endereço local
    pub fn local_addr(&self) -> Result<SocketAddr, NetworkError> {
        self.transport.local_addr()
    }

    /// Conecta a um peer por endereço
    pub fn connect(&mut self, addr: SocketAddr) -> Result<PeerId, NetworkError> {
        // Envia hello
        let hello = SilMessage::hello();
        self.transport.send_sil(&hello, addr)?;

        // Registra peer (ID temporário até receber resposta)
        let temp_id = PeerId(addr.port() as u64 ^ addr.ip().to_string().len() as u64);
        let entry = self.peers.upsert(temp_id, addr);
        entry.state = PeerState::Connecting;

        Ok(temp_id)
    }

    /// Desconecta de um peer
    pub fn disconnect(&mut self, peer_id: &PeerId) -> Result<(), NetworkError> {
        // Obtem endereço primeiro
        let addr = match self.peers.get(peer_id) {
            Some(entry) => entry.addr,
            None => return Ok(()),
        };

        // Envia goodbye
        let goodbye = SilMessage {
            msg_type: crate::MessageType::Control,
            payload: SilPayload::Control(ControlMessage::Goodbye),
            timestamp: now_nanos(),
            seq: self.next_seq(),
        };

        let _ = self.transport.send_sil(&goodbye, addr);

        // Atualiza estado
        if let Some(entry) = self.peers.get_mut(peer_id) {
            entry.state = PeerState::Disconnected;
        }
        Ok(())
    }

    /// Processa mensagens pendentes (poll)
    pub fn poll(&mut self) -> Result<(), NetworkError> {
        // Recebe mensagens disponíveis
        while let Some((msg, addr)) = self.transport.recv_sil()? {
            self.handle_message(msg, addr)?;
        }

        // Prune peers inativos
        self.peers.prune_inactive();

        Ok(())
    }

    /// Processa uma mensagem recebida
    fn handle_message(&mut self, msg: SilMessage, addr: SocketAddr) -> Result<(), NetworkError> {
        // Determina peer ID (ou cria temporário)
        let peer_id = self.find_or_create_peer(addr);

        match &msg.payload {
            SilPayload::Control(ctrl) => self.handle_control(ctrl, peer_id, addr)?,
            SilPayload::Discovery(disc) => self.handle_discovery(disc, peer_id, addr)?,
            _ => {
                // Mensagem de dados - coloca na inbox
                self.inbox.push_back((peer_id, msg));
            }
        }

        // Atualiza last_seen
        if let Some(entry) = self.peers.get_mut(&peer_id) {
            entry.touch();
        }

        Ok(())
    }

    fn handle_control(
        &mut self,
        ctrl: &ControlMessage,
        peer_id: PeerId,
        addr: SocketAddr,
    ) -> Result<(), NetworkError> {
        match ctrl {
            ControlMessage::Ping { nonce } => {
                let pong = SilMessage::pong(*nonce);
                self.transport.send_sil(&pong, addr)?;
            }
            ControlMessage::Pong { nonce: _ } => {
                // Calcula latência se tivermos registro do ping
                if let Some(entry) = self.peers.get_mut(&peer_id) {
                    entry.state = PeerState::Connected;
                }
            }
            ControlMessage::Goodbye => {
                if let Some(entry) = self.peers.get_mut(&peer_id) {
                    entry.state = PeerState::Disconnected;
                }
            }
        }
        Ok(())
    }

    fn handle_discovery(
        &mut self,
        disc: &DiscoveryMessage,
        peer_id: PeerId,
        addr: SocketAddr,
    ) -> Result<(), NetworkError> {
        match disc {
            DiscoveryMessage::Hello { version: _, .. } => {
                // Responde com nosso hello
                let hello = SilMessage::hello();
                self.transport.send_sil(&hello, addr)?;

                // Marca como conectado
                if let Some(entry) = self.peers.get_mut(&peer_id) {
                    entry.state = PeerState::Connected;
                }
            }
            DiscoveryMessage::Announce { port } => {
                // Atualiza porta do peer
                let new_addr = SocketAddr::new(addr.ip(), *port);
                self.peers.upsert(peer_id, new_addr);
            }
            DiscoveryMessage::GetPeers => {
                // Envia lista de peers
                let addrs: Vec<String> = self
                    .peers
                    .connected_addrs()
                    .iter()
                    .map(|a| a.to_string())
                    .collect();

                let response = SilMessage {
                    msg_type: crate::MessageType::Discovery,
                    payload: SilPayload::Discovery(DiscoveryMessage::Peers { addrs }),
                    timestamp: now_nanos(),
                    seq: self.next_seq(),
                };
                self.transport.send_sil(&response, addr)?;
            }
            DiscoveryMessage::Peers { addrs } => {
                // Conecta a novos peers descobertos
                for addr_str in addrs {
                    if let Ok(new_addr) = addr_str.parse::<SocketAddr>() {
                        if self.peers.connected_count() < self.config.max_peers {
                            let _ = self.connect(new_addr);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn find_or_create_peer(&mut self, addr: SocketAddr) -> PeerId {
        // Procura peer existente por endereço
        for entry in self.peers.all_peers() {
            if entry.addr == addr {
                return entry.id;
            }
        }

        // Cria novo
        let id = PeerId::random();
        self.peers.upsert(id, addr);
        id
    }

    fn next_seq(&mut self) -> u64 {
        self.seq += 1;
        self.seq
    }

    /// Envia estado para todos os peers
    pub fn broadcast_state(&mut self, state: &SilState) -> Result<usize, NetworkError> {
        let msg = SilMessage::state(state).with_seq(self.next_seq());
        let addrs = self.peers.connected_addrs();
        let mut sent = 0;

        for addr in addrs {
            if self.transport.send_sil(&msg, addr).is_ok() {
                sent += 1;
            }
        }

        Ok(sent)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Implementação do trait SilComponent
// ═══════════════════════════════════════════════════════════════════════════════

impl SilComponent for SilNode {
    fn name(&self) -> &str {
        &self.name
    }

    fn layers(&self) -> &[LayerId] {
        // L8 (Cibernético), L9 (Geopolítico), LA (Cosmopolítico)
        &[8, 9, 10]
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn is_ready(&self) -> bool {
        // Pronto se transport está bound
        self.transport.local_addr().is_ok()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Implementação do trait NetworkNode
// ═══════════════════════════════════════════════════════════════════════════════

impl NetworkNode for SilNode {
    type PeerId = PeerId;
    type Message = SilMessage;

    fn peer_id(&self) -> &Self::PeerId {
        &self.id
    }

    fn send(&mut self, to: &Self::PeerId, msg: Self::Message) -> Result<(), CoreNetworkError> {
        let entry = self
            .peers
            .get(to)
            .ok_or_else(|| CoreNetworkError::PeerNotFound(to.to_string()))?;

        let addr = entry.addr;
        self.transport.send_sil(&msg, addr).map_err(|e| e.into())
    }

    fn recv(&mut self) -> Option<(Self::PeerId, Self::Message)> {
        // Primeiro, processa novas mensagens
        let _ = self.poll();

        // Retorna da inbox
        self.inbox.pop_front()
    }

    fn peers(&self) -> Vec<PeerInfo<Self::PeerId>> {
        self.peers
            .active_peers()
            .iter()
            .map(|e| PeerInfo {
                id: e.id,
                address: e.addr.to_string(),
                last_seen: e.last_seen.elapsed().as_nanos() as Timestamp,
                latency_ms: e.latency_ms,
            })
            .collect()
    }

    fn broadcast(&mut self, msg: Self::Message) -> Result<usize, CoreNetworkError> {
        let addrs = self.peers.connected_addrs();
        let mut sent = 0;

        for addr in addrs {
            if self.transport.send_sil(&msg, addr).is_ok() {
                sent += 1;
            }
        }

        Ok(sent)
    }

    fn discover(&mut self) -> Result<Vec<Self::PeerId>, CoreNetworkError> {
        // Envia announce via multicast
        let _announce = SilMessage {
            msg_type: crate::MessageType::Discovery,
            payload: SilPayload::Discovery(DiscoveryMessage::Announce {
                port: self.config.port,
            }),
            timestamp: now_nanos(),
            seq: self.next_seq(),
        };

        let _ = self
            .transport
            .broadcast(&crate::Message::from_state(
                &SilState::default(),
                self.id.0,
                crate::MessageType::Discovery,
            ));

        // Poll para receber respostas
        let _ = self.poll();

        Ok(self.peers.all_peers().iter().map(|e| e.id).collect())
    }
}

/// Timestamp em nanossegundos
fn now_nanos() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_creation() {
        let config = NodeConfig::default().with_port(0); // Porta aleatória
        let node = SilNode::new(config);
        assert!(node.is_ok());
    }

    #[test]
    fn node_implements_network_node() {
        let config = NodeConfig::default().with_port(0);
        let node = SilNode::new(config).unwrap();

        // Verifica que implementa NetworkNode
        let _: &dyn NetworkNode<PeerId = PeerId, Message = SilMessage> = &node;
    }

    #[test]
    fn node_implements_sil_component() {
        let config = NodeConfig::default().with_port(0);
        let node = SilNode::new(config).unwrap();

        assert!(!node.name().is_empty());
        assert!(node.layers().contains(&8)); // L8 Cibernético
        assert!(node.is_ready());
    }

    #[test]
    fn node_peer_list_starts_empty() {
        let config = NodeConfig::default().with_port(0);
        let node = SilNode::new(config).unwrap();
        assert!(node.peers().is_empty());
    }
}
