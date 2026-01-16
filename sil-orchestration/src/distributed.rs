//! # Distributed Orchestration
//!
//! Coordenação multi-node para orquestração distribuída.
//!
//! ## Arquitetura
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    DistributedOrchestrator                      │
//! │  ┌───────────────────────────────────────────────────────────┐  │
//! │  │                 Local Orchestrator                         │  │
//! │  │  Components | Events | Pipeline | State                   │  │
//! │  └───────────────────────────────────────────────────────────┘  │
//! │  ┌───────────────────────────────────────────────────────────┐  │
//! │  │                 Network Layer                              │  │
//! │  │  State Sync | Event Broadcast | Leader Election           │  │
//! │  └───────────────────────────────────────────────────────────┘  │
//! │  ┌───────────────────────────────────────────────────────────┐  │
//! │  │                 Cluster State                              │  │
//! │  │  Nodes | Leader | Consensus | Partitions                  │  │
//! │  └───────────────────────────────────────────────────────────┘  │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Modos de Operação
//!
//! | Modo | Descrição | Use Case |
//! |------|-----------|----------|
//! | Standalone | Nó único, sem rede | Desenvolvimento, testes |
//! | Cluster | Multi-node com líder | Produção, alta disponibilidade |
//! | Swarm | Descentralizado, peer-to-peer | IoT, edge computing |
//!
//! ## Exemplo
//!
//! ```ignore
//! use sil_orchestration::distributed::{DistributedOrchestrator, ClusterConfig};
//!
//! let config = ClusterConfig::default()
//!     .with_node_id("node-1")
//!     .with_peers(vec!["192.168.1.2:21000", "192.168.1.3:21000"]);
//!
//! let mut orch = DistributedOrchestrator::new(config)?;
//! orch.join_cluster().await?;
//! orch.run().await?;
//! ```

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use sil_core::prelude::*;

use crate::orchestrator::{Orchestrator, OrchestratorConfig};
use crate::error::{OrchestrationError, OrchestrationResult};
use crate::lockfree::LockFreeEventBus;

// ═══════════════════════════════════════════════════════════════════════════════
// CONFIGURATION
// ═══════════════════════════════════════════════════════════════════════════════

/// Configuração do cluster
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterConfig {
    /// ID único do nó
    pub node_id: String,
    /// Endereço de bind
    pub bind_addr: SocketAddr,
    /// Peers iniciais (bootstrap)
    pub bootstrap_peers: Vec<SocketAddr>,
    /// Modo de operação
    pub mode: ClusterMode,
    /// Intervalo de heartbeat (ms)
    pub heartbeat_interval_ms: u64,
    /// Timeout para detecção de falha (ms)
    pub failure_timeout_ms: u64,
    /// Intervalo de sincronização de estado (ms)
    pub state_sync_interval_ms: u64,
    /// Configuração do orquestrador local
    pub orchestrator_config: OrchestratorConfig,
    /// Habilitar replicação de eventos
    pub replicate_events: bool,
    /// Habilitar sincronização de estado
    pub sync_state: bool,
}

impl Default for ClusterConfig {
    fn default() -> Self {
        Self {
            node_id: uuid_v4(),
            bind_addr: "0.0.0.0:21000".parse().unwrap(),
            bootstrap_peers: vec![],
            mode: ClusterMode::Standalone,
            heartbeat_interval_ms: 1000,
            failure_timeout_ms: 5000,
            state_sync_interval_ms: 100,
            orchestrator_config: OrchestratorConfig::default(),
            replicate_events: true,
            sync_state: true,
        }
    }
}

impl ClusterConfig {
    /// Define ID do nó
    pub fn with_node_id(mut self, id: impl Into<String>) -> Self {
        self.node_id = id.into();
        self
    }

    /// Define endereço de bind
    pub fn with_bind_addr(mut self, addr: SocketAddr) -> Self {
        self.bind_addr = addr;
        self
    }

    /// Adiciona peer de bootstrap
    pub fn with_peer(mut self, addr: SocketAddr) -> Self {
        self.bootstrap_peers.push(addr);
        self
    }

    /// Define peers de bootstrap
    pub fn with_peers(mut self, peers: Vec<SocketAddr>) -> Self {
        self.bootstrap_peers = peers;
        self
    }

    /// Define modo de operação
    pub fn with_mode(mut self, mode: ClusterMode) -> Self {
        self.mode = mode;
        self
    }

    /// Modo cluster com peers
    pub fn cluster(peers: Vec<SocketAddr>) -> Self {
        Self::default()
            .with_mode(ClusterMode::Cluster)
            .with_peers(peers)
    }

    /// Modo swarm descentralizado
    pub fn swarm() -> Self {
        Self::default().with_mode(ClusterMode::Swarm)
    }
}

/// Modo de operação do cluster
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ClusterMode {
    /// Nó único, sem coordenação
    #[default]
    Standalone,
    /// Cluster com eleição de líder
    Cluster,
    /// Swarm descentralizado (peer-to-peer)
    Swarm,
}

// ═══════════════════════════════════════════════════════════════════════════════
// NODE STATE
// ═══════════════════════════════════════════════════════════════════════════════

/// Estado de um nó no cluster
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeState {
    /// ID do nó
    pub id: String,
    /// Endereço do nó
    pub addr: SocketAddr,
    /// Status do nó
    pub status: NodeStatus,
    /// Último heartbeat recebido
    pub last_heartbeat: u64,
    /// Estado SIL do nó
    pub sil_state: SilState,
    /// Carga do nó (0.0 - 1.0)
    pub load: f32,
    /// Capacidade de processamento
    pub capacity: NodeCapacity,
}

/// Status do nó
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum NodeStatus {
    /// Inicializando
    #[default]
    Initializing,
    /// Ativo e saudável
    Healthy,
    /// Suspeito (heartbeats atrasados)
    Suspect,
    /// Falhou (timeout)
    Failed,
    /// Desligando gracefully
    Draining,
}

/// Capacidade do nó
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NodeCapacity {
    /// Número de CPUs
    pub cpus: u32,
    /// Memória em MB
    pub memory_mb: u64,
    /// Tem GPU?
    pub has_gpu: bool,
    /// Tem NPU?
    pub has_npu: bool,
    /// Layers suportadas
    pub supported_layers: Vec<LayerId>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// CLUSTER STATE
// ═══════════════════════════════════════════════════════════════════════════════

/// Estado do cluster
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterState {
    /// Nós conhecidos
    pub nodes: HashMap<String, NodeState>,
    /// ID do líder atual (se modo Cluster)
    pub leader_id: Option<String>,
    /// Termo atual (epoch para eleição)
    pub term: u64,
    /// Estado global agregado
    pub global_state: SilState,
    /// Timestamp da última atualização
    pub updated_at: u64,
}

impl Default for ClusterState {
    fn default() -> Self {
        Self {
            nodes: HashMap::new(),
            leader_id: None,
            term: 0,
            global_state: SilState::default(),
            updated_at: timestamp_ms(),
        }
    }
}

impl ClusterState {
    /// Adiciona ou atualiza nó
    pub fn upsert_node(&mut self, node: NodeState) {
        self.nodes.insert(node.id.clone(), node);
        self.updated_at = timestamp_ms();
    }

    /// Remove nó
    pub fn remove_node(&mut self, id: &str) {
        self.nodes.remove(id);
        self.updated_at = timestamp_ms();
    }

    /// Retorna nós saudáveis
    pub fn healthy_nodes(&self) -> Vec<&NodeState> {
        self.nodes.values()
            .filter(|n| n.status == NodeStatus::Healthy)
            .collect()
    }

    /// Número de nós ativos
    pub fn active_count(&self) -> usize {
        self.nodes.values()
            .filter(|n| matches!(n.status, NodeStatus::Healthy | NodeStatus::Suspect))
            .count()
    }

    /// Verifica se tem quórum (maioria)
    pub fn has_quorum(&self) -> bool {
        let total = self.nodes.len();
        if total == 0 {
            return false;
        }
        let healthy = self.healthy_nodes().len();
        healthy > total / 2
    }

    /// Agrega estados de todos os nós
    pub fn aggregate_state(&mut self) {
        let mut aggregated = SilState::default();

        for node in self.healthy_nodes() {
            for (i, layer) in node.sil_state.layers.iter().enumerate() {
                // Média ponderada por carga inversa (nós menos carregados têm mais peso)
                let weight = 1.0 - node.load;
                let current = aggregated.layers[i];

                // Interpolação simples
                aggregated.layers[i] = ByteSil::new(
                    (current.rho as f32 * (1.0 - weight) + layer.rho as f32 * weight) as i8,
                    (current.theta as f32 * (1.0 - weight) + layer.theta as f32 * weight) as u8,
                );
            }
        }

        self.global_state = aggregated;
        self.updated_at = timestamp_ms();
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MESSAGES
// ═══════════════════════════════════════════════════════════════════════════════

/// Mensagens do protocolo de coordenação
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoordinationMessage {
    /// Heartbeat periódico
    Heartbeat {
        node_id: String,
        term: u64,
        state: SilState,
        load: f32,
    },
    /// Resposta ao heartbeat
    HeartbeatAck {
        node_id: String,
        term: u64,
    },
    /// Solicitação de voto para eleição
    RequestVote {
        candidate_id: String,
        term: u64,
    },
    /// Voto na eleição
    Vote {
        voter_id: String,
        candidate_id: String,
        term: u64,
        granted: bool,
    },
    /// Evento replicado
    EventBroadcast {
        source_id: String,
        event: SilEvent,
        term: u64,
    },
    /// Sincronização de estado
    StateSync {
        node_id: String,
        state: SilState,
        term: u64,
    },
    /// Pedido para juntar ao cluster
    JoinRequest {
        node_id: String,
        addr: SocketAddr,
        capacity: NodeCapacity,
    },
    /// Confirmação de entrada no cluster
    JoinAck {
        accepted: bool,
        leader_id: Option<String>,
        cluster_state: Option<ClusterState>,
    },
    /// Notificação de saída
    LeaveNotification {
        node_id: String,
    },
}

// ═══════════════════════════════════════════════════════════════════════════════
// DISTRIBUTED ORCHESTRATOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Orquestrador distribuído
pub struct DistributedOrchestrator {
    /// Configuração
    config: ClusterConfig,
    /// Orquestrador local
    local: Orchestrator,
    /// Estado do cluster
    cluster_state: Arc<RwLock<ClusterState>>,
    /// Event bus distribuído
    event_bus: Arc<LockFreeEventBus>,
    /// Fila de mensagens para enviar
    outgoing: Arc<RwLock<Vec<(SocketAddr, CoordinationMessage)>>>,
    /// Este nó é o líder?
    is_leader: Arc<RwLock<bool>>,
    /// Timestamp de criação
    created_at: Instant,
    /// Está rodando?
    running: Arc<RwLock<bool>>,
}

impl DistributedOrchestrator {
    /// Cria novo orquestrador distribuído
    pub fn new(config: ClusterConfig) -> OrchestrationResult<Self> {
        let local = Orchestrator::with_config(config.orchestrator_config.clone());

        let mut cluster_state = ClusterState::default();

        // Adiciona este nó ao cluster
        cluster_state.upsert_node(NodeState {
            id: config.node_id.clone(),
            addr: config.bind_addr,
            status: NodeStatus::Initializing,
            last_heartbeat: timestamp_ms(),
            sil_state: SilState::default(),
            load: 0.0,
            capacity: NodeCapacity::default(),
        });

        Ok(Self {
            config,
            local,
            cluster_state: Arc::new(RwLock::new(cluster_state)),
            event_bus: Arc::new(LockFreeEventBus::new()),
            outgoing: Arc::new(RwLock::new(Vec::new())),
            is_leader: Arc::new(RwLock::new(false)),
            created_at: Instant::now(),
            running: Arc::new(RwLock::new(false)),
        })
    }

    /// ID deste nó
    pub fn node_id(&self) -> &str {
        &self.config.node_id
    }

    /// Modo de operação
    pub fn mode(&self) -> ClusterMode {
        self.config.mode
    }

    /// Este nó é líder?
    pub fn is_leader(&self) -> bool {
        *self.is_leader.read().unwrap()
    }

    /// Retorna estado do cluster
    pub fn cluster_state(&self) -> ClusterState {
        self.cluster_state.read().unwrap().clone()
    }

    /// Número de nós no cluster
    pub fn node_count(&self) -> usize {
        self.cluster_state.read().unwrap().nodes.len()
    }

    /// Nós saudáveis
    pub fn healthy_nodes(&self) -> Vec<String> {
        self.cluster_state.read().unwrap()
            .healthy_nodes()
            .iter()
            .map(|n| n.id.clone())
            .collect()
    }

    /// Inicia o orquestrador distribuído
    pub fn start(&self) -> OrchestrationResult<()> {
        let mut running = self.running.write()?;
        if *running {
            return Err(OrchestrationError::InvalidPipeline(
                "Already running".into()
            ));
        }

        *running = true;

        // Atualiza status do nó local
        {
            let mut state = self.cluster_state.write()?;
            if let Some(node) = state.nodes.get_mut(&self.config.node_id) {
                node.status = NodeStatus::Healthy;
                node.last_heartbeat = timestamp_ms();
            }
        }

        // Em modo standalone, este nó é sempre o líder
        if self.config.mode == ClusterMode::Standalone {
            *self.is_leader.write()? = true;
        }

        // Inicia orquestrador local
        self.local.start()?;

        Ok(())
    }

    /// Para o orquestrador
    pub fn stop(&self) -> OrchestrationResult<()> {
        let mut running = self.running.write()?;
        *running = false;

        // Atualiza status do nó
        {
            let mut state = self.cluster_state.write()?;
            if let Some(node) = state.nodes.get_mut(&self.config.node_id) {
                node.status = NodeStatus::Draining;
            }
        }

        // Envia notificação de saída
        if self.config.mode != ClusterMode::Standalone {
            self.broadcast_message(CoordinationMessage::LeaveNotification {
                node_id: self.config.node_id.clone(),
            })?;
        }

        self.local.stop()?;

        Ok(())
    }

    /// Verifica se está rodando
    pub fn is_running(&self) -> bool {
        *self.running.read().unwrap()
    }

    /// Executa um tick do orquestrador
    pub fn tick(&self) -> OrchestrationResult<()> {
        // Tick local
        self.local.tick()?;

        // Atualiza estado local
        let local_state = self.local.state()?;
        {
            let mut cluster = self.cluster_state.write()?;
            if let Some(node) = cluster.nodes.get_mut(&self.config.node_id) {
                node.sil_state = local_state;
                node.last_heartbeat = timestamp_ms();
            }
        }

        // Sincroniza estado se configurado
        if self.config.sync_state && self.config.mode != ClusterMode::Standalone {
            self.sync_state()?;
        }

        Ok(())
    }

    /// Processa mensagem recebida
    pub fn handle_message(&self, from: SocketAddr, msg: CoordinationMessage) -> OrchestrationResult<()> {
        match msg {
            CoordinationMessage::Heartbeat { node_id, term, state, load } => {
                self.handle_heartbeat(node_id, term, state, load)?;
            }
            CoordinationMessage::HeartbeatAck { node_id, term } => {
                self.handle_heartbeat_ack(node_id, term)?;
            }
            CoordinationMessage::RequestVote { candidate_id, term } => {
                self.handle_request_vote(from, candidate_id, term)?;
            }
            CoordinationMessage::Vote { voter_id, candidate_id, term, granted } => {
                self.handle_vote(voter_id, candidate_id, term, granted)?;
            }
            CoordinationMessage::EventBroadcast { source_id, event, term } => {
                self.handle_event_broadcast(source_id, event, term)?;
            }
            CoordinationMessage::StateSync { node_id, state, term } => {
                self.handle_state_sync(node_id, state, term)?;
            }
            CoordinationMessage::JoinRequest { node_id, addr, capacity } => {
                self.handle_join_request(from, node_id, addr, capacity)?;
            }
            CoordinationMessage::JoinAck { accepted, leader_id, cluster_state } => {
                self.handle_join_ack(accepted, leader_id, cluster_state)?;
            }
            CoordinationMessage::LeaveNotification { node_id } => {
                self.handle_leave_notification(node_id)?;
            }
        }
        Ok(())
    }

    // === Message Handlers ===

    fn handle_heartbeat(&self, node_id: String, _term: u64, state: SilState, load: f32) -> OrchestrationResult<()> {
        let mut cluster = self.cluster_state.write()?;

        if let Some(node) = cluster.nodes.get_mut(&node_id) {
            node.status = NodeStatus::Healthy;
            node.last_heartbeat = timestamp_ms();
            node.sil_state = state;
            node.load = load;
        }

        Ok(())
    }

    fn handle_heartbeat_ack(&self, _node_id: String, _term: u64) -> OrchestrationResult<()> {
        // Marca que o nó respondeu
        Ok(())
    }

    fn handle_request_vote(&self, _from: SocketAddr, _candidate_id: String, _term: u64) -> OrchestrationResult<()> {
        // Implementação de eleição de líder (Raft-like)
        // Por agora, sempre vota no candidato
        Ok(())
    }

    fn handle_vote(&self, _voter_id: String, _candidate_id: String, _term: u64, _granted: bool) -> OrchestrationResult<()> {
        // Contabiliza votos para eleição
        Ok(())
    }

    fn handle_event_broadcast(&self, _source_id: String, event: SilEvent, _term: u64) -> OrchestrationResult<()> {
        // Replica evento localmente
        self.event_bus.emit(event);
        Ok(())
    }

    fn handle_state_sync(&self, node_id: String, state: SilState, _term: u64) -> OrchestrationResult<()> {
        let mut cluster = self.cluster_state.write()?;

        if let Some(node) = cluster.nodes.get_mut(&node_id) {
            node.sil_state = state;
            node.last_heartbeat = timestamp_ms();
        }

        // Reagrega estado global
        cluster.aggregate_state();

        Ok(())
    }

    fn handle_join_request(&self, from: SocketAddr, node_id: String, addr: SocketAddr, capacity: NodeCapacity) -> OrchestrationResult<()> {
        // Adiciona novo nó ao cluster
        let mut cluster = self.cluster_state.write()?;

        cluster.upsert_node(NodeState {
            id: node_id.clone(),
            addr,
            status: NodeStatus::Initializing,
            last_heartbeat: timestamp_ms(),
            sil_state: SilState::default(),
            load: 0.0,
            capacity,
        });

        // Envia ACK com estado do cluster
        let ack = CoordinationMessage::JoinAck {
            accepted: true,
            leader_id: cluster.leader_id.clone(),
            cluster_state: Some(cluster.clone()),
        };

        drop(cluster); // Release lock

        self.queue_message(from, ack)?;

        Ok(())
    }

    fn handle_join_ack(&self, accepted: bool, leader_id: Option<String>, cluster_state: Option<ClusterState>) -> OrchestrationResult<()> {
        if !accepted {
            return Err(OrchestrationError::InvalidPipeline("Join rejected".into()));
        }

        if let Some(state) = cluster_state {
            *self.cluster_state.write()? = state;
        }

        // Se temos líder, não somos o líder
        if leader_id.is_some() {
            *self.is_leader.write()? = false;
        }

        // Atualiza status para healthy
        {
            let mut cluster = self.cluster_state.write()?;
            if let Some(node) = cluster.nodes.get_mut(&self.config.node_id) {
                node.status = NodeStatus::Healthy;
            }
        }

        Ok(())
    }

    fn handle_leave_notification(&self, node_id: String) -> OrchestrationResult<()> {
        let mut cluster = self.cluster_state.write()?;
        cluster.remove_node(&node_id);

        // Se o líder saiu, inicia eleição
        if cluster.leader_id.as_ref() == Some(&node_id) {
            cluster.leader_id = None;
            // TODO: Iniciar eleição
        }

        Ok(())
    }

    // === Sync Operations ===

    /// Sincroniza estado com o cluster
    fn sync_state(&self) -> OrchestrationResult<()> {
        let state = self.local.state()?;
        let term = self.cluster_state.read()?.term;

        self.broadcast_message(CoordinationMessage::StateSync {
            node_id: self.config.node_id.clone(),
            state,
            term,
        })?;

        Ok(())
    }

    /// Envia heartbeat para todos os peers
    pub fn send_heartbeat(&self) -> OrchestrationResult<()> {
        let state = self.local.state()?;
        let cluster = self.cluster_state.read()?;

        let msg = CoordinationMessage::Heartbeat {
            node_id: self.config.node_id.clone(),
            term: cluster.term,
            state,
            load: 0.0, // TODO: Calcular carga real
        };

        drop(cluster);

        self.broadcast_message(msg)?;

        Ok(())
    }

    /// Broadcast mensagem para todos os peers
    fn broadcast_message(&self, msg: CoordinationMessage) -> OrchestrationResult<()> {
        let cluster = self.cluster_state.read()?;

        for (node_id, node) in &cluster.nodes {
            if node_id != &self.config.node_id {
                let mut outgoing = self.outgoing.write()?;
                outgoing.push((node.addr, msg.clone()));
            }
        }

        Ok(())
    }

    /// Enfileira mensagem para enviar
    fn queue_message(&self, addr: SocketAddr, msg: CoordinationMessage) -> OrchestrationResult<()> {
        let mut outgoing = self.outgoing.write()?;
        outgoing.push((addr, msg));
        Ok(())
    }

    /// Drena fila de mensagens para envio
    pub fn drain_outgoing(&self) -> Vec<(SocketAddr, CoordinationMessage)> {
        let mut outgoing = self.outgoing.write().unwrap();
        std::mem::take(&mut *outgoing)
    }

    /// Acesso ao orquestrador local
    pub fn local(&self) -> &Orchestrator {
        &self.local
    }

    /// Acesso mutável ao orquestrador local
    pub fn local_mut(&mut self) -> &mut Orchestrator {
        &mut self.local
    }

    /// Estatísticas do cluster
    pub fn stats(&self) -> DistributedStats {
        let cluster = self.cluster_state.read().unwrap();
        let local_stats = self.local.stats().unwrap_or_else(|_| {
            crate::orchestrator::OrchestratorStats {
                component_count: 0,
                sensor_count: 0,
                processor_count: 0,
                actuator_count: 0,
                network_node_count: 0,
                governor_count: 0,
                swarm_agent_count: 0,
                quantum_state_count: 0,
                forkable_count: 0,
                entangled_count: 0,
                collapsible_count: 0,
                pipeline_cycles: 0,
                event_count: 0,
                uptime: Duration::ZERO,
            }
        });

        DistributedStats {
            node_id: self.config.node_id.clone(),
            mode: self.config.mode,
            is_leader: self.is_leader(),
            node_count: cluster.nodes.len(),
            healthy_count: cluster.healthy_nodes().len(),
            term: cluster.term,
            has_quorum: cluster.has_quorum(),
            local_stats,
            uptime: self.created_at.elapsed(),
        }
    }
}

/// Estatísticas do orquestrador distribuído
#[derive(Debug, Clone)]
pub struct DistributedStats {
    pub node_id: String,
    pub mode: ClusterMode,
    pub is_leader: bool,
    pub node_count: usize,
    pub healthy_count: usize,
    pub term: u64,
    pub has_quorum: bool,
    pub local_stats: crate::orchestrator::OrchestratorStats,
    pub uptime: Duration,
}

// ═══════════════════════════════════════════════════════════════════════════════
// HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

/// Gera UUID v4 simples
fn uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    format!("{:032x}", now)
}

/// Timestamp em millisegundos
fn timestamp_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cluster_config_default() {
        let config = ClusterConfig::default();
        assert_eq!(config.mode, ClusterMode::Standalone);
        assert!(config.bootstrap_peers.is_empty());
    }

    #[test]
    fn test_cluster_config_builder() {
        let config = ClusterConfig::default()
            .with_node_id("test-node")
            .with_mode(ClusterMode::Cluster)
            .with_peer("127.0.0.1:21001".parse().unwrap());

        assert_eq!(config.node_id, "test-node");
        assert_eq!(config.mode, ClusterMode::Cluster);
        assert_eq!(config.bootstrap_peers.len(), 1);
    }

    #[test]
    fn test_cluster_state() {
        let mut state = ClusterState::default();

        state.upsert_node(NodeState {
            id: "node-1".into(),
            addr: "127.0.0.1:21000".parse().unwrap(),
            status: NodeStatus::Healthy,
            last_heartbeat: timestamp_ms(),
            sil_state: SilState::default(),
            load: 0.3,
            capacity: NodeCapacity::default(),
        });

        assert_eq!(state.nodes.len(), 1);
        assert_eq!(state.healthy_nodes().len(), 1);
    }

    #[test]
    fn test_distributed_orchestrator_standalone() {
        let config = ClusterConfig::default();
        let orch = DistributedOrchestrator::new(config).unwrap();

        assert_eq!(orch.mode(), ClusterMode::Standalone);
        assert_eq!(orch.node_count(), 1);
    }

    #[test]
    fn test_distributed_orchestrator_start_stop() {
        let config = ClusterConfig::default();
        let orch = DistributedOrchestrator::new(config).unwrap();

        assert!(!orch.is_running());

        orch.start().unwrap();
        assert!(orch.is_running());
        assert!(orch.is_leader()); // Standalone = sempre líder

        orch.stop().unwrap();
        assert!(!orch.is_running());
    }

    #[test]
    fn test_cluster_quorum() {
        let mut state = ClusterState::default();

        // 0 nós - sem quórum
        assert!(!state.has_quorum());

        // 1 nó saudável - tem quórum (1 > 1/2 = 0)
        state.upsert_node(NodeState {
            id: "node-1".into(),
            addr: "127.0.0.1:21000".parse().unwrap(),
            status: NodeStatus::Healthy,
            last_heartbeat: timestamp_ms(),
            sil_state: SilState::default(),
            load: 0.0,
            capacity: NodeCapacity::default(),
        });
        assert!(state.has_quorum()); // 1 de 1, 1 > 0

        // 2 nós, 1 saudável - sem quórum
        state.upsert_node(NodeState {
            id: "node-2".into(),
            addr: "127.0.0.1:21001".parse().unwrap(),
            status: NodeStatus::Failed,
            last_heartbeat: 0,
            sil_state: SilState::default(),
            load: 0.0,
            capacity: NodeCapacity::default(),
        });
        assert!(!state.has_quorum()); // 1 de 2 = 50%, precisa >50%

        // 3 nós, 2 saudáveis - tem quórum
        state.upsert_node(NodeState {
            id: "node-3".into(),
            addr: "127.0.0.1:21002".parse().unwrap(),
            status: NodeStatus::Healthy,
            last_heartbeat: timestamp_ms(),
            sil_state: SilState::default(),
            load: 0.0,
            capacity: NodeCapacity::default(),
        });
        assert!(state.has_quorum()); // 2 de 3 = 66% > 50%
    }

    #[test]
    fn test_state_aggregation() {
        let mut state = ClusterState::default();

        // Adiciona dois nós com estados diferentes
        let mut s1 = SilState::default();
        s1.layers[0] = ByteSil::new(4, 0);

        let mut s2 = SilState::default();
        s2.layers[0] = ByteSil::new(2, 0);

        state.upsert_node(NodeState {
            id: "node-1".into(),
            addr: "127.0.0.1:21000".parse().unwrap(),
            status: NodeStatus::Healthy,
            last_heartbeat: timestamp_ms(),
            sil_state: s1,
            load: 0.0,
            capacity: NodeCapacity::default(),
        });

        state.upsert_node(NodeState {
            id: "node-2".into(),
            addr: "127.0.0.1:21001".parse().unwrap(),
            status: NodeStatus::Healthy,
            last_heartbeat: timestamp_ms(),
            sil_state: s2,
            load: 0.0,
            capacity: NodeCapacity::default(),
        });

        state.aggregate_state();

        // Estado agregado deve ser média
        assert!(state.global_state.layers[0].rho >= 2);
        assert!(state.global_state.layers[0].rho <= 4);
    }
}
