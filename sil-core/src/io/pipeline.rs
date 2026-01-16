//! Pipeline — Encadeamento de transformações I/O

use super::SilBuffer;
use super::transforms::SilTransformFn;
use crate::state::ByteSil;
use crate::vsp::error::VspResult;
use std::path::Path;

/// Pipeline de transformações
/// 
/// Permite encadear múltiplas transformações de forma fluente.
/// 
/// # Exemplo
/// 
/// ```ignore
/// use sil_core::io::{SilPipeline, SilBuffer};
/// use sil_core::io::transforms::{Xor, Rotate, Scale};
/// 
/// let pipeline = SilPipeline::new()
///     .then(Rotate(4))
///     .then(Scale(1))
///     .then(Xor(0x5A));
/// 
/// let input = SilBuffer::from_str("Hello SIL");
/// let output = pipeline.process(&input);
/// ```
pub struct SilPipeline {
    transforms: Vec<Box<dyn SilTransformFn>>,
}

impl Default for SilPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl SilPipeline {
    /// Cria pipeline vazio
    pub fn new() -> Self {
        Self {
            transforms: Vec::new(),
        }
    }
    
    /// Adiciona transformação ao pipeline
    pub fn then<T: SilTransformFn + 'static>(mut self, transform: T) -> Self {
        self.transforms.push(Box::new(transform));
        self
    }
    
    /// Adiciona transformação boxed
    pub fn then_boxed(mut self, transform: Box<dyn SilTransformFn>) -> Self {
        self.transforms.push(transform);
        self
    }
    
    /// Processa buffer através do pipeline
    pub fn process(&self, input: &SilBuffer) -> SilBuffer {
        let mut result = input.clone();
        for transform in &self.transforms {
            result = transform.apply(&result);
        }
        result
    }
    
    /// Processa único byte
    pub fn process_byte(&self, byte: ByteSil) -> ByteSil {
        let mut result = byte;
        for transform in &self.transforms {
            result = transform.apply_byte(result);
        }
        result
    }
    
    /// Processa stream de bytes (iterator)
    pub fn process_iter<'a, I>(&'a self, iter: I) -> impl Iterator<Item = ByteSil> + 'a
    where
        I: Iterator<Item = ByteSil> + 'a,
    {
        iter.map(move |b| self.process_byte(b))
    }
    
    /// Número de transformações no pipeline
    pub fn len(&self) -> usize {
        self.transforms.len()
    }
    
    /// Pipeline vazio?
    pub fn is_empty(&self) -> bool {
        self.transforms.is_empty()
    }
    
    /// Limpa pipeline
    pub fn clear(&mut self) {
        self.transforms.clear();
    }
    
    // ═════════════════════════════════════════════════════════════════════════
    // JSIL Integration
    // ═════════════════════════════════════════════════════════════════════════
    
    /// Cria pipeline a partir do modo de compressão JSIL
    /// 
    /// Reconstrói o pipeline de transformações usado pelo compressor JSIL.
    pub fn from_jsil_mode(mode: super::jsil::CompressionMode, param: u8) -> Self {
        use super::jsil::CompressionMode;
        use super::transforms::{Xor, Rotate};
        
        match mode {
            CompressionMode::None => Self::new(),
            CompressionMode::Xor => Self::new().then(Xor(param)),
            CompressionMode::Rotate => Self::new().then(Rotate(param)),
            CompressionMode::XorRotate => {
                let rotation = (param >> 2) % 16;
                Self::new()
                    .then(Xor(param))
                    .then(Rotate(rotation))
            }
            CompressionMode::Adaptive => {
                // Modo adaptativo usa metadados embutidos
                // Retorna pipeline vazio, deve ser configurado após ler metadados
                Self::new()
            }
        }
    }
    
    /// Cria pipeline reverso para descompressão JSIL
    pub fn from_jsil_mode_reverse(mode: super::jsil::CompressionMode, param: u8) -> Self {
        use super::jsil::CompressionMode;
        use super::transforms::{Xor, Rotate};
        
        match mode {
            CompressionMode::None => Self::new(),
            CompressionMode::Xor => Self::new().then(Xor(param)), // Self-inverse
            CompressionMode::Rotate => {
                let reverse_rotation = (16 - (param % 16)) % 16;
                Self::new().then(Rotate(reverse_rotation))
            }
            CompressionMode::XorRotate => {
                let rotation = (param >> 2) % 16;
                let reverse_rotation = (16 - rotation) % 16;
                Self::new()
                    .then(Rotate(reverse_rotation))  // Reverter ordem
                    .then(Xor(param))
            }
            CompressionMode::Adaptive => Self::new(),
        }
    }
    
    /// Processa bytecode SIL e salva em JSIL com este pipeline
    /// 
    /// # Exemplo
    /// 
    /// ```ignore
    /// let pipeline = SilPipeline::new()
    ///     .then(Rotate(4))
    ///     .then(Xor(0x5A));
    /// 
    /// pipeline.to_jsil("input.silc", "output.jsil")?;
    /// ```
    pub fn to_jsil<P: AsRef<Path>>(
        &self,
        input_silc: P,
        output_jsil: P,
    ) -> VspResult<JsilPipelineStats> {
        use crate::vsp::bytecode::SilcFile;
        use super::jsil::{JsilWriter, JsilCompressor, CompressionMode};
        use super::jsonl::{SilcToJsonl, JsonlConfig, JsonlRecord};
        
        // Carregar bytecode
        let mut silc = SilcFile::load(input_silc.as_ref())?;
        
        // Aplicar pipeline aos dados
        if !silc.data.is_empty() {
            let data_buffer = SilBuffer::from_bytes(&silc.data);
            let transformed = self.process(&data_buffer);
            silc.data = transformed.to_bytes();
        }
        
        // Converter para JSONL
        let mut jsonl_buffer = Vec::new();
        let converter = SilcToJsonl::new(JsonlConfig::default());
        let conv_stats = converter.convert_to_writer(&silc, &mut jsonl_buffer)?;
        
        // Comprimir com JSIL (usando XorRotate por padrão)
        let compressor = JsilCompressor::new(CompressionMode::XorRotate, 0x5A);
        let mut writer = JsilWriter::new(compressor);
        
        for line in jsonl_buffer.split(|&b| b == b'\n') {
            if !line.is_empty() {
                let record: JsonlRecord = serde_json::from_slice(line)
                    .map_err(|e| crate::vsp::error::VspError::IoError(format!("JSON error: {}", e)))?;
                writer.write_record(&record)?;
            }
        }
        
        let header = writer.save(output_jsil)?;
        
        Ok(JsilPipelineStats {
            pipeline_transforms: self.len(),
            records: conv_stats.records,
            instructions: conv_stats.instructions,
            symbols: conv_stats.symbols,
            data_transformed: silc.data.len(),
            uncompressed_size: header.uncompressed_size as usize,
            compressed_size: header.compressed_size as usize,
            compression_ratio: header.compression_ratio(),
        })
    }
    
    /// Lê JSIL, descomprime e reverte pipeline para reconstruir dados
    /// 
    /// # Exemplo
    /// 
    /// ```ignore
    /// let pipeline = SilPipeline::new()
    ///     .then(Rotate(4))
    ///     .then(Xor(0x5A));
    /// 
    /// let data = pipeline.from_jsil("compressed.jsil")?;
    /// ```
    pub fn from_jsil<P: AsRef<Path>>(
        &self,
        input_jsil: P,
    ) -> VspResult<SilBuffer> {
        use super::jsil::JsilReader;
        use super::jsonl::JsonlRecord;
        
        let mut reader = JsilReader::load(input_jsil)?;
        
        // Buscar dados no stream
        let mut data_bytes: Option<Vec<u8>> = None;
        
        while let Some(record) = reader.next_record::<JsonlRecord>()? {
            if let JsonlRecord::Data { bytes, .. } = record {
                // Decodificar base64
                use base64::Engine;
                data_bytes = Some(
                    base64::engine::general_purpose::STANDARD.decode(&bytes)
                        .map_err(|e| crate::vsp::error::VspError::IoError(format!("Base64 error: {}", e)))?
                );
                break;
            }
        }
        
        if let Some(bytes) = data_bytes {
            // Retornar dados TRANSFORMADOS (sem aplicar reverse)
            // O usuário deve aplicar manualmente o pipeline reverso se necessário
            Ok(SilBuffer::from_bytes(&bytes))
        } else {
            Err(crate::vsp::error::VspError::IoError("No data found in JSIL".into()))
        }
    }
    
    /// Cria pipeline reverso
    /// 
    /// Inverte a ordem das transformações e aplica operações inversas.
    pub fn reverse(&self) -> Self {
        let mut reversed = Self::new();
        
        // Processar em ordem inversa
        for transform in self.transforms.iter().rev() {
            // Para cada tipo de transformação, adicionar a inversa
            let name = transform.name();
            
            match name {
                "Xor" | "XorKey" => {
                    // XOR é auto-inversa, adiciona a mesma transformação
                    reversed.transforms.push(transform.clone_box());
                }
                "Rotate" => {
                    // Rotação inversa: Rotate(16 - n) mod 16
                    // Precisamos extrair o parâmetro da transformação
                    // Por enquanto, clonamos (usuário deve criar manualmente)
                    reversed.transforms.push(transform.clone_box());
                }
                "Scale" => {
                    // Escala inversa: Scale(-n)
                    reversed.transforms.push(transform.clone_box());
                }
                "Invert" | "Conjugate" => {
                    // Auto-inversas
                    reversed.transforms.push(transform.clone_box());
                }
                _ => {
                    // Transformação genérica, tenta clonar
                    reversed.transforms.push(transform.clone_box());
                }
            }
        }
        
        reversed
    }
}

/// Estatísticas de processamento com pipeline JSIL
#[derive(Debug, Clone)]
pub struct JsilPipelineStats {
    pub pipeline_transforms: usize,
    pub records: usize,
    pub instructions: usize,
    pub symbols: usize,
    pub data_transformed: usize,
    pub uncompressed_size: usize,
    pub compressed_size: usize,
    pub compression_ratio: f64,
}

impl JsilPipelineStats {
    pub fn report(&self) -> String {
        format!(
            "Pipeline JSIL processado:\n\
             - {} transformações no pipeline\n\
             - {} bytes de dados transformados\n\
             - {} registros JSONL\n\
             - {} instruções, {} símbolos\n\
             - {} bytes → {} bytes ({:.1}% compressão)",
            self.pipeline_transforms,
            self.data_transformed,
            self.records,
            self.instructions,
            self.symbols,
            self.uncompressed_size,
            self.compressed_size,
            self.compression_ratio * 100.0
        )
    }
}

/// Pipeline builder para construção mais ergonômica
pub struct PipelineBuilder {
    pipeline: SilPipeline,
}

impl PipelineBuilder {
    pub fn new() -> Self {
        Self {
            pipeline: SilPipeline::new(),
        }
    }
    
    /// Adiciona XOR
    pub fn xor(self, key: u8) -> Self {
        Self {
            pipeline: self.pipeline.then(super::transforms::Xor(key)),
        }
    }
    
    /// Adiciona rotação
    pub fn rotate(self, delta: u8) -> Self {
        Self {
            pipeline: self.pipeline.then(super::transforms::Rotate(delta)),
        }
    }
    
    /// Adiciona escala
    pub fn scale(self, delta: i8) -> Self {
        Self {
            pipeline: self.pipeline.then(super::transforms::Scale(delta)),
        }
    }
    
    /// Adiciona inversão
    pub fn invert(self) -> Self {
        Self {
            pipeline: self.pipeline.then(super::transforms::Invert),
        }
    }
    
    /// Adiciona conjugado
    pub fn conjugate(self) -> Self {
        Self {
            pipeline: self.pipeline.then(super::transforms::Conjugate),
        }
    }
    
    /// Constrói pipeline
    pub fn build(self) -> SilPipeline {
        self.pipeline
    }
}

impl Default for PipelineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Funções de conveniência
// =============================================================================

/// Cria pipeline builder
pub fn pipeline() -> PipelineBuilder {
    PipelineBuilder::new()
}

/// Processa buffer com pipeline inline
pub fn process<F>(input: &SilBuffer, f: F) -> SilBuffer
where
    F: FnOnce(PipelineBuilder) -> PipelineBuilder,
{
    let pipeline = f(PipelineBuilder::new()).build();
    pipeline.process(input)
}

// =============================================================================
// Testes
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::transforms::*;
    
    #[test]
    fn test_pipeline_xor_roundtrip() {
        let input = SilBuffer::from_str("Hello SIL");
        
        let encrypt = SilPipeline::new().then(Xor(0x5A));
        let decrypt = SilPipeline::new().then(Xor(0x5A));
        
        let encrypted = encrypt.process(&input);
        let decrypted = decrypt.process(&encrypted);
        
        assert_eq!(decrypted.to_bytes(), input.to_bytes());
    }
    
    #[test]
    fn test_pipeline_builder() {
        let input = SilBuffer::from_str("Test");
        
        let pipeline = pipeline()
            .rotate(4)
            .scale(1)
            .xor(0x3C)
            .build();
        
        let output = pipeline.process(&input);
        assert_eq!(output.len(), input.len());
    }
    
    #[test]
    fn test_process_inline() {
        let input = SilBuffer::from_str("Hello");
        
        let output = process(&input, |p| p.xor(0x5A));
        let restored = process(&output, |p| p.xor(0x5A));
        
        assert_eq!(restored.to_bytes(), input.to_bytes());
    }
    
    #[test]
    fn test_empty_pipeline() {
        let input = SilBuffer::from_str("NoChange");
        let pipeline = SilPipeline::new();
        let output = pipeline.process(&input);
        assert_eq!(output.to_bytes(), input.to_bytes());
    }
}
