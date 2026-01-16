//! # 📁 I/O — Entrada e Saída de Dados SIL
//!
//! Módulo para trabalhar com I/O de arquivos e streams usando ByteSil.
//!
//! ## Exemplo
//!
//! ```ignore
//! use sil_core::io::{SilReader, SilWriter, SilPipeline};
//! use sil_core::io::transforms::{Xor, Rotate, Scale};
//!
//! // Ler, transformar e escrever
//! let input = SilReader::from_file("input.txt")?;
//! let pipeline = SilPipeline::new()
//!     .then(Rotate(4))
//!     .then(Scale(1))
//!     .then(Xor(0x5A));
//! let output = pipeline.process(&input);
//! SilWriter::to_file("output.bin", &output)?;
//! ```

// Permitir código não usado — API pública pode não ser usada internamente
#![allow(dead_code)]

mod reader;
mod writer;
pub mod pipeline;
pub mod transforms;
pub mod jsonl;
pub mod jsil;
pub mod streaming;

pub use reader::SilReader;
pub use writer::SilWriter;
pub use pipeline::SilPipeline;
pub use streaming::{JsilStreamReader, JsilStreamWriter};

use crate::state::ByteSil;
use std::io::{self, Read, Write};
use std::path::Path;
use std::fs::File;

/// Buffer de dados SIL
#[derive(Debug, Clone, Default)]
pub struct SilBuffer {
    data: Vec<ByteSil>,
}

impl SilBuffer {
    /// Cria buffer vazio
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }
    
    /// Cria buffer com capacidade
    pub fn with_capacity(capacity: usize) -> Self {
        Self { data: Vec::with_capacity(capacity) }
    }
    
    /// Cria buffer a partir de bytes
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            data: bytes.iter().map(|&b| ByteSil::from_u8(b)).collect()
        }
    }
    
    /// Cria buffer a partir de string
    pub fn from_str(s: &str) -> Self {
        Self::from_bytes(s.as_bytes())
    }
    
    /// Cria buffer a partir de Vec<ByteSil>
    pub fn from_sil(data: Vec<ByteSil>) -> Self {
        Self { data }
    }
    
    /// Converte para bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        self.data.iter().map(|b| b.to_u8()).collect()
    }
    
    /// Converte para string (lossy)
    pub fn to_string_lossy(&self) -> String {
        String::from_utf8_lossy(&self.to_bytes()).to_string()
    }
    
    /// Referência aos dados
    pub fn data(&self) -> &[ByteSil] {
        &self.data
    }
    
    /// Referência mutável aos dados
    pub fn data_mut(&mut self) -> &mut Vec<ByteSil> {
        &mut self.data
    }
    
    /// Tamanho do buffer
    pub fn len(&self) -> usize {
        self.data.len()
    }
    
    /// Buffer vazio?
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    
    /// Adiciona byte
    pub fn push(&mut self, byte: ByteSil) {
        self.data.push(byte);
    }
    
    /// Adiciona bytes
    pub fn extend(&mut self, bytes: impl IntoIterator<Item = ByteSil>) {
        self.data.extend(bytes);
    }
    
    /// Limpa buffer
    pub fn clear(&mut self) {
        self.data.clear();
    }
    
    /// Itera sobre os bytes
    pub fn iter(&self) -> impl Iterator<Item = &ByteSil> {
        self.data.iter()
    }
    
    /// Itera mutável sobre os bytes
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut ByteSil> {
        self.data.iter_mut()
    }
}

impl IntoIterator for SilBuffer {
    type Item = ByteSil;
    type IntoIter = std::vec::IntoIter<ByteSil>;
    
    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl<'a> IntoIterator for &'a SilBuffer {
    type Item = &'a ByteSil;
    type IntoIter = std::slice::Iter<'a, ByteSil>;
    
    fn into_iter(self) -> Self::IntoIter {
        self.data.iter()
    }
}

impl FromIterator<ByteSil> for SilBuffer {
    fn from_iter<T: IntoIterator<Item = ByteSil>>(iter: T) -> Self {
        Self { data: iter.into_iter().collect() }
    }
}

impl std::ops::Index<usize> for SilBuffer {
    type Output = ByteSil;
    
    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl std::ops::IndexMut<usize> for SilBuffer {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

// =============================================================================
// Funções de conveniência
// =============================================================================

/// Lê arquivo e retorna buffer SIL
pub fn read_file<P: AsRef<Path>>(path: P) -> io::Result<SilBuffer> {
    let mut file = File::open(path)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    Ok(SilBuffer::from_bytes(&bytes))
}

/// Escreve buffer SIL em arquivo
pub fn write_file<P: AsRef<Path>>(path: P, buffer: &SilBuffer) -> io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(&buffer.to_bytes())?;
    Ok(())
}

/// Aplica XOR em buffer
pub fn xor(buffer: &SilBuffer, key: u8) -> SilBuffer {
    let key_sil = ByteSil::from_u8(key);
    buffer.iter()
        .map(|b| ByteSil::from_u8(b.to_u8() ^ key_sil.to_u8()))
        .collect()
}

/// Aplica rotação de fase em buffer
pub fn rotate(buffer: &SilBuffer, delta: u8) -> SilBuffer {
    buffer.iter()
        .map(|b| ByteSil::new(b.rho, (b.theta + delta) % 16))
        .collect()
}

/// Aplica escala de magnitude em buffer
pub fn scale(buffer: &SilBuffer, delta: i8) -> SilBuffer {
    buffer.iter()
        .map(|b| {
            let new_rho = (b.rho as i16 + delta as i16)
                .clamp(ByteSil::RHO_MIN as i16, ByteSil::RHO_MAX as i16) as i8;
            ByteSil::new(new_rho, b.theta)
        })
        .collect()
}

// =============================================================================
// Testes
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_buffer_from_bytes() {
        let buffer = SilBuffer::from_bytes(b"Hello");
        assert_eq!(buffer.len(), 5);
        assert_eq!(buffer.to_bytes(), b"Hello");
    }
    
    #[test]
    fn test_buffer_from_str() {
        let buffer = SilBuffer::from_str("SIL");
        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.to_string_lossy(), "SIL");
    }
    
    #[test]
    fn test_xor_roundtrip() {
        let original = SilBuffer::from_str("Hello SIL");
        let encrypted = xor(&original, 0x5A);
        let decrypted = xor(&encrypted, 0x5A);
        assert_eq!(decrypted.to_bytes(), original.to_bytes());
    }
    
    #[test]
    fn test_rotate() {
        let buffer = SilBuffer::from_bytes(&[0x48]); // 'H'
        let rotated = rotate(&buffer, 4);
        assert_ne!(rotated[0].theta, buffer[0].theta);
    }
    
    #[test]
    fn test_collect() {
        let buffer: SilBuffer = vec![
            ByteSil::from_u8(0x48),
            ByteSil::from_u8(0x69),
        ].into_iter().collect();
        assert_eq!(buffer.len(), 2);
    }
}
