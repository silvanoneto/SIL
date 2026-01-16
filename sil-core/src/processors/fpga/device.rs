//! Abstração de dispositivo FPGA

use super::{FpgaError, FpgaResult, FpgaVendor, FpgaFamily, FpgaCapabilities, InterfaceType};

/// Status do dispositivo FPGA
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DeviceStatus {
    /// Não inicializado
    #[default]
    Uninitialized,
    /// Inicializando
    Initializing,
    /// Pronto (sem bitstream)
    Ready,
    /// Configurado (bitstream carregado)
    Configured,
    /// Em execução
    Running,
    /// Erro
    Error,
    /// Desconectado
    Disconnected,
}

/// Informações do dispositivo FPGA
#[derive(Debug, Clone)]
pub struct FpgaInfo {
    /// ID do dispositivo
    pub device_id: u32,
    /// Nome do dispositivo
    pub name: String,
    /// Vendor
    pub vendor: FpgaVendor,
    /// Família
    pub family: FpgaFamily,
    /// Part number
    pub part_number: String,
    /// Serial number
    pub serial_number: Option<String>,
    /// Versão do firmware/driver
    pub driver_version: String,
    /// Interface
    pub interface: InterfaceType,
    /// Capacidades
    pub capabilities: FpgaCapabilities,
    /// Temperatura atual (se disponível)
    pub temperature_c: Option<f32>,
    /// Voltagem core (se disponível)
    pub vcore_v: Option<f32>,
}

impl Default for FpgaInfo {
    fn default() -> Self {
        Self {
            device_id: 0,
            name: "Unknown FPGA".into(),
            vendor: FpgaVendor::Unknown,
            family: FpgaFamily::Unknown,
            part_number: "N/A".into(),
            serial_number: None,
            driver_version: "0.0.0".into(),
            interface: InterfaceType::Simulated,
            capabilities: FpgaCapabilities::default(),
            temperature_c: None,
            vcore_v: None,
        }
    }
}

/// Dispositivo FPGA
pub struct FpgaDevice {
    /// Informações do dispositivo
    info: FpgaInfo,
    /// Status atual
    status: DeviceStatus,
    /// Handle interno (opaco por plataforma)
    #[allow(dead_code)]
    handle: DeviceHandle,
}

/// Handle do dispositivo (específico por plataforma)
enum DeviceHandle {
    /// Simulador
    Simulated,
    /// Handle real (ponteiro para estrutura do driver)
    #[allow(dead_code)]
    Native(*mut std::ffi::c_void),
}

// SAFETY: O handle é gerenciado internamente e sincronizado via mutex
unsafe impl Send for FpgaDevice {}
unsafe impl Sync for FpgaDevice {}

impl FpgaDevice {
    /// Cria dispositivo simulado
    pub fn simulated() -> Self {
        Self {
            info: FpgaInfo {
                name: "FPGA Simulator".into(),
                vendor: FpgaVendor::Unknown,
                family: FpgaFamily::Unknown,
                part_number: "SIM-001".into(),
                driver_version: env!("CARGO_PKG_VERSION").into(),
                interface: InterfaceType::Simulated,
                capabilities: FpgaCapabilities {
                    luts: 100_000,
                    flip_flops: 200_000,
                    bram_kb: 4096,
                    dsp_slices: 256,
                    max_clock_mhz: 500,
                    partial_reconfig: true,
                    interfaces: vec![InterfaceType::Simulated],
                },
                ..Default::default()
            },
            status: DeviceStatus::Ready,
            handle: DeviceHandle::Simulated,
        }
    }

    /// Enumera dispositivos disponíveis
    pub fn enumerate() -> Vec<FpgaInfo> {
        let mut devices = Vec::new();

        // Tenta detectar dispositivos reais
        #[cfg(feature = "fpga-xilinx")]
        devices.extend(Self::enumerate_xilinx());

        #[cfg(feature = "fpga-intel")]
        devices.extend(Self::enumerate_intel());

        // Se nenhum dispositivo real, adiciona simulador
        if devices.is_empty() {
            devices.push(Self::simulated().info.clone());
        }

        devices
    }

    #[cfg(feature = "fpga-xilinx")]
    fn enumerate_xilinx() -> Vec<FpgaInfo> {
        // TODO: Implementar enumeração real via XRT
        vec![]
    }

    #[cfg(feature = "fpga-intel")]
    fn enumerate_intel() -> Vec<FpgaInfo> {
        // TODO: Implementar enumeração real via OPAE
        vec![]
    }

    /// Abre dispositivo por ID
    pub fn open(device_id: u32) -> FpgaResult<Self> {
        let devices = Self::enumerate();

        if device_id as usize >= devices.len() {
            return Err(FpgaError::DeviceNotFound(format!(
                "Device {} not found (available: {})",
                device_id,
                devices.len()
            )));
        }

        let info = devices[device_id as usize].clone();

        // Para simulador, retorna diretamente
        if info.interface == InterfaceType::Simulated {
            return Ok(Self::simulated());
        }

        // TODO: Abrir dispositivo real via driver
        Err(FpgaError::Unsupported("Real FPGA devices not yet implemented".into()))
    }

    /// Retorna informações do dispositivo
    pub fn info(&self) -> &FpgaInfo {
        &self.info
    }

    /// Retorna status atual
    pub fn status(&self) -> DeviceStatus {
        self.status
    }

    /// Atualiza status
    pub fn set_status(&mut self, status: DeviceStatus) {
        self.status = status;
    }

    /// Verifica se está pronto
    pub fn is_ready(&self) -> bool {
        matches!(self.status, DeviceStatus::Ready | DeviceStatus::Configured)
    }

    /// Verifica se está configurado
    pub fn is_configured(&self) -> bool {
        matches!(self.status, DeviceStatus::Configured | DeviceStatus::Running)
    }

    /// Reset do dispositivo
    pub fn reset(&mut self) -> FpgaResult<()> {
        match &self.handle {
            DeviceHandle::Simulated => {
                self.status = DeviceStatus::Ready;
                Ok(())
            }
            DeviceHandle::Native(_) => {
                // TODO: Reset via driver
                Err(FpgaError::Unsupported("Reset not implemented".into()))
            }
        }
    }

    /// Lê temperatura
    pub fn read_temperature(&self) -> FpgaResult<f32> {
        match &self.handle {
            DeviceHandle::Simulated => Ok(45.0), // Temperatura fictícia
            DeviceHandle::Native(_) => {
                // TODO: Ler via sysfs/driver
                Err(FpgaError::Unsupported("Temperature reading not implemented".into()))
            }
        }
    }
}

impl Drop for FpgaDevice {
    fn drop(&mut self) {
        // Limpa handle nativo se necessário
        if let DeviceHandle::Native(ptr) = &self.handle {
            if !ptr.is_null() {
                // TODO: Fechar handle via driver
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_enumerate() {
        let devices = FpgaDevice::enumerate();
        assert!(!devices.is_empty(), "Should have at least simulator");
    }

    #[test]
    fn test_simulated_device() {
        let device = FpgaDevice::simulated();
        assert_eq!(device.status(), DeviceStatus::Ready);
        assert!(device.is_ready());
    }

    #[test]
    fn test_device_open() {
        let result = FpgaDevice::open(0);
        // Deve funcionar (retorna simulador)
        assert!(result.is_ok());
    }

    #[test]
    fn test_device_temperature() {
        let device = FpgaDevice::simulated();
        let temp = device.read_temperature().unwrap();
        assert!(temp > 0.0);
    }
}
