//! Erros do módulo FPGA

use std::fmt;

/// Resultado de operação FPGA
pub type FpgaResult<T> = Result<T, FpgaError>;

/// Erros do FPGA
#[derive(Debug, Clone)]
pub enum FpgaError {
    /// Dispositivo não encontrado
    DeviceNotFound(String),
    /// Falha ao abrir dispositivo
    DeviceOpenFailed(String),
    /// Bitstream inválido
    InvalidBitstream(String),
    /// Falha ao carregar bitstream
    BitstreamLoadFailed(String),
    /// Timeout na operação
    Timeout(String),
    /// Erro de DMA
    DmaError(String),
    /// Erro de configuração
    ConfigError(String),
    /// Recurso não disponível
    ResourceUnavailable(String),
    /// Operação não suportada
    Unsupported(String),
    /// Erro de I/O
    IoError(String),
    /// Erro do driver
    DriverError(String),
    /// Simulador: operação não implementada
    SimulatorError(String),
}

impl fmt::Display for FpgaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DeviceNotFound(msg) => write!(f, "FPGA device not found: {}", msg),
            Self::DeviceOpenFailed(msg) => write!(f, "Failed to open FPGA device: {}", msg),
            Self::InvalidBitstream(msg) => write!(f, "Invalid bitstream: {}", msg),
            Self::BitstreamLoadFailed(msg) => write!(f, "Failed to load bitstream: {}", msg),
            Self::Timeout(msg) => write!(f, "FPGA operation timeout: {}", msg),
            Self::DmaError(msg) => write!(f, "DMA error: {}", msg),
            Self::ConfigError(msg) => write!(f, "FPGA configuration error: {}", msg),
            Self::ResourceUnavailable(msg) => write!(f, "FPGA resource unavailable: {}", msg),
            Self::Unsupported(msg) => write!(f, "Unsupported FPGA operation: {}", msg),
            Self::IoError(msg) => write!(f, "FPGA I/O error: {}", msg),
            Self::DriverError(msg) => write!(f, "FPGA driver error: {}", msg),
            Self::SimulatorError(msg) => write!(f, "FPGA simulator error: {}", msg),
        }
    }
}

impl std::error::Error for FpgaError {}

impl From<std::io::Error> for FpgaError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = FpgaError::DeviceNotFound("Device 0".into());
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
        let fpga_err: FpgaError = io_err.into();
        assert!(matches!(fpga_err, FpgaError::IoError(_)));
    }
}
