//! Reader — Leitura de dados SIL

use super::SilBuffer;
use crate::state::ByteSil;
use std::io::{self, BufRead, BufReader, Read};
use std::path::Path;
use std::fs::File;

/// Leitor de dados SIL
pub struct SilReader<R: Read> {
    inner: BufReader<R>,
    buffer: Vec<u8>,
}

impl<R: Read> SilReader<R> {
    /// Cria leitor a partir de Read
    pub fn new(reader: R) -> Self {
        Self {
            inner: BufReader::new(reader),
            buffer: Vec::new(),
        }
    }
    
    /// Lê todos os bytes e retorna buffer SIL
    pub fn read_all(mut self) -> io::Result<SilBuffer> {
        self.inner.read_to_end(&mut self.buffer)?;
        Ok(SilBuffer::from_bytes(&self.buffer))
    }
    
    /// Lê até N bytes
    pub fn read_n(&mut self, n: usize) -> io::Result<SilBuffer> {
        let mut buf = vec![0u8; n];
        let read = self.inner.read(&mut buf)?;
        buf.truncate(read);
        Ok(SilBuffer::from_bytes(&buf))
    }
    
    /// Lê uma linha
    pub fn read_line(&mut self) -> io::Result<Option<SilBuffer>> {
        self.buffer.clear();
        let read = self.inner.read_until(b'\n', &mut self.buffer)?;
        if read == 0 {
            Ok(None)
        } else {
            Ok(Some(SilBuffer::from_bytes(&self.buffer)))
        }
    }
}

impl SilReader<File> {
    /// Abre arquivo para leitura
    pub fn from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = File::open(path)?;
        Ok(Self::new(file))
    }
}

impl SilReader<&[u8]> {
    /// Cria leitor a partir de slice
    pub fn from_bytes(bytes: &[u8]) -> SilReader<&[u8]> {
        SilReader::new(bytes)
    }
}

/// Itera sobre ByteSil de um reader
pub struct SilReadIter<R: Read> {
    inner: BufReader<R>,
}

impl<R: Read> SilReadIter<R> {
    pub fn new(reader: R) -> Self {
        Self {
            inner: BufReader::new(reader),
        }
    }
}

impl<R: Read> Iterator for SilReadIter<R> {
    type Item = io::Result<ByteSil>;
    
    fn next(&mut self) -> Option<Self::Item> {
        let mut buf = [0u8; 1];
        match self.inner.read(&mut buf) {
            Ok(0) => None,
            Ok(_) => Some(Ok(ByteSil::from_u8(buf[0]))),
            Err(e) => Some(Err(e)),
        }
    }
}

// =============================================================================
// Funções de conveniência
// =============================================================================

/// Lê arquivo inteiro como SilBuffer
pub fn read_file<P: AsRef<Path>>(path: P) -> io::Result<SilBuffer> {
    SilReader::from_file(path)?.read_all()
}

/// Lê string como SilBuffer
pub fn read_string(s: &str) -> SilBuffer {
    SilBuffer::from_str(s)
}

/// Lê bytes como SilBuffer
pub fn read_bytes(bytes: &[u8]) -> SilBuffer {
    SilBuffer::from_bytes(bytes)
}

// =============================================================================
// Testes
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_read_bytes() {
        let reader = SilReader::from_bytes(b"Hello");
        let buffer = reader.read_all().unwrap();
        assert_eq!(buffer.to_string_lossy(), "Hello");
    }
    
    #[test]
    fn test_read_iter() {
        let iter = SilReadIter::new(&b"Hi"[..]);
        let bytes: Vec<_> = iter.filter_map(|r| r.ok()).collect();
        assert_eq!(bytes.len(), 2);
        assert_eq!(bytes[0].to_u8(), b'H');
        assert_eq!(bytes[1].to_u8(), b'i');
    }
}
