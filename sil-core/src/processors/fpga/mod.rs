//! # FPGA Backend
//!
//! Aceleração de hardware via FPGA para operações VSP/SIL.
//!
//! ## Arquitetura
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                       FpgaContext                                │
//! │  ┌───────────────────────────────────────────────────────────┐  │
//! │  │                   FPGA Device                              │  │
//! │  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐       │  │
//! │  │  │ ByteSil ALU │  │ Layer Ops   │  │ Transform   │       │  │
//! │  │  │  O(1) mul   │  │  O(16) xor  │  │  pipeline   │       │  │
//! │  │  └─────────────┘  └─────────────┘  └─────────────┘       │  │
//! │  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐       │  │
//! │  │  │ DMA Engine  │  │ Interrupt   │  │ Clock Mgmt  │       │  │
//! │  │  │  zero-copy  │  │  handler    │  │  dynamic    │       │  │
//! │  │  └─────────────┘  └─────────────┘  └─────────────┘       │  │
//! │  └───────────────────────────────────────────────────────────┘  │
//! │  ┌───────────────────────────────────────────────────────────┐  │
//! │  │                   Host Interface                          │  │
//! │  │  PCIe/USB3/AXI | Memory Mapped | DMA Transfers           │  │
//! │  └───────────────────────────────────────────────────────────┘  │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Plataformas Suportadas
//!
//! | Vendor | Família | Interface | Status |
//! |--------|---------|-----------|--------|
//! | Xilinx | Zynq UltraScale+ | AXI/PCIe | Planejado |
//! | Intel/Altera | Agilex | PCIe | Planejado |
//! | Lattice | ECP5/Nexus | USB3/SPI | Planejado |
//! | Gowin | GW2A | USB2/SPI | Planejado |
//!
//! ## Operações Aceleradas
//!
//! - **ByteSil ALU**: Multiplicação, divisão, potência em O(1)
//! - **Layer Ops**: XOR/AND/OR de 16 camadas em paralelo
//! - **Transforms**: Pipeline de transformações com latência fixa
//! - **Batch Processing**: DMA para processamento de lotes
//!
//! ## Uso
//!
//! ```ignore
//! use sil_core::processors::fpga::{FpgaContext, FpgaConfig};
//!
//! let config = FpgaConfig::default()
//!     .with_device_id(0)
//!     .with_clock_mhz(200);
//!
//! let fpga = FpgaContext::new(config)?;
//! let result = fpga.execute_batch(&states)?;
//! ```

mod context;
mod config;
mod error;
mod device;
mod bitstream;
mod dma;

pub use context::FpgaContext;
pub use config::{FpgaConfig, FpgaVendor, FpgaFamily, InterfaceType};
pub use error::{FpgaError, FpgaResult};
pub use device::{FpgaDevice, FpgaInfo, DeviceStatus};
pub use bitstream::{Bitstream, BitstreamMetadata};
pub use dma::{DmaBuffer, DmaDirection};

use crate::prelude::SilState;

/// Trait para backends FPGA
pub trait FpgaBackendImpl: Send + Sync {
    /// Nome do backend
    fn name(&self) -> &str;

    /// Verifica se está disponível
    fn is_available(&self) -> bool;

    /// Versão do driver/runtime
    fn version(&self) -> Option<String>;

    /// Carrega bitstream
    fn load_bitstream(&mut self, bitstream: &Bitstream) -> FpgaResult<()>;

    /// Executa operação em batch
    fn execute_batch(&self, states: &[SilState]) -> FpgaResult<Vec<SilState>>;
}

/// Backend FPGA ativo
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FpgaBackend {
    /// Xilinx FPGA (Vivado runtime)
    Xilinx,
    /// Intel/Altera FPGA (Quartus runtime)
    Intel,
    /// Lattice FPGA
    Lattice,
    /// Gowin FPGA
    Gowin,
    /// Simulação (sem hardware)
    Simulator,
    /// Nenhum disponível
    None,
}

impl FpgaBackend {
    /// Detecta backend disponível
    pub fn detect() -> Self {
        // Ordem de preferência: Xilinx > Intel > Lattice > Gowin > Simulator

        #[cfg(feature = "fpga-xilinx")]
        if Self::is_xilinx_available() {
            return Self::Xilinx;
        }

        #[cfg(feature = "fpga-intel")]
        if Self::is_intel_available() {
            return Self::Intel;
        }

        #[cfg(feature = "fpga-lattice")]
        if Self::is_lattice_available() {
            return Self::Lattice;
        }

        // Simulador sempre disponível para desenvolvimento
        #[cfg(feature = "fpga-sim")]
        {
            return Self::Simulator;
        }

        #[cfg(not(feature = "fpga-sim"))]
        Self::None
    }

    /// Nome do backend
    pub fn name(&self) -> &'static str {
        match self {
            Self::Xilinx => "Xilinx FPGA",
            Self::Intel => "Intel FPGA",
            Self::Lattice => "Lattice FPGA",
            Self::Gowin => "Gowin FPGA",
            Self::Simulator => "FPGA Simulator",
            Self::None => "No FPGA",
        }
    }

    /// Verifica se está disponível
    pub fn is_available(&self) -> bool {
        !matches!(self, Self::None)
    }

    #[cfg(feature = "fpga-xilinx")]
    fn is_xilinx_available() -> bool {
        // Verifica se Vivado runtime está disponível
        // Em produção: checaria /dev/xdma* ou libxrt
        std::path::Path::new("/opt/xilinx").exists()
            || std::env::var("XILINX_XRT").is_ok()
    }

    #[cfg(feature = "fpga-intel")]
    fn is_intel_available() -> bool {
        // Verifica se Quartus runtime está disponível
        std::path::Path::new("/opt/intel/fpga").exists()
            || std::env::var("QUARTUS_ROOTDIR").is_ok()
    }

    #[cfg(feature = "fpga-lattice")]
    fn is_lattice_available() -> bool {
        // Verifica se Lattice Diamond está disponível
        std::path::Path::new("/usr/local/diamond").exists()
            || std::env::var("LATTICE_DIAMOND").is_ok()
    }
}

impl Default for FpgaBackend {
    fn default() -> Self {
        Self::detect()
    }
}

/// Capacidades do FPGA
#[derive(Debug, Clone)]
pub struct FpgaCapabilities {
    /// LUTs disponíveis
    pub luts: u32,
    /// FFs disponíveis
    pub flip_flops: u32,
    /// Block RAM em Kb
    pub bram_kb: u32,
    /// DSP slices
    pub dsp_slices: u32,
    /// Frequência máxima em MHz
    pub max_clock_mhz: u32,
    /// Suporta partial reconfiguration
    pub partial_reconfig: bool,
    /// Interfaces disponíveis
    pub interfaces: Vec<InterfaceType>,
}

impl Default for FpgaCapabilities {
    fn default() -> Self {
        Self {
            luts: 0,
            flip_flops: 0,
            bram_kb: 0,
            dsp_slices: 0,
            max_clock_mhz: 100,
            partial_reconfig: false,
            interfaces: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fpga_backend_detection() {
        let backend = FpgaBackend::detect();
        // Em ambiente de CI sem FPGA, deve retornar None ou Simulator
        println!("Detected FPGA backend: {:?}", backend);
    }

    #[test]
    fn test_fpga_backend_name() {
        assert_eq!(FpgaBackend::Xilinx.name(), "Xilinx FPGA");
        assert_eq!(FpgaBackend::Intel.name(), "Intel FPGA");
        assert_eq!(FpgaBackend::None.name(), "No FPGA");
    }

    #[test]
    fn test_fpga_capabilities_default() {
        let caps = FpgaCapabilities::default();
        assert_eq!(caps.max_clock_mhz, 100);
        assert!(!caps.partial_reconfig);
    }
}
