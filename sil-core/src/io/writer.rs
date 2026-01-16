//! Writer — Escrita de dados SIL

use super::SilBuffer;
use crate::state::ByteSil;
use std::io::{self, BufWriter, Write};
use std::path::Path;
use std::fs::File;

/// Escritor de dados SIL
pub struct SilWriter<W: Write> {
    inner: BufWriter<W>,
}

impl<W: Write> SilWriter<W> {
    /// Cria escritor a partir de Write
    pub fn new(writer: W) -> Self {
        Self {
            inner: BufWriter::new(writer),
        }
    }
    
    /// Escreve buffer SIL
    pub fn write_buffer(&mut self, buffer: &SilBuffer) -> io::Result<usize> {
        let bytes = buffer.to_bytes();
        self.inner.write_all(&bytes)?;
        Ok(bytes.len())
    }
    
    /// Escreve único ByteSil
    pub fn write_byte(&mut self, byte: ByteSil) -> io::Result<()> {
        self.inner.write_all(&[byte.to_u8()])
    }
    
    /// Escreve bytes raw
    pub fn write_raw(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.inner.write_all(bytes)
    }
    
    /// Flush do buffer interno
    pub fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
    
    /// Finaliza escrita e retorna writer interno
    pub fn into_inner(self) -> io::Result<W> {
        self.inner.into_inner().map_err(|e| e.into_error())
    }
}

impl SilWriter<File> {
    /// Cria arquivo para escrita
    pub fn to_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = File::create(path)?;
        Ok(Self::new(file))
    }
}

impl SilWriter<Vec<u8>> {
    /// Cria escritor em memória
    pub fn to_vec() -> Self {
        Self::new(Vec::new())
    }
    
    /// Retorna bytes escritos
    pub fn into_bytes(self) -> io::Result<Vec<u8>> {
        self.into_inner()
    }
}

// =============================================================================
// Funções de conveniência
// =============================================================================

/// Escreve SilBuffer em arquivo
pub fn write_file<P: AsRef<Path>>(path: P, buffer: &SilBuffer) -> io::Result<()> {
    let mut writer = SilWriter::to_file(path)?;
    writer.write_buffer(buffer)?;
    writer.flush()
}

/// Converte SilBuffer para bytes
pub fn to_bytes(buffer: &SilBuffer) -> Vec<u8> {
    buffer.to_bytes()
}

/// Converte SilBuffer para string (lossy)
pub fn to_string(buffer: &SilBuffer) -> String {
    buffer.to_string_lossy()
}

// =============================================================================
// Testes
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_write_to_vec() {
        let buffer = SilBuffer::from_str("Hello");
        let mut writer = SilWriter::to_vec();
        writer.write_buffer(&buffer).unwrap();
        let bytes = writer.into_bytes().unwrap();
        assert_eq!(bytes, b"Hello");
    }
    
    #[test]
    fn test_write_bytes() {
        let mut writer = SilWriter::to_vec();
        writer.write_byte(ByteSil::from_u8(b'H')).unwrap();
        writer.write_byte(ByteSil::from_u8(b'i')).unwrap();
        let bytes = writer.into_bytes().unwrap();
        assert_eq!(bytes, b"Hi");
    }
}
