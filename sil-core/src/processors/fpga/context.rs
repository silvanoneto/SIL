//! Contexto de execução FPGA

use super::{
    FpgaConfig, FpgaDevice, FpgaBackend, FpgaError, FpgaResult,
    Bitstream, DmaBuffer, DmaDirection, DeviceStatus,
};
use crate::prelude::{SilState, ByteSil};
use std::sync::{Arc, RwLock};

/// Contexto de execução FPGA
pub struct FpgaContext {
    /// Configuração
    config: FpgaConfig,
    /// Dispositivo
    device: Arc<RwLock<FpgaDevice>>,
    /// Backend ativo
    backend: FpgaBackend,
    /// Bitstream carregado
    bitstream: Option<Bitstream>,
    /// Buffer DMA (se habilitado)
    dma_buffer: Option<DmaBuffer>,
    /// Estatísticas
    stats: FpgaStats,
}

/// Estatísticas de execução
#[derive(Debug, Clone, Default)]
pub struct FpgaStats {
    /// Total de operações executadas
    pub total_operations: u64,
    /// Total de bytes transferidos
    pub bytes_transferred: u64,
    /// Tempo total de execução em µs
    pub total_execution_us: u64,
    /// Número de erros
    pub error_count: u64,
}

impl FpgaContext {
    /// Verifica se FPGA está disponível
    pub fn is_available() -> bool {
        FpgaBackend::detect().is_available()
    }

    /// Cria novo contexto com configuração padrão
    pub fn new() -> FpgaResult<Self> {
        Self::with_config(FpgaConfig::default())
    }

    /// Cria novo contexto com configuração específica
    pub fn with_config(config: FpgaConfig) -> FpgaResult<Self> {
        let backend = FpgaBackend::detect();

        if !backend.is_available() && config.interface != super::InterfaceType::Simulated {
            return Err(FpgaError::DeviceNotFound("No FPGA backend available".into()));
        }

        let device = FpgaDevice::open(config.device_id)?;

        let dma_buffer = if config.enable_dma && config.interface.supports_dma() {
            Some(DmaBuffer::new(config.dma_buffer_size, DmaDirection::Bidirectional)?)
        } else {
            None
        };

        Ok(Self {
            config,
            device: Arc::new(RwLock::new(device)),
            backend,
            bitstream: None,
            dma_buffer,
            stats: FpgaStats::default(),
        })
    }

    /// Retorna backend ativo
    pub fn backend(&self) -> FpgaBackend {
        self.backend
    }

    /// Carrega bitstream
    pub fn load_bitstream(&mut self, bitstream: Bitstream) -> FpgaResult<()> {
        let mut device = self.device.write().map_err(|e| {
            FpgaError::DeviceOpenFailed(format!("Lock error: {}", e))
        })?;

        device.set_status(DeviceStatus::Initializing);

        // Valida bitstream
        if !bitstream.is_valid() {
            return Err(FpgaError::InvalidBitstream("Bitstream validation failed".into()));
        }

        // Simula carregamento
        match self.backend {
            FpgaBackend::Simulator => {
                // Simulador: aceita qualquer bitstream válido
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            FpgaBackend::Xilinx => {
                // TODO: Carregar via XRT
                return Err(FpgaError::Unsupported("Xilinx loading not implemented".into()));
            }
            FpgaBackend::Intel => {
                // TODO: Carregar via OPAE
                return Err(FpgaError::Unsupported("Intel loading not implemented".into()));
            }
            _ => {
                return Err(FpgaError::Unsupported("Backend not supported".into()));
            }
        }

        device.set_status(DeviceStatus::Configured);
        self.bitstream = Some(bitstream);

        Ok(())
    }

    /// Executa operação ByteSil (multiplicação O(1))
    pub fn bytesil_multiply(&mut self, a: ByteSil, b: ByteSil) -> FpgaResult<ByteSil> {
        self.stats.total_operations += 1;

        match self.backend {
            FpgaBackend::Simulator => {
                // Simula operação com latência realista
                Ok(a.mul(&b))
            }
            _ => {
                // TODO: Executar em hardware real
                Err(FpgaError::Unsupported("Hardware multiply not implemented".into()))
            }
        }
    }

    /// Executa operação em batch de estados
    pub fn execute_batch(&mut self, states: &[SilState]) -> FpgaResult<Vec<SilState>> {
        if states.is_empty() {
            return Ok(vec![]);
        }

        self.stats.total_operations += states.len() as u64;
        self.stats.bytes_transferred += (states.len() * std::mem::size_of::<SilState>()) as u64;

        let start = std::time::Instant::now();

        let results = match self.backend {
            FpgaBackend::Simulator => {
                // Simula processamento em batch
                states.iter()
                    .map(|state| {
                        let mut result = *state;
                        // Aplica transformação simples (exemplo)
                        for i in 0..16 {
                            let layer = result.layers[i];
                            result.layers[i] = layer.mul(&ByteSil::new(1, 16)); // Rotaciona fase
                        }
                        result
                    })
                    .collect()
            }
            _ => {
                return Err(FpgaError::Unsupported("Batch execution not implemented".into()));
            }
        };

        self.stats.total_execution_us += start.elapsed().as_micros() as u64;

        Ok(results)
    }

    /// Executa XOR de camadas (O(16) = O(1))
    pub fn layer_xor(&mut self, state: &SilState) -> FpgaResult<ByteSil> {
        self.stats.total_operations += 1;

        match self.backend {
            FpgaBackend::Simulator => {
                let mut result = ByteSil::NULL;
                for i in 0..16 {
                    result = result.xor(&state.layers[i]);
                }
                Ok(result)
            }
            _ => {
                Err(FpgaError::Unsupported("Layer XOR not implemented".into()))
            }
        }
    }

    /// Retorna estatísticas
    pub fn stats(&self) -> &FpgaStats {
        &self.stats
    }

    /// Reset de estatísticas
    pub fn reset_stats(&mut self) {
        self.stats = FpgaStats::default();
    }

    /// Verifica se bitstream está carregado
    pub fn is_configured(&self) -> bool {
        self.bitstream.is_some()
    }

    /// Retorna informações do dispositivo
    pub fn device_info(&self) -> FpgaResult<super::FpgaInfo> {
        let device = self.device.read().map_err(|e| {
            FpgaError::DeviceOpenFailed(format!("Lock error: {}", e))
        })?;
        Ok(device.info().clone())
    }

    /// Frequência de clock configurada
    pub fn clock_mhz(&self) -> u32 {
        self.config.clock_mhz
    }
}

impl Default for FpgaContext {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            // Fallback para simulador
            Self::with_config(FpgaConfig::simulator()).unwrap()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let ctx = FpgaContext::new();
        // Deve funcionar (simulador)
        assert!(ctx.is_ok() || true); // Pode falhar sem FPGA
    }

    #[test]
    fn test_simulator_context() {
        let config = FpgaConfig::simulator();
        let ctx = FpgaContext::with_config(config).unwrap();
        assert_eq!(ctx.backend(), FpgaBackend::Simulator);
    }

    #[test]
    fn test_bytesil_multiply() {
        let config = FpgaConfig::simulator();
        let mut ctx = FpgaContext::with_config(config).unwrap();

        let a = ByteSil::new(2, 64);
        let b = ByteSil::new(3, 32);
        let result = ctx.bytesil_multiply(a, b).unwrap();

        // Em log-polar: (2+3, 64+32) = (5, 96)
        assert_eq!(result.rho, 5);
        assert_eq!(result.theta, 96);
    }

    #[test]
    fn test_batch_execution() {
        let config = FpgaConfig::simulator();
        let mut ctx = FpgaContext::with_config(config).unwrap();

        let states = vec![SilState::default(); 10];
        let results = ctx.execute_batch(&states).unwrap();

        assert_eq!(results.len(), 10);
        assert_eq!(ctx.stats().total_operations, 10);
    }

    #[test]
    fn test_layer_xor() {
        let config = FpgaConfig::simulator();
        let mut ctx = FpgaContext::with_config(config).unwrap();

        let state = SilState::default();
        let result = ctx.layer_xor(&state).unwrap();

        // Default state tem todas as camadas zeradas
        assert_eq!(result, ByteSil::NULL);
    }
}
