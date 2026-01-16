//! Network message format
//!
//! Mensagens SIL são tipadas e assinadas com Ed25519.

use sil_core::SilState;
use serde::{Deserialize, Serialize};

/// Tipos de mensagem de rede
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum MessageType {
    /// Dados (SilState)
    Data = 0x01,
    /// Controle (ping, pong, etc.)
    Control = 0x02,
    /// Discovery (hello, announce)
    Discovery = 0x03,
    /// Gossip (propagação de estado)
    Gossip = 0x04,
    /// Requisição de estado
    Request = 0x05,
    /// Resposta a requisição
    Response = 0x06,
}

/// Mensagem de controle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ControlMessage {
    /// Ping (espera Pong)
    Ping { nonce: u64 },
    /// Resposta a Ping
    Pong { nonce: u64 },
    /// Anuncio de saída
    Goodbye,
}

/// Mensagem de discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscoveryMessage {
    /// Hello (primeiro contato)
    Hello { version: u8, capabilities: u8 },
    /// Anúncio de existência
    Announce { port: u16 },
    /// Requisição de lista de peers
    GetPeers,
    /// Resposta com lista de peers
    Peers { addrs: Vec<String> },
}

/// Payload tipado de mensagem SIL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SilPayload {
    /// Estado completo (128 bits)
    State(Vec<u8>),
    /// Atualização parcial (camada + byte)
    Update { layer: u8, byte: u8 },
    /// Controle
    Control(ControlMessage),
    /// Discovery
    Discovery(DiscoveryMessage),
    /// Dados binários
    Raw(Vec<u8>),
}

/// Mensagem SIL completa (usada no trait NetworkNode)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SilMessage {
    /// Tipo da mensagem
    pub msg_type: MessageType,
    /// Payload
    pub payload: SilPayload,
    /// Timestamp (nanos desde epoch)
    pub timestamp: u64,
    /// Sequence number para ordenação
    pub seq: u64,
}

impl SilMessage {
    /// Cria mensagem de estado
    pub fn state(state: &SilState) -> Self {
        let mut bytes = Vec::with_capacity(16);
        for i in 0..16 {
            bytes.push(state.get(i).to_u8());
        }
        Self {
            msg_type: MessageType::Data,
            payload: SilPayload::State(bytes),
            timestamp: now_nanos(),
            seq: 0,
        }
    }

    /// Cria mensagem de atualização parcial
    pub fn update(layer: u8, byte: u8) -> Self {
        Self {
            msg_type: MessageType::Data,
            payload: SilPayload::Update { layer, byte },
            timestamp: now_nanos(),
            seq: 0,
        }
    }

    /// Cria ping
    pub fn ping() -> Self {
        Self {
            msg_type: MessageType::Control,
            payload: SilPayload::Control(ControlMessage::Ping {
                nonce: rand::random(),
            }),
            timestamp: now_nanos(),
            seq: 0,
        }
    }

    /// Cria pong em resposta a ping
    pub fn pong(nonce: u64) -> Self {
        Self {
            msg_type: MessageType::Control,
            payload: SilPayload::Control(ControlMessage::Pong { nonce }),
            timestamp: now_nanos(),
            seq: 0,
        }
    }

    /// Cria hello para discovery
    pub fn hello() -> Self {
        Self {
            msg_type: MessageType::Discovery,
            payload: SilPayload::Discovery(DiscoveryMessage::Hello {
                version: 1,
                capabilities: 0xFF,
            }),
            timestamp: now_nanos(),
            seq: 0,
        }
    }

    /// Extrai estado do payload (se for mensagem de estado)
    pub fn to_state(&self) -> Option<SilState> {
        match &self.payload {
            SilPayload::State(bytes) if bytes.len() == 16 => {
                let mut state = SilState::default();
                for (i, &b) in bytes.iter().enumerate() {
                    state.set_layer(i, sil_core::ByteSil::from_u8(b));
                }
                Some(state)
            }
            _ => None,
        }
    }

    /// Define sequence number
    pub fn with_seq(mut self, seq: u64) -> Self {
        self.seq = seq;
        self
    }
}

/// Mensagem binária (formato wire) para compatibilidade
pub struct Message {
    pub msg_type: MessageType,
    pub sender_id: u64,
    pub payload: [u8; 16],
    pub signature: [u8; 64],
}

impl Message {
    /// Cria mensagem binária a partir de SilState
    pub fn from_state(state: &SilState, sender_id: u64, msg_type: MessageType) -> Self {
        let mut payload = [0u8; 16];
        for (i, byte_sil) in (0..16).map(|i| state.get(i)).enumerate() {
            payload[i] = byte_sil.to_u8();
        }

        Self {
            msg_type,
            sender_id,
            payload,
            signature: [0u8; 64], // Preenchido por signer
        }
    }

    /// Converte para SilMessage
    pub fn to_sil_message(&self) -> SilMessage {
        SilMessage {
            msg_type: self.msg_type,
            payload: SilPayload::State(self.payload.to_vec()),
            timestamp: now_nanos(),
            seq: 0,
        }
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
    fn sil_message_state_roundtrip() {
        let state = SilState::default();
        let msg = SilMessage::state(&state);
        let recovered = msg.to_state().unwrap();
        assert_eq!(state, recovered);
    }

    #[test]
    fn sil_message_ping_pong() {
        let ping = SilMessage::ping();
        assert_eq!(ping.msg_type, MessageType::Control);

        if let SilPayload::Control(ControlMessage::Ping { nonce }) = ping.payload {
            let pong = SilMessage::pong(nonce);
            if let SilPayload::Control(ControlMessage::Pong { nonce: n2 }) = pong.payload {
                assert_eq!(nonce, n2);
            } else {
                panic!("Expected Pong");
            }
        } else {
            panic!("Expected Ping");
        }
    }
}
