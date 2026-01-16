//! Formato de bytecode .silc
//!
//! Estrutura do arquivo compilado SIL.

use super::state::SilMode;
use super::error::{VspError, VspResult};

/// Magic number: "SILC"
pub const SILC_MAGIC: u32 = 0x434C4953; // "SILC" em little-endian

/// Versão do formato
pub const SILC_VERSION: u16 = 0x0100; // v1.0

/// Header do arquivo .silc
#[derive(Debug, Clone)]
pub struct SilcHeader {
    /// Magic number (deve ser SILC_MAGIC)
    pub magic: u32,
    /// Versão do formato
    pub version: u16,
    /// Modo SIL do programa
    pub mode: SilMode,
    /// Tamanho do segmento de código
    pub code_size: u32,
    /// Tamanho do segmento de dados
    pub data_size: u32,
    /// Tamanho da tabela de símbolos
    pub symbol_size: u32,
    /// Entry point (endereço de main)
    pub entry_point: u32,
    /// Checksum (SHA-256 truncado)
    pub checksum: u64,
}

impl SilcHeader {
    /// Tamanho do header em bytes
    pub const SIZE: usize = 32;
    
    /// Cria header padrão
    pub fn new(mode: SilMode) -> Self {
        Self {
            magic: SILC_MAGIC,
            version: SILC_VERSION,
            mode,
            code_size: 0,
            data_size: 0,
            symbol_size: 0,
            entry_point: 0,
            checksum: 0,
        }
    }
    
    /// Serializa header
    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut bytes = [0u8; Self::SIZE];
        
        bytes[0..4].copy_from_slice(&self.magic.to_le_bytes());
        bytes[4..6].copy_from_slice(&self.version.to_le_bytes());
        bytes[6..8].copy_from_slice(&(self.mode as u16).to_le_bytes());
        bytes[8..12].copy_from_slice(&self.code_size.to_le_bytes());
        bytes[12..16].copy_from_slice(&self.data_size.to_le_bytes());
        bytes[16..20].copy_from_slice(&self.symbol_size.to_le_bytes());
        bytes[20..24].copy_from_slice(&self.entry_point.to_le_bytes());
        bytes[24..32].copy_from_slice(&self.checksum.to_le_bytes());
        
        bytes
    }
    
    /// Deserializa header
    pub fn from_bytes(bytes: &[u8]) -> VspResult<Self> {
        if bytes.len() < Self::SIZE {
            return Err(VspError::InvalidBytecode("Header too short".into()));
        }
        
        let magic = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        if magic != SILC_MAGIC {
            return Err(VspError::InvalidBytecode(format!(
                "Invalid magic: expected 0x{:08X}, got 0x{:08X}",
                SILC_MAGIC, magic
            )));
        }
        
        let version = u16::from_le_bytes([bytes[4], bytes[5]]);
        let mode_bits = u16::from_le_bytes([bytes[6], bytes[7]]);
        let mode = SilMode::from_bits(mode_bits as u8)?;
        
        Ok(Self {
            magic,
            version,
            mode,
            code_size: u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]),
            data_size: u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]),
            symbol_size: u32::from_le_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]),
            entry_point: u32::from_le_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]),
            checksum: u64::from_le_bytes([
                bytes[24], bytes[25], bytes[26], bytes[27],
                bytes[28], bytes[29], bytes[30], bytes[31],
            ]),
        })
    }
}

/// Entrada na tabela de símbolos
#[derive(Debug, Clone)]
pub struct Symbol {
    /// Nome do símbolo
    pub name: String,
    /// Endereço
    pub address: u32,
    /// Tipo
    pub kind: SymbolKind,
}

/// Tipo de símbolo
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SymbolKind {
    /// Label de código
    Label = 0,
    /// Dado estático
    Data = 1,
    /// Transformação
    Transform = 2,
    /// Função/subrotina
    Function = 3,
}

impl SymbolKind {
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0 => Some(Self::Label),
            1 => Some(Self::Data),
            2 => Some(Self::Transform),
            3 => Some(Self::Function),
            _ => None,
        }
    }
}

/// Informação de símbolo para o assembler
#[derive(Debug, Clone)]
pub struct SymbolInfo {
    /// Nome do símbolo
    pub name: String,
    /// Endereço
    pub address: u32,
    /// Símbolo global (exportado)
    pub is_global: bool,
    /// Símbolo externo (importado)
    pub is_extern: bool,
}

/// Arquivo .silc completo
#[derive(Debug, Clone)]
pub struct SilcFile {
    /// Header
    pub header: SilcHeader,
    /// Segmento de código
    pub code: Vec<u8>,
    /// Segmento de dados
    pub data: Vec<u8>,
    /// Tabela de símbolos
    pub symbols: Vec<Symbol>,
    /// Informações de debug (opcional)
    pub debug_info: Option<DebugInfo>,
}

impl SilcFile {
    /// Cria novo arquivo vazio
    pub fn new(mode: SilMode) -> Self {
        Self {
            header: SilcHeader::new(mode),
            code: Vec::new(),
            data: Vec::new(),
            symbols: Vec::new(),
            debug_info: None,
        }
    }
    
    /// Carrega de arquivo
    pub fn load(path: &std::path::Path) -> VspResult<Self> {
        let bytes = std::fs::read(path)
            .map_err(|e| VspError::IoError(e.to_string()))?;
        
        Self::from_bytes(&bytes)
    }
    
    /// Salva em arquivo
    pub fn save(&self, path: &std::path::Path) -> VspResult<()> {
        let bytes = self.to_bytes();
        std::fs::write(path, bytes)
            .map_err(|e| VspError::IoError(e.to_string()))
    }
    
    /// Serializa para bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        // Header (atualizado com tamanhos)
        let mut header = self.header.clone();
        header.code_size = self.code.len() as u32;
        header.data_size = self.data.len() as u32;
        header.symbol_size = self.serialize_symbols().len() as u32;
        
        // Calcula checksum
        header.checksum = self.calculate_checksum();
        
        bytes.extend_from_slice(&header.to_bytes());
        bytes.extend_from_slice(&self.code);
        bytes.extend_from_slice(&self.data);
        bytes.extend_from_slice(&self.serialize_symbols());
        
        // Debug info se presente
        if let Some(ref debug) = self.debug_info {
            bytes.extend_from_slice(&debug.to_bytes());
        }
        
        bytes
    }
    
    /// Deserializa de bytes
    pub fn from_bytes(bytes: &[u8]) -> VspResult<Self> {
        if bytes.len() < SilcHeader::SIZE {
            return Err(VspError::InvalidBytecode("File too short".into()));
        }
        
        let header = SilcHeader::from_bytes(bytes)?;
        
        let code_start = SilcHeader::SIZE;
        let code_end = code_start + header.code_size as usize;
        let data_start = code_end;
        let data_end = data_start + header.data_size as usize;
        let symbol_start = data_end;
        let symbol_end = symbol_start + header.symbol_size as usize;
        
        if bytes.len() < symbol_end {
            return Err(VspError::InvalidBytecode("File truncated".into()));
        }
        
        let code = bytes[code_start..code_end].to_vec();
        let data = bytes[data_start..data_end].to_vec();
        let symbols = Self::deserialize_symbols(&bytes[symbol_start..symbol_end])?;
        
        // Debug info é opcional
        let debug_info = if bytes.len() > symbol_end {
            DebugInfo::from_bytes(&bytes[symbol_end..]).ok()
        } else {
            None
        };
        
        Ok(Self {
            header,
            code,
            data,
            symbols,
            debug_info,
        })
    }
    
    /// Serializa tabela de símbolos
    fn serialize_symbols(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        // Número de símbolos
        bytes.extend_from_slice(&(self.symbols.len() as u32).to_le_bytes());
        
        for sym in &self.symbols {
            // Tamanho do nome
            bytes.extend_from_slice(&(sym.name.len() as u16).to_le_bytes());
            // Nome
            bytes.extend_from_slice(sym.name.as_bytes());
            // Endereço
            bytes.extend_from_slice(&sym.address.to_le_bytes());
            // Tipo
            bytes.push(sym.kind as u8);
        }
        
        bytes
    }
    
    /// Deserializa tabela de símbolos
    fn deserialize_symbols(bytes: &[u8]) -> VspResult<Vec<Symbol>> {
        if bytes.len() < 4 {
            return Ok(Vec::new());
        }
        
        let count = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
        let mut symbols = Vec::with_capacity(count);
        let mut offset = 4;
        
        for _ in 0..count {
            if offset + 2 > bytes.len() {
                break;
            }
            
            let name_len = u16::from_le_bytes([bytes[offset], bytes[offset + 1]]) as usize;
            offset += 2;
            
            // SECURITY FIX: Validate name_len before using it
            // Prevent integer overflow and buffer overflow
            if name_len > bytes.len().saturating_sub(offset) {
                break;
            }
            
            if offset + name_len + 5 > bytes.len() {
                break;
            }
            
            let name = String::from_utf8_lossy(&bytes[offset..offset + name_len]).to_string();
            offset += name_len;
            
            let address = u32::from_le_bytes([
                bytes[offset], bytes[offset + 1],
                bytes[offset + 2], bytes[offset + 3],
            ]);
            offset += 4;
            
            let kind = SymbolKind::from_byte(bytes[offset])
                .unwrap_or(SymbolKind::Label);
            offset += 1;
            
            symbols.push(Symbol { name, address, kind });
        }
        
        Ok(symbols)
    }
    
    /// Calcula checksum (hash simples)
    fn calculate_checksum(&self) -> u64 {
        // FNV-1a hash simplificado
        let mut hash: u64 = 0xcbf29ce484222325;
        
        for &byte in &self.code {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        
        for &byte in &self.data {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        
        hash
    }
    
    /// Busca símbolo por nome
    pub fn find_symbol(&self, name: &str) -> Option<&Symbol> {
        self.symbols.iter().find(|s| s.name == name)
    }
    
    /// Adiciona símbolo
    pub fn add_symbol(&mut self, name: String, address: u32, kind: SymbolKind) {
        self.symbols.push(Symbol { name, address, kind });
    }
}

/// Informações de debug
#[derive(Debug, Clone)]
pub struct DebugInfo {
    /// Mapeamento linha -> endereço
    pub line_map: Vec<(u32, u32)>, // (line, address)
    /// Arquivo fonte
    pub source_file: String,
}

impl DebugInfo {
    pub fn new(source_file: String) -> Self {
        Self {
            line_map: Vec::new(),
            source_file,
        }
    }
    
    pub fn add_line(&mut self, line: u32, address: u32) {
        self.line_map.push((line, address));
    }
    
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        // Tamanho do nome do arquivo
        bytes.extend_from_slice(&(self.source_file.len() as u16).to_le_bytes());
        bytes.extend_from_slice(self.source_file.as_bytes());
        
        // Número de entradas
        bytes.extend_from_slice(&(self.line_map.len() as u32).to_le_bytes());
        
        for &(line, addr) in &self.line_map {
            bytes.extend_from_slice(&line.to_le_bytes());
            bytes.extend_from_slice(&addr.to_le_bytes());
        }
        
        bytes
    }
    
    pub fn from_bytes(bytes: &[u8]) -> VspResult<Self> {
        if bytes.len() < 2 {
            return Err(VspError::InvalidBytecode("Debug info too short".into()));
        }
        
        let name_len = u16::from_le_bytes([bytes[0], bytes[1]]) as usize;
        
        // SECURITY FIX: Prevent buffer overflow by checking bounds before slicing
        if name_len > bytes.len().saturating_sub(2) {
            return Err(VspError::InvalidBytecode("Invalid debug info: name_len exceeds buffer".into()));
        }
        
        let source_file = String::from_utf8_lossy(&bytes[2..2 + name_len]).to_string();
        
        let mut offset = 2 + name_len;
        if offset + 4 > bytes.len() {
            return Ok(Self { line_map: Vec::new(), source_file });
        }
        
        let count = u32::from_le_bytes([
            bytes[offset], bytes[offset + 1],
            bytes[offset + 2], bytes[offset + 3],
        ]) as usize;
        offset += 4;
        
        let mut line_map = Vec::with_capacity(count);
        for _ in 0..count {
            if offset + 8 > bytes.len() {
                break;
            }
            
            let line = u32::from_le_bytes([
                bytes[offset], bytes[offset + 1],
                bytes[offset + 2], bytes[offset + 3],
            ]);
            offset += 4;
            
            let addr = u32::from_le_bytes([
                bytes[offset], bytes[offset + 1],
                bytes[offset + 2], bytes[offset + 3],
            ]);
            offset += 4;
            
            line_map.push((line, addr));
        }
        
        Ok(Self { line_map, source_file })
    }
    
    /// Encontra linha para endereço
    pub fn line_for_address(&self, addr: u32) -> Option<u32> {
        // Busca a última linha com endereço <= addr
        self.line_map.iter()
            .filter(|(_, a)| *a <= addr)
            .max_by_key(|(_, a)| *a)
            .map(|(line, _)| *line)
    }
    
    /// Encontra endereço para linha
    pub fn address_for_line(&self, line: u32) -> Option<u32> {
        self.line_map.iter()
            .find(|(l, _)| *l == line)
            .map(|(_, addr)| *addr)
    }
}

/// Builder para criar arquivos .silc
pub struct SilcBuilder {
    file: SilcFile,
    current_address: u32,
}

impl SilcBuilder {
    /// Cria novo builder com modo especificado
    pub fn with_mode(mode: SilMode) -> Self {
        Self {
            file: SilcFile::new(mode),
            current_address: 0,
        }
    }
    
    /// Cria novo builder com modo default (SIL-128)
    pub fn new() -> Self {
        Self::with_mode(SilMode::Sil128)
    }
    
    /// Define entry point
    pub fn entry(mut self, symbol: &str) -> Self {
        // Será resolvido em build()
        self.file.add_symbol(format!("__entry_{}", symbol), 0, SymbolKind::Function);
        self
    }
    
    /// Adiciona label
    pub fn label(mut self, name: &str) -> Self {
        self.file.add_symbol(name.to_string(), self.current_address, SymbolKind::Label);
        self
    }
    
    /// Adiciona código
    pub fn code(mut self, bytes: impl AsRef<[u8]>) -> Self {
        let bytes = bytes.as_ref();
        self.file.code.extend_from_slice(bytes);
        self.current_address += bytes.len() as u32;
        self
    }
    
    /// Adiciona dado
    pub fn data(mut self, bytes: impl AsRef<[u8]>) -> Self {
        self.file.data.extend_from_slice(bytes.as_ref());
        self
    }
    
    /// Adiciona estado inicial
    pub fn state(mut self, name: &str, layers: &[u8; 16]) -> Self {
        let addr = self.file.data.len() as u32;
        self.file.add_symbol(name.to_string(), addr, SymbolKind::Data);
        self.file.data.extend_from_slice(layers);
        self
    }
    
    /// Adiciona símbolo com informação completa
    pub fn symbol(mut self, info: SymbolInfo) -> Self {
        let kind = if info.is_extern {
            SymbolKind::Label // Externos serão resolvidos em link time
        } else {
            SymbolKind::Label
        };
        self.file.add_symbol(info.name, info.address, kind);
        self
    }
    
    /// Define modo SIL
    pub fn mode(mut self, mode: SilMode) -> Self {
        self.file.header.mode = mode;
        self
    }
    
    /// Constrói arquivo
    pub fn build(mut self) -> SilcFile {
        // Resolve entry point
        for sym in &self.file.symbols {
            if sym.name.starts_with("__entry_") {
                let target_name = &sym.name[8..];
                if let Some(target) = self.file.symbols.iter()
                    .find(|s| s.name == target_name) {
                    self.file.header.entry_point = target.address;
                    break;
                }
            }
        }
        
        // Remove símbolos temporários
        self.file.symbols.retain(|s| !s.name.starts_with("__"));
        
        self.file
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_header_roundtrip() {
        let header = SilcHeader::new(SilMode::Sil128);
        let bytes = header.to_bytes();
        let header2 = SilcHeader::from_bytes(&bytes).unwrap();
        
        assert_eq!(header.magic, header2.magic);
        assert_eq!(header.mode, header2.mode);
    }
    
    #[test]
    fn test_silc_file_roundtrip() {
        let mut file = SilcFile::new(SilMode::Sil64);
        file.code = vec![0x00, 0x01, 0x02];
        file.data = vec![0xFF; 16];
        file.add_symbol("main".to_string(), 0, SymbolKind::Function);
        
        let bytes = file.to_bytes();
        let file2 = SilcFile::from_bytes(&bytes).unwrap();
        
        assert_eq!(file.code, file2.code);
        assert_eq!(file.data, file2.data);
        assert_eq!(file.symbols.len(), file2.symbols.len());
    }
    
    #[test]
    fn test_builder() {
        let file = SilcBuilder::with_mode(SilMode::Sil128)
            .entry("main")
            .label("main")
            .code(&[0x00, 0x01]) // NOP, HLT
            .state("initial", &[0; 16])
            .build();
        
        assert_eq!(file.header.entry_point, 0);
        assert_eq!(file.code.len(), 2);
        assert_eq!(file.data.len(), 16);
    }
}
