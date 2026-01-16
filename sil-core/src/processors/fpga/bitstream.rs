//! Gerenciamento de bitstream FPGA

use super::{FpgaError, FpgaResult, FpgaVendor, FpgaFamily};
use std::path::Path;

/// Metadados do bitstream
#[derive(Debug, Clone)]
pub struct BitstreamMetadata {
    /// Nome do design
    pub design_name: String,
    /// Versão
    pub version: String,
    /// Data de criação
    pub created_at: String,
    /// Vendor alvo
    pub target_vendor: FpgaVendor,
    /// Família alvo
    pub target_family: FpgaFamily,
    /// Part number alvo
    pub target_part: String,
    /// Frequência de clock em MHz
    pub clock_mhz: u32,
    /// Tamanho em bytes
    pub size_bytes: usize,
    /// Checksum (CRC32)
    pub checksum: u32,
}

impl Default for BitstreamMetadata {
    fn default() -> Self {
        Self {
            design_name: "Unknown".into(),
            version: "0.0.0".into(),
            created_at: "Unknown".into(),
            target_vendor: FpgaVendor::Unknown,
            target_family: FpgaFamily::Unknown,
            target_part: "Unknown".into(),
            clock_mhz: 100,
            size_bytes: 0,
            checksum: 0,
        }
    }
}

/// Bitstream FPGA
pub struct Bitstream {
    /// Metadados
    metadata: BitstreamMetadata,
    /// Dados do bitstream
    data: Vec<u8>,
    /// Validado?
    validated: bool,
}

impl Bitstream {
    /// Cria bitstream vazio (para testes/simulação)
    pub fn empty() -> Self {
        Self {
            metadata: BitstreamMetadata::default(),
            data: vec![],
            validated: true,
        }
    }

    /// Carrega bitstream de arquivo
    pub fn from_file<P: AsRef<Path>>(path: P) -> FpgaResult<Self> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(FpgaError::InvalidBitstream(format!(
                "File not found: {}",
                path.display()
            )));
        }

        let data = std::fs::read(path)?;

        if data.is_empty() {
            return Err(FpgaError::InvalidBitstream("Empty bitstream file".into()));
        }

        // Detecta formato pelo header
        let (vendor, family) = Self::detect_format(&data)?;

        let metadata = BitstreamMetadata {
            design_name: path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown")
                .to_string(),
            target_vendor: vendor,
            target_family: family,
            size_bytes: data.len(),
            checksum: Self::calculate_checksum(&data),
            ..Default::default()
        };

        let mut bitstream = Self {
            metadata,
            data,
            validated: false,
        };

        bitstream.validate()?;

        Ok(bitstream)
    }

    /// Cria bitstream de bytes
    pub fn from_bytes(data: Vec<u8>, metadata: BitstreamMetadata) -> FpgaResult<Self> {
        if data.is_empty() {
            return Err(FpgaError::InvalidBitstream("Empty bitstream data".into()));
        }

        let mut bitstream = Self {
            metadata: BitstreamMetadata {
                size_bytes: data.len(),
                checksum: Self::calculate_checksum(&data),
                ..metadata
            },
            data,
            validated: false,
        };

        bitstream.validate()?;

        Ok(bitstream)
    }

    /// Detecta formato do bitstream pelo header
    fn detect_format(data: &[u8]) -> FpgaResult<(FpgaVendor, FpgaFamily)> {
        if data.len() < 16 {
            return Err(FpgaError::InvalidBitstream("Bitstream too small".into()));
        }

        // Xilinx .bit file magic: 0x00 0x09 0x0F 0xF0
        if data.starts_with(&[0x00, 0x09, 0x0F, 0xF0]) {
            return Ok((FpgaVendor::Xilinx, FpgaFamily::Unknown));
        }

        // Xilinx .bin file (raw) - começa com sync word
        if data.len() >= 4 {
            let sync = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
            if sync == 0xAA995566 {
                return Ok((FpgaVendor::Xilinx, FpgaFamily::Unknown));
            }
        }

        // Intel/Altera .sof/.rbf - diferentes headers
        if data.starts_with(&[0x00, 0x00, 0x01, 0x00]) {
            return Ok((FpgaVendor::Intel, FpgaFamily::Unknown));
        }

        // Lattice .bit - começa com 0xFF 0x00
        if data.starts_with(&[0xFF, 0x00]) {
            return Ok((FpgaVendor::Lattice, FpgaFamily::Unknown));
        }

        // Formato desconhecido - aceita como genérico
        Ok((FpgaVendor::Unknown, FpgaFamily::Unknown))
    }

    /// Calcula checksum CRC32
    fn calculate_checksum(data: &[u8]) -> u32 {
        // CRC32 simples (IEEE 802.3)
        let mut crc: u32 = 0xFFFFFFFF;
        for byte in data {
            crc ^= *byte as u32;
            for _ in 0..8 {
                if crc & 1 != 0 {
                    crc = (crc >> 1) ^ 0xEDB88320;
                } else {
                    crc >>= 1;
                }
            }
        }
        !crc
    }

    /// Valida bitstream
    pub fn validate(&mut self) -> FpgaResult<()> {
        // Verifica tamanho mínimo
        if self.data.len() < 16 {
            return Err(FpgaError::InvalidBitstream("Bitstream too small".into()));
        }

        // Verifica checksum
        let calculated = Self::calculate_checksum(&self.data);
        if self.metadata.checksum != 0 && self.metadata.checksum != calculated {
            return Err(FpgaError::InvalidBitstream(format!(
                "Checksum mismatch: expected 0x{:08X}, got 0x{:08X}",
                self.metadata.checksum, calculated
            )));
        }

        self.metadata.checksum = calculated;
        self.validated = true;

        Ok(())
    }

    /// Retorna metadados
    pub fn metadata(&self) -> &BitstreamMetadata {
        &self.metadata
    }

    /// Retorna dados
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Tamanho em bytes
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Verifica se está validado
    pub fn is_valid(&self) -> bool {
        self.validated
    }

    /// Vendor alvo
    pub fn vendor(&self) -> FpgaVendor {
        self.metadata.target_vendor
    }

    /// Família alvo
    pub fn family(&self) -> FpgaFamily {
        self.metadata.target_family
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_bitstream() {
        let bs = Bitstream::empty();
        assert!(bs.is_valid());
        assert_eq!(bs.size(), 0);
    }

    #[test]
    fn test_bitstream_from_bytes() {
        // Dados fictícios com header Xilinx
        let data = vec![0xAA, 0x99, 0x55, 0x66, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

        let bs = Bitstream::from_bytes(data, BitstreamMetadata::default()).unwrap();
        assert!(bs.is_valid());
        assert_eq!(bs.vendor(), FpgaVendor::Xilinx);
    }

    #[test]
    fn test_checksum() {
        let data = b"Hello FPGA!".to_vec();
        let crc = Bitstream::calculate_checksum(&data);
        assert_ne!(crc, 0);

        // Mesmo dado = mesmo checksum
        let crc2 = Bitstream::calculate_checksum(&data);
        assert_eq!(crc, crc2);
    }

    #[test]
    fn test_detect_xilinx_format() {
        // Sync word Xilinx
        let data = vec![0xAA, 0x99, 0x55, 0x66, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

        let (vendor, _) = Bitstream::detect_format(&data).unwrap();
        assert_eq!(vendor, FpgaVendor::Xilinx);
    }
}
