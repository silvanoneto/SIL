//! # 🎯 Traits — Abstrações Fundamentais do Sistema SIL
//!
//! Este módulo define os traits base que todos os componentes do ecossistema SIL
//! devem implementar. A arquitetura é organizada por grupos de camadas:
//!
//! | Grupo | Camadas | Traits |
//! |:------|:--------|:-------|
//! | Percepção | L0-L4 | [`Sensor`] |
//! | Processamento | L5-L7 | [`Processor`], [`Actuator`] |
//! | Interação | L8-LA | [`NetworkNode`], [`Governor`] |
//! | Emergência | LB-LC | [`SwarmAgent`], [`QuantumState`] |
//! | Meta | LD-LF | [`Forkable`], [`Entangled`], [`Collapsible`] |
//!
//! ## Princípio de Design
//!
//! > *"Trait no core, implementação no módulo."*
//!
//! Os traits aqui são **abstrações puras**. As implementações concretas vivem
//! nos crates específicos (`sil-photonic`, `sil-network`, etc.).

use crate::state::{ByteSil, SilState};
use std::fmt::Debug;

// ═══════════════════════════════════════════════════════════════════════════════
// TIPOS COMUNS
// ═══════════════════════════════════════════════════════════════════════════════

/// Identificador de camada (0-15, hex 0-F)
pub type LayerId = u8;

/// Timestamp em microsegundos desde epoch
pub type Timestamp = u64;

/// Nível de confiança (0.0 = incerto, 1.0 = certo)
pub type Confidence = f32;

/// Atualização parcial de estado
///
/// Representa uma mudança em uma única camada, com metadados.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SilUpdate {
    /// Camada afetada (0-15)
    pub layer: LayerId,
    /// Novo valor da camada
    pub byte: ByteSil,
    /// Confiança na atualização (0.0 - 1.0)
    pub confidence: Confidence,
    /// Timestamp da atualização (µs)
    pub timestamp: Timestamp,
}

impl SilUpdate {
    /// Cria nova atualização
    pub fn new(layer: LayerId, byte: ByteSil) -> Self {
        Self {
            layer,
            byte,
            confidence: 1.0,
            timestamp: Self::now(),
        }
    }

    /// Cria atualização com confiança específica
    pub fn with_confidence(layer: LayerId, byte: ByteSil, confidence: Confidence) -> Self {
        Self {
            layer,
            byte,
            confidence: confidence.clamp(0.0, 1.0),
            timestamp: Self::now(),
        }
    }

    /// Timestamp atual em microsegundos
    fn now() -> Timestamp {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_micros() as u64)
            .unwrap_or(0)
    }

    /// Aplica esta atualização a um estado
    pub fn apply(&self, state: &SilState) -> SilState {
        state.with_layer(self.layer as usize, self.byte)
    }
}

/// Evento emitido por componentes SIL
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum SilEvent {
    /// Mudança de estado em uma camada
    StateChange {
        layer: LayerId,
        old: ByteSil,
        new: ByteSil,
        timestamp: Timestamp,
    },
    /// Threshold ultrapassado
    Threshold {
        layer: LayerId,
        value: f32,
        threshold: f32,
        direction: ThresholdDirection,
    },
    /// Erro em componente
    Error {
        component: String,
        message: String,
        recoverable: bool,
    },
    /// Componente pronto
    Ready {
        component: String,
    },
    /// Componente desligando
    Shutdown {
        component: String,
        reason: String,
    },
}

/// Direção de threshold
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ThresholdDirection {
    Rising,
    Falling,
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRAIT BASE — Todo componente SIL
// ═══════════════════════════════════════════════════════════════════════════════

/// Trait base para qualquer componente do ecossistema SIL.
///
/// Todo sensor, processador, atuador, nó de rede, etc. implementa este trait.
///
/// # Exemplo
///
/// ```ignore
/// use sil_core::traits::SilComponent;
///
/// struct MyCamera;
///
/// impl SilComponent for MyCamera {
///     fn name(&self) -> &str { "my-camera" }
///     fn layers(&self) -> &[u8] { &[0] }  // L0 = Fotônico
/// }
/// ```
pub trait SilComponent: Send + Sync + Debug {
    /// Nome único do componente (para logs e debug)
    fn name(&self) -> &str;

    /// Camada(s) que este componente afeta (0-15)
    ///
    /// - Sensor de câmera: `&[0]` (L0)
    /// - Mediador de rede: `&[8, 9, 10]` (L8-LA)
    /// - Orquestrador: `&[0..=15]` (todas)
    fn layers(&self) -> &[LayerId];

    /// Versão do componente (para compatibilidade)
    fn version(&self) -> &str {
        "2026.1.16"
    }

    /// Componente está pronto para uso?
    fn is_ready(&self) -> bool {
        true
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// PERCEPÇÃO (L0-L4) — Sensores
// ═══════════════════════════════════════════════════════════════════════════════

/// Erro de sensor
#[derive(Debug, Clone, thiserror::Error)]
pub enum SensorError {
    #[error("Sensor not initialized")]
    NotInitialized,
    #[error("Sensor read failed: {0}")]
    ReadFailed(String),
    #[error("Calibration failed: {0}")]
    CalibrationFailed(String),
    #[error("Configuration invalid: {0}")]
    InvalidConfig(String),
    #[error("Hardware error: {0}")]
    Hardware(String),
    #[error("Timeout after {0}ms")]
    Timeout(u64),
}

/// Trait para sensores que populam as camadas de percepção (L0-L4).
///
/// # Camadas
///
/// | Camada | Tipo | Exemplo |
/// |:-------|:-----|:--------|
/// | L0 | Fotônico | Câmera, LiDAR |
/// | L1 | Acústico | Microfone, Ultrassom |
/// | L2 | Olfativo | Sensor de gás |
/// | L3 | Gustativo | Sensor de pH |
/// | L4 | Dérmico | Pressão, temperatura |
///
/// # Exemplo
///
/// ```ignore
/// use sil_core::traits::{Sensor, SensorError};
/// use sil_core::state::ByteSil;
///
/// struct TemperatureSensor {
///     pin: u8,
/// }
///
/// impl Sensor for TemperatureSensor {
///     type RawData = f32;  // Celsius
///     type Config = u8;    // Pin number
///
///     fn read(&mut self) -> Result<f32, SensorError> {
///         // Ler do hardware...
///         Ok(25.0)
///     }
///
///     fn to_byte_sil(&self, celsius: &f32) -> ByteSil {
///         // Mapear -40°C..+85°C para ρ=-8..+7
///         let rho = ((*celsius + 40.0) / 125.0 * 15.0 - 8.0) as i8;
///         ByteSil::new(rho.clamp(-8, 7), 0)
///     }
///
///     fn target_layer(&self) -> u8 { 4 }  // L4 = Dérmico
/// }
/// ```
pub trait Sensor: SilComponent {
    /// Tipo dos dados brutos lidos do sensor
    type RawData;

    /// Tipo de configuração do sensor
    type Config;

    /// Configura o sensor
    fn configure(&mut self, _config: Self::Config) -> Result<(), SensorError> {
        Ok(())
    }

    /// Lê dados brutos do sensor
    fn read(&mut self) -> Result<Self::RawData, SensorError>;

    /// Converte dados brutos para ByteSil
    fn to_byte_sil(&self, raw: &Self::RawData) -> ByteSil;

    /// Camada alvo deste sensor (0-4)
    fn target_layer(&self) -> LayerId;

    /// Taxa de amostragem em Hz (0 = sob demanda)
    fn sample_rate(&self) -> f32 {
        0.0
    }

    /// Calibra o sensor
    fn calibrate(&mut self) -> Result<(), SensorError> {
        Ok(())
    }

    /// Lê e converte em uma operação
    fn sense(&mut self) -> Result<SilUpdate, SensorError> {
        let raw = self.read()?;
        let byte = self.to_byte_sil(&raw);
        Ok(SilUpdate::new(self.target_layer(), byte))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// PROCESSAMENTO (L5-L7) — Processadores e Atuadores
// ═══════════════════════════════════════════════════════════════════════════════

/// Erro de processador
#[derive(Debug, Clone, thiserror::Error)]
pub enum ProcessorError {
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Resource exhausted: {0}")]
    ResourceExhausted(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Backend unavailable: {0}")]
    BackendUnavailable(String),
}

/// Trait para processadores que executam computação em L5-L7.
///
/// # Camadas
///
/// | Camada | Função | Exemplo |
/// |:-------|:-------|:--------|
/// | L5 | Eletrônico | Inferência, computação |
/// | L6 | Psicomotor | Controle motor |
/// | L7 | Ambiental | Fusão sensorial |
pub trait Processor: SilComponent {
    /// Executa processamento sobre o estado
    fn execute(&mut self, state: &SilState) -> Result<SilState, ProcessorError>;

    /// Latência esperada em milissegundos
    fn latency_ms(&self) -> f32 {
        0.0
    }

    /// Processador suporta execução em batch?
    fn supports_batch(&self) -> bool {
        false
    }

    /// Executa em batch (se suportado)
    fn execute_batch(&mut self, states: &[SilState]) -> Result<Vec<SilState>, ProcessorError> {
        states.iter().map(|s| self.execute(s)).collect()
    }
}

/// Erro de atuador
#[derive(Debug, Clone, thiserror::Error)]
pub enum ActuatorError {
    #[error("Command failed: {0}")]
    CommandFailed(String),
    #[error("Actuator busy")]
    Busy,
    #[error("Actuator fault: {0}")]
    Fault(String),
    #[error("Out of range: {0}")]
    OutOfRange(String),
}

/// Status de atuador
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActuatorStatus {
    /// Pronto para receber comandos
    Ready,
    /// Executando comando
    Busy,
    /// Em erro (precisa reset)
    Fault,
    /// Desligado
    Off,
}

/// Trait para atuadores que executam ações físicas (L6).
///
/// # Exemplo
///
/// ```ignore
/// use sil_core::traits::{Actuator, ActuatorError, ActuatorStatus};
///
/// struct ServoMotor {
///     angle: f32,
/// }
///
/// impl Actuator for ServoMotor {
///     type Command = f32;  // Ângulo em graus
///
///     fn send(&mut self, angle: f32) -> Result<(), ActuatorError> {
///         if angle < 0.0 || angle > 180.0 {
///             return Err(ActuatorError::OutOfRange("0-180°".into()));
///         }
///         self.angle = angle;
///         Ok(())
///     }
///
///     fn status(&self) -> ActuatorStatus {
///         ActuatorStatus::Ready
///     }
/// }
/// ```
pub trait Actuator: SilComponent {
    /// Tipo de comando aceito
    type Command;

    /// Envia comando para o atuador
    fn send(&mut self, cmd: Self::Command) -> Result<(), ActuatorError>;

    /// Status atual do atuador
    fn status(&self) -> ActuatorStatus;

    /// Para movimento imediatamente (emergência)
    fn emergency_stop(&mut self) -> Result<(), ActuatorError> {
        Ok(())
    }

    /// Reseta atuador após falha
    fn reset(&mut self) -> Result<(), ActuatorError> {
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// INTERAÇÃO (L8-LA) — Rede e Governança
// ═══════════════════════════════════════════════════════════════════════════════

/// Erro de rede
#[derive(Debug, Clone, thiserror::Error)]
pub enum NetworkError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Peer not found: {0}")]
    PeerNotFound(String),
    #[error("Send failed: {0}")]
    SendFailed(String),
    #[error("Timeout")]
    Timeout,
    #[error("Protocol error: {0}")]
    Protocol(String),
}

/// Informações de um peer
#[derive(Debug, Clone)]
pub struct PeerInfo<Id> {
    pub id: Id,
    pub address: String,
    pub last_seen: Timestamp,
    pub latency_ms: Option<f32>,
}

/// Trait para nós de rede (L8-LA).
///
/// # Camadas
///
/// | Camada | Função | Implementação |
/// |:-------|:-------|:--------------|
/// | L8 | Cibernético | Feedback loops |
/// | L9 | Geopolítico | Soberania, borders |
/// | LA | Cosmopolítico | Ética, consenso |
pub trait NetworkNode: SilComponent {
    /// Tipo de identificador de peer
    type PeerId: Clone + Eq + Debug;

    /// Tipo de mensagem
    type Message: Clone + Debug;

    /// ID deste nó
    fn peer_id(&self) -> &Self::PeerId;

    /// Envia mensagem para peer
    fn send(&mut self, to: &Self::PeerId, msg: Self::Message) -> Result<(), NetworkError>;

    /// Recebe próxima mensagem (non-blocking)
    fn recv(&mut self) -> Option<(Self::PeerId, Self::Message)>;

    /// Lista peers conhecidos
    fn peers(&self) -> Vec<PeerInfo<Self::PeerId>>;

    /// Broadcast para todos os peers
    fn broadcast(&mut self, msg: Self::Message) -> Result<usize, NetworkError> {
        let peers: Vec<_> = self.peers().into_iter().map(|p| p.id).collect();
        let mut sent = 0;
        for peer in peers {
            if self.send(&peer, msg.clone()).is_ok() {
                sent += 1;
            }
        }
        Ok(sent)
    }

    /// Descobre novos peers
    fn discover(&mut self) -> Result<Vec<Self::PeerId>, NetworkError> {
        Ok(vec![])
    }
}

/// Erro de governança
#[derive(Debug, Clone, thiserror::Error)]
pub enum GovernanceError {
    #[error("Proposal rejected: {0}")]
    Rejected(String),
    #[error("Not authorized")]
    NotAuthorized,
    #[error("Quorum not reached")]
    NoQuorum,
    #[error("Already voted")]
    AlreadyVoted,
}

/// Voto em proposta
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vote {
    Yes,
    No,
    Abstain,
}

/// Status de proposta
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProposalStatus {
    Pending,
    Approved,
    Rejected,
    Expired,
}

/// Trait para governança distribuída (L9-LA).
pub trait Governor: SilComponent {
    /// Tipo de ID de proposta
    type ProposalId: Clone + Eq + Debug;

    /// Tipo de proposta
    type Proposal: Clone + Debug;

    /// Cria nova proposta
    fn propose(&mut self, proposal: Self::Proposal) -> Result<Self::ProposalId, GovernanceError>;

    /// Vota em proposta
    fn vote(&mut self, id: &Self::ProposalId, vote: Vote) -> Result<(), GovernanceError>;

    /// Status de proposta
    fn status(&self, id: &Self::ProposalId) -> Option<ProposalStatus>;

    /// Lista propostas ativas
    fn active_proposals(&self) -> Vec<Self::ProposalId>;
}

// ═══════════════════════════════════════════════════════════════════════════════
// EMERGÊNCIA (LB-LC) — Enxame e Quântico
// ═══════════════════════════════════════════════════════════════════════════════

/// Trait para agentes de enxame (LB - Sinérgico).
///
/// Implementa comportamento coletivo onde o todo é maior que a soma das partes.
pub trait SwarmAgent: SilComponent {
    /// Tipo de ID de nó
    type NodeId: Clone + Eq + Debug;

    /// Retorna IDs dos vizinhos conhecidos
    fn neighbors(&self) -> Vec<Self::NodeId>;

    /// Calcula próximo estado baseado no estado local e dos vizinhos
    ///
    /// # Argumentos
    ///
    /// * `local` - Estado atual deste agente
    /// * `neighbor_states` - Estados dos vizinhos (na mesma ordem de `neighbors()`)
    fn behavior(&mut self, local: &SilState, neighbor_states: &[SilState]) -> SilState;

    /// Distância para um vizinho (para ponderação)
    fn distance_to(&self, _neighbor: &Self::NodeId) -> f32 {
        1.0
    }
}

/// Trait para estados quânticos simulados (LC - Quântico).
///
/// Permite superposição e colapso de estados.
pub trait QuantumState: SilComponent {
    /// Cria superposição de estados (média ponderada)
    fn superpose(&self, states: &[SilState], weights: &[f32]) -> SilState;

    /// Colapsa superposição para estado definido
    fn collapse(&mut self, seed: u64) -> SilState;

    /// Medida de coerência (0 = decoerido, 1 = coerente)
    fn coherence(&self) -> f32;

    /// Estado está em superposição?
    fn is_superposed(&self) -> bool {
        self.coherence() > 0.5
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// META (LD-LF) — Fork, Entanglement, Collapse
// ═══════════════════════════════════════════════════════════════════════════════

/// Erro de merge
#[derive(Debug, Clone, thiserror::Error)]
pub enum MergeError {
    #[error("Conflicting states at layer {0}")]
    Conflict(LayerId),
    #[error("Incompatible versions")]
    IncompatibleVersion,
    #[error("Merge strategy failed: {0}")]
    StrategyFailed(String),
}

/// Trait para estados que podem ser forkados e merged (LD - Superposição).
///
/// Permite criar branches de estado para experimentação e depois reconciliar.
pub trait Forkable: Clone {
    /// Cria fork (cópia independente) do estado
    fn fork(&self) -> Self;

    /// Merge outro estado neste
    ///
    /// # Estratégias comuns
    ///
    /// - XOR: `self.layer ^= other.layer`
    /// - Max: `self.layer = max(self, other)`
    /// - Weighted: média ponderada
    fn merge(&mut self, other: &Self) -> Result<(), MergeError>;

    /// Verifica se dois estados divergiram
    fn has_diverged(&self, other: &Self) -> bool;
}

/// Erro de entanglement
#[derive(Debug, Clone, thiserror::Error)]
pub enum EntanglementError {
    #[error("Pair not found: {0}")]
    PairNotFound(String),
    #[error("Entanglement broken")]
    Broken,
    #[error("Sync failed: {0}")]
    SyncFailed(String),
    #[error("Already entangled")]
    AlreadyEntangled,
}

/// Trait para estados emaranhados (LE - Entanglement).
///
/// Estados emaranhados mantêm correlação mesmo quando distribuídos.
pub trait Entangled: SilComponent {
    /// Tipo de ID de par
    type PairId: Clone + Eq + Debug;

    /// Emaranha este estado com outro, retorna ID do par
    fn entangle(&mut self, other: &mut Self) -> Result<Self::PairId, EntanglementError>;

    /// Verifica se está emaranhado com um par específico
    fn is_entangled_with(&self, pair_id: &Self::PairId) -> bool;

    /// Sincroniza com par emaranhado
    fn sync(&mut self, pair_id: &Self::PairId) -> Result<(), EntanglementError>;

    /// Quebra entanglement
    fn disentangle(&mut self, pair_id: &Self::PairId) -> Result<(), EntanglementError>;

    /// Lista todos os pares ativos
    fn entangled_pairs(&self) -> Vec<Self::PairId>;
}

/// Erro de colapso
#[derive(Debug, Clone, thiserror::Error)]
pub enum CollapseError {
    #[error("Cannot collapse: {0}")]
    CannotCollapse(String),
    #[error("Checkpoint not found: {0}")]
    CheckpointNotFound(String),
    #[error("Restore failed: {0}")]
    RestoreFailed(String),
}

/// Trait para estados que podem colapsar (LF - Colapso).
///
/// Representa o fim de um ciclo — reset, checkpoint, EOF.
pub trait Collapsible: SilComponent {
    /// Tipo de ID de checkpoint
    type CheckpointId: Clone + Eq + Debug;

    /// Cria checkpoint do estado atual
    fn checkpoint(&mut self) -> Result<Self::CheckpointId, CollapseError>;

    /// Restaura estado de checkpoint
    fn restore(&mut self, id: &Self::CheckpointId) -> Result<(), CollapseError>;

    /// Colapsa estado (reset para inicial)
    fn collapse(&mut self) -> Result<SilState, CollapseError>;

    /// Estado precisa colapsar? (baseado em L(F))
    fn should_collapse(&self) -> bool;

    /// Lista checkpoints disponíveis
    fn checkpoints(&self) -> Vec<Self::CheckpointId>;
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRAIT DE CONVENIÊNCIA — Componente Completo
// ═══════════════════════════════════════════════════════════════════════════════

/// Resultado de processamento de componente
pub type ComponentResult<T> = Result<T, ComponentError>;

/// Erro genérico de componente
#[derive(Debug, Clone, thiserror::Error)]
pub enum ComponentError {
    #[error("Sensor error: {0}")]
    Sensor(#[from] SensorError),
    #[error("Processor error: {0}")]
    Processor(#[from] ProcessorError),
    #[error("Actuator error: {0}")]
    Actuator(#[from] ActuatorError),
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),
    #[error("Governance error: {0}")]
    Governance(#[from] GovernanceError),
    #[error("Merge error: {0}")]
    Merge(#[from] MergeError),
    #[error("Entanglement error: {0}")]
    Entanglement(#[from] EntanglementError),
    #[error("Collapse error: {0}")]
    Collapse(#[from] CollapseError),
    #[error("Other error: {0}")]
    Other(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sil_update_creation() {
        let update = SilUpdate::new(0, ByteSil::ONE);
        assert_eq!(update.layer, 0);
        assert_eq!(update.confidence, 1.0);
        assert!(update.timestamp > 0);
    }

    #[test]
    fn test_sil_update_with_confidence() {
        let update = SilUpdate::with_confidence(5, ByteSil::NULL, 0.75);
        assert_eq!(update.layer, 5);
        assert_eq!(update.confidence, 0.75);
    }

    #[test]
    fn test_sil_update_confidence_clamping() {
        let update = SilUpdate::with_confidence(0, ByteSil::ONE, 1.5);
        assert_eq!(update.confidence, 1.0);

        let update = SilUpdate::with_confidence(0, ByteSil::ONE, -0.5);
        assert_eq!(update.confidence, 0.0);
    }

    #[test]
    fn test_sil_update_apply() {
        let state = SilState::neutral();
        let update = SilUpdate::new(3, ByteSil::new(5, 8));
        let new_state = update.apply(&state);
        
        assert_eq!(new_state.layers[3].rho, 5);
        assert_eq!(new_state.layers[3].theta, 8);
    }
}
