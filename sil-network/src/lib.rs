//! # L8 Cibernético — Networking P2P
//!
//! Camada de rede: mesh topology, discovery, mensagens assinadas.
//!
//! ## Responsabilidades
//! - Transporte UDP com assinatura Ed25519
//! - Discovery via multicast
//! - Peer registry e routing
//! - SilState distribution
//!
//! ## Uso
//!
//! ```rust,no_run
//! use sil_network::{SilNode, NodeConfig};
//! use sil_core::traits::NetworkNode;
//!
//! let config = NodeConfig::default();
//! let mut node = SilNode::new(config).unwrap();
//!
//! // Enviar estado para peer
//! let state = sil_core::SilState::default();
//! node.broadcast_state(&state).unwrap();
//! ```

use sil_core::traits::NetworkError as CoreNetworkError;

pub mod message;
pub mod transport;
pub mod peer;
pub mod mesh;
mod node;
pub mod control_mode;
pub mod pid;
pub mod cybernetic;

pub use message::{Message, MessageType, SilMessage};
pub use transport::{Transport, UdpTransport};
pub use peer::{PeerRegistry, PeerId};
pub use mesh::MeshNode;
pub use node::{SilNode, NodeConfig};
pub use control_mode::ControlMode;
pub use pid::{PidController, PidConfig, FeedbackLoop};
pub use cybernetic::{CyberneticState, ControlStats, CYBERNETIC_LAYER};

// Alias para compatibilidade com CLI
pub type SilNodeConfig = NodeConfig;

/// Erros específicos de sil-network
#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    #[error("Transport error: {0}")]
    Transport(String),
    #[error("Signature error: {0}")]
    Signature(String),
    #[error("Peer error: {0}")]
    Peer(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Timeout")]
    Timeout,
}

impl From<NetworkError> for CoreNetworkError {
    fn from(e: NetworkError) -> Self {
        match e {
            NetworkError::Transport(s) => CoreNetworkError::ConnectionFailed(s),
            NetworkError::Signature(s) => CoreNetworkError::Protocol(s),
            NetworkError::Peer(s) => CoreNetworkError::PeerNotFound(s),
            NetworkError::Io(e) => CoreNetworkError::ConnectionFailed(e.to_string()),
            NetworkError::Serialization(s) => CoreNetworkError::Protocol(s),
            NetworkError::Timeout => CoreNetworkError::Timeout,
        }
    }
}
