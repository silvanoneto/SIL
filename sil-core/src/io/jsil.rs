//! # JSIL — JSON Lines comprimido com SIL
//!
//! Formato híbrido que combina estrutura JSONL com compressão ByteSil nativa.
//! 
//! ## Formato .jsil
//!
//! ```text
//! ┌─────────────────────────────────────────────────────┐
//! │ JSIL Header (32 bytes)                              │
//! ├─────────────────────────────────────────────────────┤
//! │ Compressed JSONL Data (ByteSil encoded)             │
//! └─────────────────────────────────────────────────────┘
//! ```
//!
//! ## Vantagens
//!
//! - **Compressão semântica**: ByteSil explora padrões nos dados
//! - **Streaming nativo**: Descompressão incremental
//! - **Recuperável**: Checkpoints em formato não-comprimido
//! - **Reversível**: Transformações SIL são inversíveis

use crate::vsp::bytecode::SilcFile;
use crate::vsp::error::{VspError, VspResult};
use crate::state::ByteSil;
use crate::io::transforms::{SilTransformFn, Rotate, Xor};
use crate::io::jsonl::{SilcToJsonl, JsonlConfig, JsonlRecord};
use std::io::{Write, Read};
use std::path::Path;
use serde::{Serialize, Deserialize};

// ═══════════════════════════════════════════════════════════════════════════
// HEADER JSIL
// ═══════════════════════════════════════════════════════════════════════════

/// Magic number: "JSIL"
pub const JSIL_MAGIC: u32 = 0x4C49534A; // "JSIL" em little-endian

/// Versão do formato
pub const JSIL_VERSION: u16 = 0x0100; // v1.0

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CompressionMode {
    /// Sem compressão (JSONL puro)
    None = 0,
    /// XOR simples (rápido, compressão leve)
    Xor = 1,
    /// Rotação de fase (médio)
    Rotate = 2,
    /// XOR + Rotação (forte)
    XorRotate = 3,
    /// Adaptive (analisa dados e escolhe melhor)
    Adaptive = 4,
}

impl CompressionMode {
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0 => Some(Self::None),
            1 => Some(Self::Xor),
            2 => Some(Self::Rotate),
            3 => Some(Self::XorRotate),
            4 => Some(Self::Adaptive),
            _ => None,
        }
    }
}

/// Header do arquivo .jsil
#[derive(Debug, Clone)]
pub struct JsilHeader {
    /// Magic number
    pub magic: u32,
    /// Versão
    pub version: u16,
    /// Modo de compressão
    pub compression: CompressionMode,
    /// Parâmetro de compressão (ex: chave XOR)
    pub compression_param: u8,
    /// Tamanho original (descomprimido)
    pub uncompressed_size: u32,
    /// Tamanho comprimido
    pub compressed_size: u32,
    /// Número de registros JSONL
    pub record_count: u32,
    /// Checksum dos dados comprimidos
    pub checksum: u64,
}

impl JsilHeader {
    pub const SIZE: usize = 32;
    
    pub fn new(compression: CompressionMode) -> Self {
        Self {
            magic: JSIL_MAGIC,
            version: JSIL_VERSION,
            compression,
            compression_param: 0,
            uncompressed_size: 0,
            compressed_size: 0,
            record_count: 0,
            checksum: 0,
        }
    }
    
    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut bytes = [0u8; Self::SIZE];
        
        bytes[0..4].copy_from_slice(&self.magic.to_le_bytes());
        bytes[4..6].copy_from_slice(&self.version.to_le_bytes());
        bytes[6] = self.compression as u8;
        bytes[7] = self.compression_param;
        bytes[8..12].copy_from_slice(&self.uncompressed_size.to_le_bytes());
        bytes[12..16].copy_from_slice(&self.compressed_size.to_le_bytes());
        bytes[16..20].copy_from_slice(&self.record_count.to_le_bytes());
        bytes[20..28].copy_from_slice(&self.checksum.to_le_bytes());
        // bytes[28..32] reservado para futuro
        
        bytes
    }
    
    pub fn from_bytes(bytes: &[u8]) -> VspResult<Self> {
        if bytes.len() < Self::SIZE {
            return Err(VspError::InvalidBytecode("JSIL header too short".into()));
        }
        
        let magic = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        if magic != JSIL_MAGIC {
            return Err(VspError::InvalidBytecode(format!(
                "Invalid JSIL magic: expected 0x{:08X}, got 0x{:08X}",
                JSIL_MAGIC, magic
            )));
        }
        
        let compression = CompressionMode::from_byte(bytes[6])
            .ok_or_else(|| VspError::InvalidBytecode("Invalid compression mode".into()))?;
        
        Ok(Self {
            magic,
            version: u16::from_le_bytes([bytes[4], bytes[5]]),
            compression,
            compression_param: bytes[7],
            uncompressed_size: u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]),
            compressed_size: u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]),
            record_count: u32::from_le_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]),
            checksum: u64::from_le_bytes([
                bytes[20], bytes[21], bytes[22], bytes[23],
                bytes[24], bytes[25], bytes[26], bytes[27],
            ]),
        })
    }
    
    pub fn compression_ratio(&self) -> f64 {
        if self.uncompressed_size == 0 {
            return 1.0;
        }
        self.compressed_size as f64 / self.uncompressed_size as f64
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// COMPRESSOR JSIL
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Clone)]
pub struct JsilCompressor {
    mode: CompressionMode,
    param: u8,
}

impl JsilCompressor {
    pub fn new(mode: CompressionMode, param: u8) -> Self {
        Self { mode, param }
    }
    
    /// Compressor padrão (XOR + Rotate com chave 0x5A)
    pub fn default() -> Self {
        Self::new(CompressionMode::XorRotate, 0x5A)
    }
    
    /// Compressor adaptativo (analisa dados)
    pub fn adaptive() -> Self {
        Self::new(CompressionMode::Adaptive, 0)
    }
    
    /// Comprime bytes usando transformações SIL
    pub fn compress(&self, data: &[u8]) -> Vec<u8> {
        match self.mode {
            CompressionMode::None => data.to_vec(),
            CompressionMode::Xor => self.compress_xor(data),
            CompressionMode::Rotate => self.compress_rotate(data),
            CompressionMode::XorRotate => self.compress_xor_rotate(data),
            CompressionMode::Adaptive => self.compress_adaptive(data),
        }
    }
    
    /// Descomprime bytes
    pub fn decompress(&self, data: &[u8]) -> Vec<u8> {
        match self.mode {
            CompressionMode::None => data.to_vec(),
            CompressionMode::Xor => self.decompress_xor(data),
            CompressionMode::Rotate => self.decompress_rotate(data),
            CompressionMode::XorRotate => self.decompress_xor_rotate(data),
            CompressionMode::Adaptive => self.decompress_adaptive(data),
        }
    }
    
    // ───────────────────────────────────────────────────────────────────────
    // XOR Compression
    // ───────────────────────────────────────────────────────────────────────
    
    fn compress_xor(&self, data: &[u8]) -> Vec<u8> {
        let xor = Xor(self.param);
        data.iter()
            .map(|&b| {
                let byte_sil = ByteSil::from_u8(b);
                xor.apply_byte(byte_sil).to_u8()
            })
            .collect()
    }
    
    fn decompress_xor(&self, data: &[u8]) -> Vec<u8> {
        // XOR é reversível
        self.compress_xor(data)
    }
    
    // ───────────────────────────────────────────────────────────────────────
    // Rotate Compression
    // ───────────────────────────────────────────────────────────────────────
    
    fn compress_rotate(&self, data: &[u8]) -> Vec<u8> {
        let rotate = Rotate(self.param);
        data.iter()
            .map(|&b| {
                let byte_sil = ByteSil::from_u8(b);
                rotate.apply_byte(byte_sil).to_u8()
            })
            .collect()
    }
    
    fn decompress_rotate(&self, data: &[u8]) -> Vec<u8> {
        // Rotação reversa (módulo 16 para evitar underflow)
        let rotate = Rotate((16 - (self.param % 16)) % 16);
        data.iter()
            .map(|&b| {
                let byte_sil = ByteSil::from_u8(b);
                rotate.apply_byte(byte_sil).to_u8()
            })
            .collect()
    }
    
    // ───────────────────────────────────────────────────────────────────────
    // XOR + Rotate Compression (melhor compressão)
    // ───────────────────────────────────────────────────────────────────────
    
    fn compress_xor_rotate(&self, data: &[u8]) -> Vec<u8> {
        let xor = Xor(self.param);
        let rotate = Rotate(self.param >> 2); // Usa bits superiores para rotação
        
        data.iter()
            .map(|&b| {
                let byte_sil = ByteSil::from_u8(b);
                let xored = xor.apply_byte(byte_sil);
                rotate.apply_byte(xored).to_u8()
            })
            .collect()
    }
    
    fn decompress_xor_rotate(&self, data: &[u8]) -> Vec<u8> {
        let xor = Xor(self.param);
        let rotation = (self.param >> 2) % 16;
        let rotate_back = Rotate((16 - rotation) % 16);
        
        data.iter()
            .map(|&b| {
                let byte_sil = ByteSil::from_u8(b);
                let rotated_back = rotate_back.apply_byte(byte_sil);
                xor.apply_byte(rotated_back).to_u8()
            })
            .collect()
    }
    
    // ───────────────────────────────────────────────────────────────────────
    // Adaptive Compression (analisa e escolhe melhor método)
    // ───────────────────────────────────────────────────────────────────────
    
    fn compress_adaptive(&self, data: &[u8]) -> Vec<u8> {
        if data.len() < 64 {
            return data.to_vec(); // Muito pequeno para analisar
        }
        
        // Analisa sample dos dados
        let sample = &data[..data.len().min(256)];
        
        // Testa diferentes modos
        let mut best_mode = CompressionMode::None;
        let mut best_param = 0u8;
        let mut best_entropy = f64::MAX;
        
        // Testa XOR com diferentes chaves
        for key in [0x5A, 0xA5, 0x3C, 0xC3, 0x69, 0x96] {
            let compressed = JsilCompressor::new(CompressionMode::Xor, key)
                .compress_xor(sample);
            let entropy = calculate_entropy(&compressed);
            
            if entropy < best_entropy {
                best_entropy = entropy;
                best_mode = CompressionMode::Xor;
                best_param = key;
            }
        }
        
        // Testa rotações
        for rot in [2, 4, 6, 8] {
            let compressed = JsilCompressor::new(CompressionMode::Rotate, rot)
                .compress_rotate(sample);
            let entropy = calculate_entropy(&compressed);
            
            if entropy < best_entropy {
                best_entropy = entropy;
                best_mode = CompressionMode::Rotate;
                best_param = rot;
            }
        }
        
        // Aplica melhor modo encontrado
        let compressor = JsilCompressor::new(best_mode, best_param);
        
        // Adiciona metadados no início (1 byte de modo + 1 byte de param)
        let mut result = vec![best_mode as u8, best_param];
        result.extend(compressor.compress(data));
        result
    }
    
    fn decompress_adaptive(&self, data: &[u8]) -> Vec<u8> {
        if data.len() < 2 {
            return data.to_vec();
        }
        
        // Lê metadados
        let mode = CompressionMode::from_byte(data[0])
            .unwrap_or(CompressionMode::None);
        let param = data[1];
        
        let compressor = JsilCompressor::new(mode, param);
        compressor.decompress(&data[2..])
    }
}

/// Calcula entropia de Shannon (para seleção adaptativa)
fn calculate_entropy(data: &[u8]) -> f64 {
    let mut freq = [0u32; 256];
    for &byte in data {
        freq[byte as usize] += 1;
    }
    
    let len = data.len() as f64;
    let mut entropy = 0.0;
    
    for &count in &freq {
        if count > 0 {
            let p = count as f64 / len;
            entropy -= p * p.log2();
        }
    }
    
    entropy
}

// ═══════════════════════════════════════════════════════════════════════════
// WRITER JSIL
// ═══════════════════════════════════════════════════════════════════════════

pub struct JsilWriter {
    compressor: JsilCompressor,
    buffer: Vec<u8>,
    record_count: u32,
}

impl JsilWriter {
    pub fn new(compressor: JsilCompressor) -> Self {
        Self {
            compressor,
            buffer: Vec::new(),
            record_count: 0,
        }
    }
    
    /// Adiciona registro JSONL
    pub fn write_record<T: Serialize>(&mut self, record: &T) -> VspResult<()> {
        let json = serde_json::to_string(record)
            .map_err(|e| VspError::IoError(format!("JSON error: {}", e)))?;
        
        self.buffer.extend_from_slice(json.as_bytes());
        self.buffer.push(b'\n');
        self.record_count += 1;
        
        Ok(())
    }
    
    /// Finaliza e escreve arquivo
    pub fn finalize<W: Write>(&self, mut writer: W) -> VspResult<JsilHeader> {
        let uncompressed_size = self.buffer.len() as u32;
        let compressed_data = self.compressor.compress(&self.buffer);
        let compressed_size = compressed_data.len() as u32;
        
        // Calcula checksum
        let checksum = calculate_checksum(&compressed_data);
        
        // Cria header
        let mut header = JsilHeader::new(self.compressor.mode);
        header.compression_param = self.compressor.param;
        header.uncompressed_size = uncompressed_size;
        header.compressed_size = compressed_size;
        header.record_count = self.record_count;
        header.checksum = checksum;
        
        // Escreve header + dados comprimidos
        writer.write_all(&header.to_bytes())
            .map_err(|e| VspError::IoError(e.to_string()))?;
        writer.write_all(&compressed_data)
            .map_err(|e| VspError::IoError(e.to_string()))?;
        
        Ok(header)
    }
    
    pub fn save<P: AsRef<Path>>(&self, path: P) -> VspResult<JsilHeader> {
        let file = std::fs::File::create(path.as_ref())
            .map_err(|e| VspError::IoError(e.to_string()))?;
        self.finalize(file)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// READER JSIL (Streaming)
// ═══════════════════════════════════════════════════════════════════════════

pub struct JsilReader {
    header: JsilHeader,
    decompressed: Vec<u8>,
    offset: usize,
}

impl JsilReader {
    pub fn load<P: AsRef<Path>>(path: P) -> VspResult<Self> {
        let mut file = std::fs::File::open(path.as_ref())
            .map_err(|e| VspError::IoError(e.to_string()))?;
        
        // Lê header
        let mut header_bytes = [0u8; JsilHeader::SIZE];
        file.read_exact(&mut header_bytes)
            .map_err(|e| VspError::IoError(e.to_string()))?;
        
        let header = JsilHeader::from_bytes(&header_bytes)?;
        
        // Lê dados comprimidos
        let mut compressed = vec![0u8; header.compressed_size as usize];
        file.read_exact(&mut compressed)
            .map_err(|e| VspError::IoError(e.to_string()))?;
        
        // Verifica checksum
        let checksum = calculate_checksum(&compressed);
        if checksum != header.checksum {
            return Err(VspError::InvalidBytecode(
                format!("Checksum mismatch: expected {:016x}, got {:016x}",
                    header.checksum, checksum)
            ));
        }
        
        // Descomprime
        let compressor = JsilCompressor::new(header.compression, header.compression_param);
        let decompressed = compressor.decompress(&compressed);
        
        Ok(Self {
            header,
            decompressed,
            offset: 0,
        })
    }
    
    pub fn header(&self) -> &JsilHeader {
        &self.header
    }
    
    /// Lê próximo registro
    pub fn next_record<T: for<'de> Deserialize<'de>>(&mut self) -> VspResult<Option<T>> {
        if self.offset >= self.decompressed.len() {
            return Ok(None);
        }
        
        // Encontra próxima quebra de linha
        let line_end = self.decompressed[self.offset..]
            .iter()
            .position(|&b| b == b'\n')
            .map(|pos| self.offset + pos)
            .unwrap_or(self.decompressed.len());
        
        let line = &self.decompressed[self.offset..line_end];
        self.offset = line_end + 1;
        
        if line.is_empty() {
            return self.next_record();
        }
        
        let record = serde_json::from_slice(line)
            .map_err(|e| VspError::IoError(format!("JSON parse error: {}", e)))?;
        
        Ok(Some(record))
    }
}

// Note: JsilReader não implementa Iterator diretamente devido a constraints de tipo
// Use next_record() em um loop while

// ═══════════════════════════════════════════════════════════════════════════
// HELPERS
// ═══════════════════════════════════════════════════════════════════════════

fn calculate_checksum(data: &[u8]) -> u64 {
    // FNV-1a hash
    let mut hash: u64 = 0xcbf29ce484222325;
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

// ═══════════════════════════════════════════════════════════════════════════
// INTEGRAÇÃO COM SILC → JSIL
// ═══════════════════════════════════════════════════════════════════════════

pub struct SilcToJsil {
    compressor: JsilCompressor,
    jsonl_config: JsonlConfig,
}

impl SilcToJsil {
    pub fn new(compressor: JsilCompressor, jsonl_config: JsonlConfig) -> Self {
        Self { compressor, jsonl_config }
    }
    
    pub fn default() -> Self {
        Self::new(JsilCompressor::adaptive(), JsonlConfig::default())
    }
    
    /// Converte .silc para .jsil
    pub fn convert<P: AsRef<Path>>(
        &self,
        input: P,
        output: P,
    ) -> VspResult<JsilStats> {
        let silc = SilcFile::load(input.as_ref())?;
        
        // Gera JSONL intermediário em memória
        let mut jsonl_buffer = Vec::new();
        let converter = SilcToJsonl::new(self.jsonl_config.clone());
        let conv_stats = converter.convert_to_writer(&silc, &mut jsonl_buffer)?;
        
        // Comprime com JSIL
        let mut writer = JsilWriter::new(self.compressor.clone());
        
        // Processa cada linha JSONL
        for line in jsonl_buffer.split(|&b| b == b'\n') {
            if !line.is_empty() {
                let record: JsonlRecord = serde_json::from_slice(line)
                    .map_err(|e| VspError::IoError(format!("JSON error: {}", e)))?;
                writer.write_record(&record)?;
            }
        }
        
        let header = writer.save(output)?;
        
        Ok(JsilStats {
            records: conv_stats.records,
            instructions: conv_stats.instructions,
            symbols: conv_stats.symbols,
            uncompressed_size: header.uncompressed_size as usize,
            compressed_size: header.compressed_size as usize,
            compression_ratio: header.compression_ratio(),
        })
    }
}

#[derive(Debug)]
pub struct JsilStats {
    pub records: usize,
    pub instructions: usize,
    pub symbols: usize,
    pub uncompressed_size: usize,
    pub compressed_size: usize,
    pub compression_ratio: f64,
}

impl JsilStats {
    pub fn report(&self) -> String {
        format!(
            "Conversão JSIL concluída:\n\
             - {} registros ({} instruções, {} símbolos)\n\
             - {} bytes → {} bytes\n\
             - Taxa de compressão: {:.1}%\n\
             - Economia: {:.1}%",
            self.records,
            self.instructions,
            self.symbols,
            self.uncompressed_size,
            self.compressed_size,
            self.compression_ratio * 100.0,
            (1.0 - self.compression_ratio) * 100.0
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_jsil_compression() {
        let data = b"Hello, World! This is a test of JSIL compression.";
        
        let compressor = JsilCompressor::default();
        let compressed = compressor.compress(data);
        let decompressed = compressor.decompress(&compressed);
        
        assert_eq!(data.as_slice(), decompressed.as_slice());
    }
    
    #[test]
    fn test_jsil_header() {
        let header = JsilHeader::new(CompressionMode::XorRotate);
        let bytes = header.to_bytes();
        let parsed = JsilHeader::from_bytes(&bytes).unwrap();
        
        assert_eq!(header.magic, parsed.magic);
        assert_eq!(header.version, parsed.version);
        assert_eq!(header.compression, parsed.compression);
    }
}
