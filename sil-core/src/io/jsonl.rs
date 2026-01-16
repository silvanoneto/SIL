//! # Conversão de Bytecode SIL para JSONL
//!
//! Transforma arquivos .silc em formato JSONL para streaming e processamento.

use crate::vsp::bytecode::{SilcFile, Symbol, SymbolKind};
use crate::vsp::instruction::Instruction;
use crate::vsp::error::{VspError, VspResult};
use std::io::{BufWriter, BufRead, Write};
use std::path::Path;
use serde::{Serialize, Deserialize};

// ═══════════════════════════════════════════════════════════════════════════
// ESTRUTURAS JSONL
// ═══════════════════════════════════════════════════════════════════════════

/// Tipo de registro JSONL
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum JsonlRecord {
    /// Metadados do arquivo
    #[serde(rename = "meta")]
    Metadata {
        version: String,
        mode: String,
        entry_point: u32,
        code_size: u32,
        data_size: u32,
        symbol_count: u32,
        checksum: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        source_file: Option<String>,
    },
    
    /// Instrução decodificada
    #[serde(rename = "inst")]
    Instruction {
        /// Endereço da instrução
        addr: u32,
        /// Opcode mnemônico
        op: String,
        /// Bytes raw (hex)
        bytes: String,
        /// Argumentos decodificados
        #[serde(skip_serializing_if = "Option::is_none")]
        args: Option<InstructionArgs>,
        /// Linha no código fonte
        #[serde(skip_serializing_if = "Option::is_none")]
        line: Option<u32>,
    },
    
    /// Símbolo
    #[serde(rename = "sym")]
    Symbol {
        name: String,
        addr: u32,
        kind: String,
    },
    
    /// Dado estático
    #[serde(rename = "data")]
    Data {
        /// Offset no segmento de dados
        offset: u32,
        /// Comprimento dos dados
        len: u32,
        /// Dados codificados em base64
        bytes: String,
    },
    
    /// Checkpoint para recuperação
    #[serde(rename = "ckpt")]
    Checkpoint {
        /// ID do checkpoint
        id: u32,
        /// Endereço
        addr: u32,
        /// Hash incremental
        hash: String,
    },
}

/// Argumentos de instrução decodificados
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionArgs {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reg: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reg_a: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reg_b: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub imm: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub addr: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

// ═══════════════════════════════════════════════════════════════════════════
// CONVERSOR PRINCIPAL
// ═══════════════════════════════════════════════════════════════════════════

/// Configurações de exportação JSONL
#[derive(Debug, Clone)]
pub struct JsonlConfig {
    /// Tamanho do chunk de dados (default: 64 bytes)
    pub data_chunk_size: usize,
    /// Intervalo de checkpoints (default: 256 instruções)
    pub checkpoint_interval: u32,
    /// Incluir informações de debug
    pub include_debug: bool,
    /// Pretty-print JSON (diminui performance)
    pub pretty: bool,
}

impl Default for JsonlConfig {
    fn default() -> Self {
        Self {
            data_chunk_size: 64,
            checkpoint_interval: 256,
            include_debug: true,
            pretty: false,
        }
    }
}

/// Conversor de bytecode SIL para JSONL
pub struct SilcToJsonl {
    config: JsonlConfig,
    hash_state: u64, // Hash incremental para checkpoints
}

impl SilcToJsonl {
    /// Cria novo conversor
    pub fn new(config: JsonlConfig) -> Self {
        Self {
            config,
            hash_state: 0xcbf29ce484222325, // FNV-1a seed
        }
    }
    
    /// Converte arquivo .silc para JSONL
    pub fn convert_file<P: AsRef<Path>>(
        &self,
        input: P,
        output: P,
    ) -> VspResult<ConversionStats> {
        let silc = SilcFile::load(input.as_ref())?;
        let file = std::fs::File::create(output.as_ref())
            .map_err(|e| VspError::IoError(e.to_string()))?;
        
        self.convert_to_writer(&silc, file)
    }
    
    /// Converte para um Writer (permite streaming)
    pub fn convert_to_writer<W: Write>(
        &self,
        silc: &SilcFile,
        writer: W,
    ) -> VspResult<ConversionStats> {
        let mut buf_writer = BufWriter::with_capacity(64 * 1024, writer);
        let mut stats = ConversionStats::default();
        
        // 1. Escreve metadados
        self.write_metadata(&mut buf_writer, silc)?;
        stats.records += 1;
        
        // 2. Escreve símbolos
        for symbol in &silc.symbols {
            self.write_symbol(&mut buf_writer, symbol)?;
            stats.symbols += 1;
            stats.records += 1;
        }
        
        // 3. Escreve instruções com checkpoints
        let mut checkpoint_counter = 0;
        let mut offset = 0;
        
        while offset < silc.code.len() {
            // Checkpoint periódico
            if checkpoint_counter % self.config.checkpoint_interval == 0 {
                self.write_checkpoint(&mut buf_writer, checkpoint_counter, offset as u32)?;
                stats.records += 1;
            }
            
            // Decodifica e escreve instrução
            let inst = Instruction::decode(&silc.code[offset..])?;
            let line = self.find_line(silc, offset as u32);
            let label = self.find_label(silc, offset as u32);
            
            self.write_instruction(
                &mut buf_writer,
                offset as u32,
                &inst,
                line,
                label,
            )?;
            
            stats.instructions += 1;
            stats.records += 1;
            offset += inst.size();
            checkpoint_counter += 1;
        }
        
        // 4. Escreve dados em chunks
        let mut data_offset = 0;
        while data_offset < silc.data.len() {
            let chunk_size = self.config.data_chunk_size.min(silc.data.len() - data_offset);
            let chunk = &silc.data[data_offset..data_offset + chunk_size];
            
            self.write_data_chunk(&mut buf_writer, data_offset as u32, chunk)?;
            stats.data_chunks += 1;
            stats.records += 1;
            data_offset += chunk_size;
        }
        
        // 5. Checkpoint final
        self.write_checkpoint(&mut buf_writer, checkpoint_counter, offset as u32)?;
        stats.records += 1;
        
        buf_writer.flush()
            .map_err(|e| VspError::IoError(e.to_string()))?;
        
        Ok(stats)
    }
    
    // ───────────────────────────────────────────────────────────────────────
    // ESCRITA DE REGISTROS
    // ───────────────────────────────────────────────────────────────────────
    
    fn write_metadata<W: Write>(
        &self,
        writer: &mut W,
        silc: &SilcFile,
    ) -> VspResult<()> {
        let record = JsonlRecord::Metadata {
            version: format!("{}.{}", 
                silc.header.version >> 8,
                silc.header.version & 0xFF),
            mode: format!("{:?}", silc.header.mode),
            entry_point: silc.header.entry_point,
            code_size: silc.header.code_size,
            data_size: silc.header.data_size,
            symbol_count: silc.symbols.len() as u32,
            checksum: format!("{:016x}", silc.header.checksum),
            source_file: silc.debug_info.as_ref().map(|d| d.source_file.clone()),
        };
        
        self.write_record(writer, &record)
    }
    
    fn write_symbol<W: Write>(
        &self,
        writer: &mut W,
        symbol: &Symbol,
    ) -> VspResult<()> {
        let record = JsonlRecord::Symbol {
            name: symbol.name.clone(),
            addr: symbol.address,
            kind: match symbol.kind {
                SymbolKind::Label => "label".into(),
                SymbolKind::Data => "data".into(),
                SymbolKind::Transform => "transform".into(),
                SymbolKind::Function => "function".into(),
            },
        };
        
        self.write_record(writer, &record)
    }
    
    fn write_instruction<W: Write>(
        &self,
        writer: &mut W,
        addr: u32,
        inst: &Instruction,
        line: Option<u32>,
        label: Option<String>,
    ) -> VspResult<()> {
        let args = label.map(|l| InstructionArgs {
            reg: None,
            reg_a: None,
            reg_b: None,
            imm: None,
            addr: None,
            label: Some(l),
        });
        
        let record = JsonlRecord::Instruction {
            addr,
            op: format!("{:?}", inst.opcode).to_uppercase(),
            bytes: hex::encode(inst.raw_bytes()),
            args,
            line: if self.config.include_debug { line } else { None },
        };
        
        self.write_record(writer, &record)
    }
    
    fn write_data_chunk<W: Write>(
        &self,
        writer: &mut W,
        offset: u32,
        chunk: &[u8],
    ) -> VspResult<()> {
        let record = JsonlRecord::Data {
            offset,
            len: chunk.len() as u32,
            bytes: base64_encode(chunk),
        };
        
        self.write_record(writer, &record)
    }
    
    fn write_checkpoint<W: Write>(
        &self,
        writer: &mut W,
        id: u32,
        addr: u32,
    ) -> VspResult<()> {
        let record = JsonlRecord::Checkpoint {
            id,
            addr,
            hash: format!("{:016x}", self.hash_state),
        };
        
        self.write_record(writer, &record)
    }
    
    fn write_record<W: Write>(
        &self,
        writer: &mut W,
        record: &JsonlRecord,
    ) -> VspResult<()> {
        let json = if self.config.pretty {
            serde_json::to_string_pretty(record)
        } else {
            serde_json::to_string(record)
        }.map_err(|e| VspError::IoError(format!("JSON error: {}", e)))?;
        
        writeln!(writer, "{}", json)
            .map_err(|e| VspError::IoError(e.to_string()))?;
        
        Ok(())
    }
    
    // ───────────────────────────────────────────────────────────────────────
    // HELPERS
    // ───────────────────────────────────────────────────────────────────────
    
    fn find_line(&self, silc: &SilcFile, addr: u32) -> Option<u32> {
        if !self.config.include_debug {
            return None;
        }
        
        silc.debug_info.as_ref()?.line_map
            .iter()
            .rev()
            .find(|(_, a)| *a <= addr)
            .map(|(line, _)| *line)
    }
    
    fn find_label(&self, silc: &SilcFile, addr: u32) -> Option<String> {
        silc.symbols
            .iter()
            .find(|s| s.address == addr && matches!(s.kind, SymbolKind::Label | SymbolKind::Function))
            .map(|s| s.name.clone())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// ESTATÍSTICAS
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Default)]
pub struct ConversionStats {
    pub records: usize,
    pub instructions: usize,
    pub symbols: usize,
    pub data_chunks: usize,
}

impl ConversionStats {
    pub fn report(&self) -> String {
        format!(
            "Conversão JSONL concluída:\n\
             - {} registros totais\n\
             - {} instruções\n\
             - {} símbolos\n\
             - {} chunks de dados",
            self.records, self.instructions, self.symbols, self.data_chunks
        )
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// STREAMING READER
// ═══════════════════════════════════════════════════════════════════════════

pub struct JsonlStreamReader<R: BufRead> {
    reader: R,
    line_buffer: String,
}

impl<R: BufRead> JsonlStreamReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            line_buffer: String::with_capacity(1024),
        }
    }
    
    /// Lê próximo registro do stream
    pub fn next_record(&mut self) -> VspResult<Option<JsonlRecord>> {
        self.line_buffer.clear();
        
        let bytes_read = self.reader.read_line(&mut self.line_buffer)
            .map_err(|e| VspError::IoError(e.to_string()))?;
        
        if bytes_read == 0 {
            return Ok(None);
        }
        
        let record = serde_json::from_str(self.line_buffer.trim())
            .map_err(|e| VspError::IoError(format!("JSON parse error: {}", e)))?;
        
        Ok(Some(record))
    }
}

impl<R: BufRead> Iterator for JsonlStreamReader<R> {
    type Item = VspResult<JsonlRecord>;
    
    fn next(&mut self) -> Option<Self::Item> {
        match self.next_record() {
            Ok(Some(record)) => Some(Ok(record)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// HELPERS
// ═══════════════════════════════════════════════════════════════════════════

fn base64_encode(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vsp::state::SilMode;
    
    #[test]
    fn test_jsonl_metadata() {
        let mut silc = SilcFile::new(SilMode::Sil128);
        silc.add_symbol("main".into(), 0, SymbolKind::Function);
        
        let mut converter = SilcToJsonl::new(JsonlConfig::default());
        let mut output = Vec::new();
        
        let stats = converter.convert_to_writer(&silc, &mut output).unwrap();
        
        assert!(stats.records > 0);
        assert_eq!(stats.symbols, 1);
        
        // Verifica formato JSONL
        let lines: Vec<&str> = std::str::from_utf8(&output)
            .unwrap()
            .lines()
            .collect();
        
        assert!(lines.len() >= 2);
        
        // Primeira linha deve ser metadata
        let first: JsonlRecord = serde_json::from_str(lines[0]).unwrap();
        assert!(matches!(first, JsonlRecord::Metadata { .. }));
    }
}
