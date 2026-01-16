//! Mesh topology — rede descentralizada
//!
//! MeshNode orquestra múltiplas conexões em topologia mesh.

use crate::peer::PeerId;
use crate::{NetworkError, SilNode, NodeConfig};

use std::net::SocketAddr;

/// Nó mesh que gerencia topologia
pub struct MeshNode {
    /// Nó SIL subjacente
    node: SilNode,
    /// Peers bootstrap (sempre tentar reconectar)
    bootstrap_peers: Vec<SocketAddr>,
}

impl MeshNode {
    /// Cria novo nó mesh
    pub fn new(config: NodeConfig) -> Result<Self, NetworkError> {
        Ok(Self {
            node: SilNode::new(config)?,
            bootstrap_peers: Vec::new(),
        })
    }

    /// Adiciona peer bootstrap
    pub fn add_bootstrap(&mut self, addr: SocketAddr) {
        if !self.bootstrap_peers.contains(&addr) {
            self.bootstrap_peers.push(addr);
        }
    }

    /// Conecta aos peers bootstrap
    pub fn connect_bootstrap(&mut self) -> Result<usize, NetworkError> {
        let mut connected = 0;
        for addr in self.bootstrap_peers.clone() {
            if self.node.connect(addr).is_ok() {
                connected += 1;
            }
        }
        Ok(connected)
    }

    /// Descobre peers via multicast
    pub fn discover(&mut self) -> Result<Vec<PeerId>, NetworkError> {
        use sil_core::traits::NetworkNode;
        self.node.discover().map_err(|e| NetworkError::Transport(e.to_string()))
    }

    /// Executa manutenção da mesh
    pub fn maintain(&mut self) -> Result<(), NetworkError> {
        // Poll mensagens
        self.node.poll()?;

        // Reconecta a bootstrap peers se necessário
        let connected = self.node.connected_count();
        if connected < self.bootstrap_peers.len() {
            let _ = self.connect_bootstrap();
        }

        Ok(())
    }

    /// Número de peers conectados
    pub fn peer_count(&self) -> usize {
        use sil_core::traits::NetworkNode;
        self.node.peers().len()
    }

    /// ID do nó
    pub fn id(&self) -> PeerId {
        self.node.id()
    }

    /// Acesso ao nó subjacente
    pub fn node(&self) -> &SilNode {
        &self.node
    }

    /// Acesso mutável ao nó subjacente
    pub fn node_mut(&mut self) -> &mut SilNode {
        &mut self.node
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mesh_node_creation() {
        let config = NodeConfig::default().with_port(0);
        let mesh = MeshNode::new(config);
        assert!(mesh.is_ok());
    }

    #[test]
    fn mesh_add_bootstrap() {
        let config = NodeConfig::default().with_port(0);
        let mut mesh = MeshNode::new(config).unwrap();

        let addr: SocketAddr = "127.0.0.1:21000".parse().unwrap();
        mesh.add_bootstrap(addr);
        assert_eq!(mesh.bootstrap_peers.len(), 1);

        // Não duplica
        mesh.add_bootstrap(addr);
        assert_eq!(mesh.bootstrap_peers.len(), 1);
    }
}
