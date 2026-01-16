//! Transport abstraction — UDP e TCP
//!
//! UDP para discovery e mensagens pequenas.
//! TCP futuro para streams e mensagens grandes.

use crate::{Message, NetworkError, SilMessage};
use std::net::{SocketAddr, UdpSocket, Ipv4Addr};
use std::time::Duration;
use std::io;

/// Constantes de rede
pub const DEFAULT_PORT: u16 = 21000;
pub const MULTICAST_ADDR: Ipv4Addr = Ipv4Addr::new(239, 21, 0, 1);
pub const MULTICAST_PORT: u16 = 21001;
pub const MAX_PACKET_SIZE: usize = 1500;

/// Trait de transporte (síncrono)
pub trait Transport {
    /// Envia mensagem binária para destino
    fn send(&mut self, msg: &Message, dest: SocketAddr) -> Result<(), NetworkError>;

    /// Recebe próxima mensagem (blocking com timeout)
    fn recv(&mut self) -> Result<Option<(Message, SocketAddr)>, NetworkError>;

    /// Broadcast para multicast
    fn broadcast(&mut self, msg: &Message) -> Result<(), NetworkError>;

    /// Envia mensagem SIL serializada
    fn send_sil(&mut self, msg: &SilMessage, dest: SocketAddr) -> Result<(), NetworkError>;

    /// Recebe mensagem SIL
    fn recv_sil(&mut self) -> Result<Option<(SilMessage, SocketAddr)>, NetworkError>;
}

/// Transporte UDP
#[derive(Debug)]
pub struct UdpTransport {
    socket: UdpSocket,
    multicast_socket: Option<UdpSocket>,
    buffer: [u8; MAX_PACKET_SIZE],
}

impl UdpTransport {
    /// Cria novo transporte UDP
    pub fn bind(port: u16) -> Result<Self, NetworkError> {
        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        let socket = UdpSocket::bind(addr)?;
        socket.set_read_timeout(Some(Duration::from_millis(100)))?;
        socket.set_nonblocking(false)?;

        Ok(Self {
            socket,
            multicast_socket: None,
            buffer: [0u8; MAX_PACKET_SIZE],
        })
    }

    /// Liga socket multicast para discovery
    pub fn join_multicast(&mut self) -> Result<(), NetworkError> {
        let _mcast_addr = SocketAddr::from((MULTICAST_ADDR, MULTICAST_PORT));

        // Cria socket separado para multicast
        let socket = UdpSocket::bind(SocketAddr::from(([0, 0, 0, 0], MULTICAST_PORT)))?;
        socket.join_multicast_v4(&MULTICAST_ADDR, &Ipv4Addr::UNSPECIFIED)?;
        socket.set_read_timeout(Some(Duration::from_millis(100)))?;

        self.multicast_socket = Some(socket);
        Ok(())
    }

    /// Endereço local do socket
    pub fn local_addr(&self) -> Result<SocketAddr, NetworkError> {
        Ok(self.socket.local_addr()?)
    }

    /// Define timeout de leitura
    pub fn set_timeout(&self, timeout: Duration) -> Result<(), NetworkError> {
        self.socket.set_read_timeout(Some(timeout))?;
        Ok(())
    }
}

impl Transport for UdpTransport {
    fn send(&mut self, msg: &Message, dest: SocketAddr) -> Result<(), NetworkError> {
        // Formato wire: [type:1][sender:8][payload:16][sig:64] = 89 bytes
        let mut buf = [0u8; 89];
        buf[0] = msg.msg_type as u8;
        buf[1..9].copy_from_slice(&msg.sender_id.to_le_bytes());
        buf[9..25].copy_from_slice(&msg.payload);
        buf[25..89].copy_from_slice(&msg.signature);

        self.socket
            .send_to(&buf, dest)
            .map_err(|e| NetworkError::Transport(e.to_string()))?;
        Ok(())
    }

    fn recv(&mut self) -> Result<Option<(Message, SocketAddr)>, NetworkError> {
        match self.socket.recv_from(&mut self.buffer) {
            Ok((len, addr)) => {
                if len < 89 {
                    return Ok(None); // Pacote muito pequeno
                }

                let msg_type = match self.buffer[0] {
                    0x01 => crate::MessageType::Data,
                    0x02 => crate::MessageType::Control,
                    0x03 => crate::MessageType::Discovery,
                    _ => return Ok(None), // Tipo desconhecido
                };

                let sender_id = u64::from_le_bytes(self.buffer[1..9].try_into().unwrap());
                let mut payload = [0u8; 16];
                payload.copy_from_slice(&self.buffer[9..25]);
                let mut signature = [0u8; 64];
                signature.copy_from_slice(&self.buffer[25..89]);

                Ok(Some((
                    Message {
                        msg_type,
                        sender_id,
                        payload,
                        signature,
                    },
                    addr,
                )))
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => Ok(None),
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => Ok(None),
            Err(e) => Err(NetworkError::Transport(e.to_string())),
        }
    }

    fn broadcast(&mut self, msg: &Message) -> Result<(), NetworkError> {
        let dest = SocketAddr::from((MULTICAST_ADDR, MULTICAST_PORT));
        self.send(msg, dest)
    }

    fn send_sil(&mut self, msg: &SilMessage, dest: SocketAddr) -> Result<(), NetworkError> {
        let json = serde_json::to_vec(msg)
            .map_err(|e| NetworkError::Serialization(e.to_string()))?;

        if json.len() > MAX_PACKET_SIZE {
            return Err(NetworkError::Transport("Message too large".into()));
        }

        self.socket
            .send_to(&json, dest)
            .map_err(|e| NetworkError::Transport(e.to_string()))?;
        Ok(())
    }

    fn recv_sil(&mut self) -> Result<Option<(SilMessage, SocketAddr)>, NetworkError> {
        match self.socket.recv_from(&mut self.buffer) {
            Ok((len, addr)) => {
                let msg: SilMessage = serde_json::from_slice(&self.buffer[..len])
                    .map_err(|e| NetworkError::Serialization(e.to_string()))?;
                Ok(Some((msg, addr)))
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => Ok(None),
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => Ok(None),
            Err(e) => Err(NetworkError::Transport(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MessageType;

    #[test]
    fn udp_transport_bind() {
        let transport = UdpTransport::bind(0); // Porta aleatória
        assert!(transport.is_ok());
    }

    #[test]
    fn udp_transport_local_addr() {
        let transport = UdpTransport::bind(0).unwrap();
        let addr = transport.local_addr().unwrap();
        assert_ne!(addr.port(), 0);
    }
}
