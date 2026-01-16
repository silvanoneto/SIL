//! Configuração do FPGA

use serde::{Deserialize, Serialize};

/// Configuração do contexto FPGA
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FpgaConfig {
    /// ID do dispositivo (0 = primeiro FPGA)
    pub device_id: u32,
    /// Vendor do FPGA
    pub vendor: FpgaVendor,
    /// Família do FPGA
    pub family: FpgaFamily,
    /// Frequência de clock em MHz
    pub clock_mhz: u32,
    /// Tipo de interface
    pub interface: InterfaceType,
    /// Caminho para bitstream
    pub bitstream_path: Option<String>,
    /// Habilitar DMA
    pub enable_dma: bool,
    /// Tamanho do buffer DMA em bytes
    pub dma_buffer_size: usize,
    /// Habilitar interrupt handling
    pub enable_interrupts: bool,
    /// Timeout para operações em ms
    pub timeout_ms: u32,
}

impl Default for FpgaConfig {
    fn default() -> Self {
        Self {
            device_id: 0,
            vendor: FpgaVendor::Unknown,
            family: FpgaFamily::Unknown,
            clock_mhz: 100,
            interface: InterfaceType::Pcie,
            bitstream_path: None,
            enable_dma: true,
            dma_buffer_size: 1024 * 1024, // 1MB
            enable_interrupts: true,
            timeout_ms: 5000,
        }
    }
}

impl FpgaConfig {
    /// Define ID do dispositivo
    pub fn with_device_id(mut self, id: u32) -> Self {
        self.device_id = id;
        self
    }

    /// Define vendor
    pub fn with_vendor(mut self, vendor: FpgaVendor) -> Self {
        self.vendor = vendor;
        self
    }

    /// Define família
    pub fn with_family(mut self, family: FpgaFamily) -> Self {
        self.family = family;
        self
    }

    /// Define frequência de clock
    pub fn with_clock_mhz(mut self, mhz: u32) -> Self {
        self.clock_mhz = mhz;
        self
    }

    /// Define interface
    pub fn with_interface(mut self, interface: InterfaceType) -> Self {
        self.interface = interface;
        self
    }

    /// Define caminho do bitstream
    pub fn with_bitstream(mut self, path: impl Into<String>) -> Self {
        self.bitstream_path = Some(path.into());
        self
    }

    /// Desabilita DMA
    pub fn without_dma(mut self) -> Self {
        self.enable_dma = false;
        self
    }

    /// Define tamanho do buffer DMA
    pub fn with_dma_buffer_size(mut self, size: usize) -> Self {
        self.dma_buffer_size = size;
        self
    }

    /// Configuração para Xilinx Zynq
    pub fn xilinx_zynq() -> Self {
        Self::default()
            .with_vendor(FpgaVendor::Xilinx)
            .with_family(FpgaFamily::Zynq)
            .with_interface(InterfaceType::Axi)
            .with_clock_mhz(200)
    }

    /// Configuração para Intel Agilex
    pub fn intel_agilex() -> Self {
        Self::default()
            .with_vendor(FpgaVendor::Intel)
            .with_family(FpgaFamily::Agilex)
            .with_interface(InterfaceType::Pcie)
            .with_clock_mhz(300)
    }

    /// Configuração para simulador
    pub fn simulator() -> Self {
        Self::default()
            .with_vendor(FpgaVendor::Unknown)
            .with_family(FpgaFamily::Unknown)
            .with_interface(InterfaceType::Simulated)
            .without_dma()
    }
}

/// Vendor de FPGA
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FpgaVendor {
    /// Xilinx (AMD)
    Xilinx,
    /// Intel (Altera)
    Intel,
    /// Lattice Semiconductor
    Lattice,
    /// Gowin Semiconductor
    Gowin,
    /// Microchip (Microsemi)
    Microchip,
    /// Desconhecido
    #[default]
    Unknown,
}

impl FpgaVendor {
    /// Nome do vendor
    pub fn name(&self) -> &'static str {
        match self {
            Self::Xilinx => "Xilinx (AMD)",
            Self::Intel => "Intel (Altera)",
            Self::Lattice => "Lattice Semiconductor",
            Self::Gowin => "Gowin Semiconductor",
            Self::Microchip => "Microchip (Microsemi)",
            Self::Unknown => "Unknown",
        }
    }

    /// Ferramenta de síntese
    pub fn toolchain(&self) -> &'static str {
        match self {
            Self::Xilinx => "Vivado/Vitis",
            Self::Intel => "Quartus Prime",
            Self::Lattice => "Diamond/Radiant",
            Self::Gowin => "GOWIN EDA",
            Self::Microchip => "Libero SoC",
            Self::Unknown => "N/A",
        }
    }
}

/// Família de FPGA
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FpgaFamily {
    // Xilinx
    /// Zynq-7000 SoC
    Zynq,
    /// Zynq UltraScale+ MPSoC
    ZynqUltraScale,
    /// Versal ACAP
    Versal,
    /// Artix-7
    Artix7,
    /// Kintex-7
    Kintex7,
    /// Virtex-7
    Virtex7,
    /// UltraScale+
    UltraScale,

    // Intel
    /// Cyclone V
    CycloneV,
    /// Arria 10
    Arria10,
    /// Stratix 10
    Stratix10,
    /// Agilex
    Agilex,

    // Lattice
    /// ECP5
    Ecp5,
    /// CrossLink-NX
    CrossLinkNx,
    /// CertusPro-NX
    CertusProNx,

    // Gowin
    /// GW2A
    Gw2a,
    /// GW5A
    Gw5a,

    /// Desconhecida
    #[default]
    Unknown,
}

impl FpgaFamily {
    /// Vendor associado
    pub fn vendor(&self) -> FpgaVendor {
        match self {
            Self::Zynq | Self::ZynqUltraScale | Self::Versal |
            Self::Artix7 | Self::Kintex7 | Self::Virtex7 |
            Self::UltraScale => FpgaVendor::Xilinx,

            Self::CycloneV | Self::Arria10 | Self::Stratix10 |
            Self::Agilex => FpgaVendor::Intel,

            Self::Ecp5 | Self::CrossLinkNx | Self::CertusProNx => FpgaVendor::Lattice,

            Self::Gw2a | Self::Gw5a => FpgaVendor::Gowin,

            Self::Unknown => FpgaVendor::Unknown,
        }
    }

    /// Tem CPU integrado (SoC)
    pub fn has_cpu(&self) -> bool {
        matches!(
            self,
            Self::Zynq | Self::ZynqUltraScale | Self::Versal |
            Self::CycloneV | Self::Arria10 | Self::Agilex
        )
    }

    /// Suporta high-level synthesis (HLS)
    pub fn supports_hls(&self) -> bool {
        matches!(
            self,
            Self::Zynq | Self::ZynqUltraScale | Self::Versal |
            Self::UltraScale | Self::Agilex | Self::Stratix10
        )
    }
}

/// Tipo de interface host-FPGA
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum InterfaceType {
    /// PCIe (para FPGAs em placas de aceleração)
    #[default]
    Pcie,
    /// AXI (para SoCs ARM+FPGA)
    Axi,
    /// USB 3.0
    Usb3,
    /// USB 2.0
    Usb2,
    /// SPI
    Spi,
    /// JTAG (apenas programação)
    Jtag,
    /// Ethernet (para acesso remoto)
    Ethernet,
    /// Simulado (sem hardware)
    Simulated,
}

impl InterfaceType {
    /// Bandwidth teórico em MB/s
    pub fn bandwidth_mbps(&self) -> u32 {
        match self {
            Self::Pcie => 16000,   // PCIe Gen3 x16
            Self::Axi => 12800,   // AXI HP 128-bit @ 200MHz
            Self::Usb3 => 625,    // USB 3.0 SuperSpeed
            Self::Usb2 => 60,     // USB 2.0 High Speed
            Self::Spi => 50,      // SPI @ 50MHz
            Self::Jtag => 1,      // JTAG TCK @ 10MHz (apenas programação)
            Self::Ethernet => 125, // 1GbE
            Self::Simulated => u32::MAX,
        }
    }

    /// Suporta DMA
    pub fn supports_dma(&self) -> bool {
        matches!(self, Self::Pcie | Self::Axi | Self::Usb3)
    }

    /// Latência típica em µs
    pub fn typical_latency_us(&self) -> u32 {
        match self {
            Self::Pcie => 1,
            Self::Axi => 0,       // Zero-copy em SoCs
            Self::Usb3 => 100,
            Self::Usb2 => 1000,
            Self::Spi => 10,
            Self::Jtag => 10000,
            Self::Ethernet => 500,
            Self::Simulated => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = FpgaConfig::default()
            .with_device_id(1)
            .with_vendor(FpgaVendor::Xilinx)
            .with_clock_mhz(200);

        assert_eq!(config.device_id, 1);
        assert_eq!(config.vendor, FpgaVendor::Xilinx);
        assert_eq!(config.clock_mhz, 200);
    }

    #[test]
    fn test_xilinx_preset() {
        let config = FpgaConfig::xilinx_zynq();
        assert_eq!(config.vendor, FpgaVendor::Xilinx);
        assert_eq!(config.family, FpgaFamily::Zynq);
        assert_eq!(config.interface, InterfaceType::Axi);
    }

    #[test]
    fn test_family_vendor() {
        assert_eq!(FpgaFamily::Zynq.vendor(), FpgaVendor::Xilinx);
        assert_eq!(FpgaFamily::Agilex.vendor(), FpgaVendor::Intel);
        assert_eq!(FpgaFamily::Ecp5.vendor(), FpgaVendor::Lattice);
    }

    #[test]
    fn test_interface_bandwidth() {
        assert!(InterfaceType::Pcie.bandwidth_mbps() > InterfaceType::Usb3.bandwidth_mbps());
        assert!(InterfaceType::Axi.supports_dma());
        assert!(!InterfaceType::Spi.supports_dma());
    }
}
