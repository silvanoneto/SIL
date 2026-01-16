//! DMA (Direct Memory Access) para FPGA

use super::{FpgaError, FpgaResult};

/// Direção do DMA
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DmaDirection {
    /// Host → FPGA
    HostToDevice,
    /// FPGA → Host
    DeviceToHost,
    /// Bidirecional
    Bidirectional,
}

/// Buffer DMA
pub struct DmaBuffer {
    /// Dados
    data: Vec<u8>,
    /// Capacidade
    capacity: usize,
    /// Direção
    direction: DmaDirection,
    /// Bytes usados
    used: usize,
    /// Alinhado a página?
    page_aligned: bool,
}

impl DmaBuffer {
    /// Tamanho de página padrão (4KB)
    const PAGE_SIZE: usize = 4096;

    /// Cria novo buffer DMA
    pub fn new(size: usize, direction: DmaDirection) -> FpgaResult<Self> {
        if size == 0 {
            return Err(FpgaError::ConfigError("DMA buffer size cannot be zero".into()));
        }

        // Alinha para página
        let aligned_size = (size + Self::PAGE_SIZE - 1) & !(Self::PAGE_SIZE - 1);

        Ok(Self {
            data: vec![0u8; aligned_size],
            capacity: aligned_size,
            direction,
            used: 0,
            page_aligned: true,
        })
    }

    /// Cria buffer não-alinhado (para testes)
    pub fn unaligned(size: usize, direction: DmaDirection) -> FpgaResult<Self> {
        if size == 0 {
            return Err(FpgaError::ConfigError("DMA buffer size cannot be zero".into()));
        }

        Ok(Self {
            data: vec![0u8; size],
            capacity: size,
            direction,
            used: 0,
            page_aligned: false,
        })
    }

    /// Retorna direção
    pub fn direction(&self) -> DmaDirection {
        self.direction
    }

    /// Verifica se buffer está alinhado a página
    pub fn is_page_aligned(&self) -> bool {
        self.page_aligned
    }

    /// Retorna capacidade
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Retorna bytes usados
    pub fn used(&self) -> usize {
        self.used
    }

    /// Retorna bytes disponíveis
    pub fn available(&self) -> usize {
        self.capacity - self.used
    }

    /// Escreve dados no buffer
    pub fn write(&mut self, data: &[u8]) -> FpgaResult<usize> {
        if !matches!(self.direction, DmaDirection::HostToDevice | DmaDirection::Bidirectional) {
            return Err(FpgaError::DmaError("Buffer is read-only".into()));
        }

        let to_write = data.len().min(self.available());
        if to_write == 0 {
            return Ok(0);
        }

        self.data[self.used..self.used + to_write].copy_from_slice(&data[..to_write]);
        self.used += to_write;

        Ok(to_write)
    }

    /// Lê dados do buffer
    pub fn read(&self, offset: usize, len: usize) -> FpgaResult<&[u8]> {
        if !matches!(self.direction, DmaDirection::DeviceToHost | DmaDirection::Bidirectional) {
            return Err(FpgaError::DmaError("Buffer is write-only".into()));
        }

        if offset + len > self.used {
            return Err(FpgaError::DmaError(format!(
                "Read out of bounds: offset {} + len {} > used {}",
                offset, len, self.used
            )));
        }

        Ok(&self.data[offset..offset + len])
    }

    /// Retorna slice do buffer usado
    pub fn as_slice(&self) -> &[u8] {
        &self.data[..self.used]
    }

    /// Retorna slice mutável do buffer
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.data[..self.used]
    }

    /// Retorna ponteiro para dados (para FFI)
    pub fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    /// Retorna ponteiro mutável (para FFI)
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.data.as_mut_ptr()
    }

    /// Reset do buffer
    pub fn reset(&mut self) {
        self.used = 0;
    }

    /// Zera buffer
    pub fn zero(&mut self) {
        self.data.fill(0);
        self.used = 0;
    }

    /// Define bytes usados (após DMA do dispositivo)
    pub fn set_used(&mut self, used: usize) -> FpgaResult<()> {
        if used > self.capacity {
            return Err(FpgaError::DmaError(format!(
                "Used {} exceeds capacity {}",
                used, self.capacity
            )));
        }
        self.used = used;
        Ok(())
    }
}

/// Descritor de transferência DMA
#[derive(Debug, Clone)]
pub struct DmaDescriptor {
    /// Endereço fonte
    pub src_addr: u64,
    /// Endereço destino
    pub dst_addr: u64,
    /// Tamanho em bytes
    pub size: u32,
    /// Flags
    pub flags: DmaFlags,
}

/// Flags de DMA
#[derive(Debug, Clone, Copy, Default)]
pub struct DmaFlags {
    /// Gera interrupção ao completar
    pub interrupt_on_complete: bool,
    /// Próximo descritor encadeado
    pub chained: bool,
    /// Último descritor da cadeia
    pub last: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dma_buffer_creation() {
        let buf = DmaBuffer::new(1024, DmaDirection::Bidirectional).unwrap();
        // Deve estar alinhado para 4KB
        assert!(buf.capacity() >= 1024);
        assert_eq!(buf.capacity() % DmaBuffer::PAGE_SIZE, 0);
    }

    #[test]
    fn test_dma_buffer_write() {
        let mut buf = DmaBuffer::new(1024, DmaDirection::HostToDevice).unwrap();

        let data = [1u8, 2, 3, 4, 5];
        let written = buf.write(&data).unwrap();

        assert_eq!(written, 5);
        assert_eq!(buf.used(), 5);
    }

    #[test]
    fn test_dma_buffer_read() {
        let mut buf = DmaBuffer::new(1024, DmaDirection::Bidirectional).unwrap();

        let data = [1u8, 2, 3, 4, 5];
        buf.write(&data).unwrap();

        let read = buf.read(0, 5).unwrap();
        assert_eq!(read, &data);
    }

    #[test]
    fn test_dma_direction_enforcement() {
        let buf = DmaBuffer::new(1024, DmaDirection::HostToDevice).unwrap();

        // Tentar ler de buffer write-only deve falhar
        let result = buf.read(0, 1);
        assert!(result.is_err());
    }

    #[test]
    fn test_dma_reset() {
        let mut buf = DmaBuffer::new(1024, DmaDirection::Bidirectional).unwrap();

        buf.write(&[1, 2, 3]).unwrap();
        assert_eq!(buf.used(), 3);

        buf.reset();
        assert_eq!(buf.used(), 0);
    }
}
