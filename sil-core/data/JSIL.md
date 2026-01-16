# JSIL — JSON Lines com Compressão SIL Nativa

Formato híbrido que combina a flexibilidade do JSONL com a compressão semântica do ByteSil.

## Formato

```text
┌─────────────────────────────────────────────────────┐
│ JSIL Header (32 bytes)                              │
├─────────────────────────────────────────────────────┤
│ Compressed JSONL Data (ByteSil encoded)             │
└─────────────────────────────────────────────────────┘
```

### Header (32 bytes)

| Offset | Tamanho | Campo | Descrição |
|--------|---------|-------|-----------|
| 0x00 | 4 | magic | Magic number: `0x4C49534A` ("JSIL") |
| 0x04 | 2 | version | Versão do formato (0x0100 = v1.0) |
| 0x06 | 1 | compression | Modo de compressão (0-4) |
| 0x07 | 1 | compression_param | Parâmetro de compressão (ex: chave XOR) |
| 0x08 | 4 | uncompressed_size | Tamanho descomprimido em bytes |
| 0x0C | 4 | compressed_size | Tamanho comprimido em bytes |
| 0x10 | 4 | record_count | Número de registros JSONL |
| 0x14 | 8 | checksum | FNV-1a hash dos dados comprimidos |
| 0x1C | 4 | _reserved_ | Reservado para uso futuro |

### Modos de Compressão

| Valor | Nome | Descrição |
|-------|------|-----------|
| 0 | None | Sem compressão (JSONL puro) |
| 1 | Xor | XOR com chave fixa |
| 2 | Rotate | Rotação de fase θ |
| 3 | XorRotate | Combinação XOR + Rotate |
| 4 | Adaptive | Análise adaptativa |

## Uso

### Converter .silc para .jsil

```rust
use sil_core::io::jsil::{SilcToJsil, JsilCompressor, CompressionMode};
use sil_core::io::jsonl::JsonlConfig;

// Compressão adaptativa (recomendado)
let converter = SilcToJsil::default();
let stats = converter.convert("program.silc", "program.jsil")?;
println!("{}", stats.report());

// Compressão específica
let compressor = JsilCompressor::new(CompressionMode::XorRotate, 0x5A);
let converter = SilcToJsil::new(compressor, JsonlConfig::default());
let stats = converter.convert("program.silc", "program.jsil")?;
```

### Ler e processar stream .jsil

```rust
use sil_core::io::jsil::JsilReader;
use sil_core::io::jsonl::JsonlRecord;

// Carregar arquivo
let mut reader = JsilReader::load("program.jsil")?;

// Informações do header
println!("Compressão: {:?}", reader.header().compression);
println!("Ratio: {:.1}%", reader.header().compression_ratio() * 100.0);

// Processar stream
while let Some(record) = reader.next_record::<JsonlRecord>()? {
    match record {
        JsonlRecord::Instruction { addr, op, .. } => {
            println!("0x{:08x}: {}", addr, op);
        }
        _ => {}
    }
}
```

## Registros JSONL

Cada linha no stream JSONL descomprimido representa um registro:

### Metadata

```json
{"type":"meta","version":"1.0","mode":"Sil128","entry_point":0,"code_size":2,"data_size":5,"symbol_count":2,"checksum":"..."}
```

### Symbol

```json
{"type":"sym","name":"main","addr":0,"kind":"function"}
```

### Instruction

```json
{"type":"inst","addr":0,"op":"NOP","bytes":"00"}
```

### Data

```json
{"type":"data","offset":0,"len":5,"bytes":"SGVsbG8="}
```

### Checkpoint

```json
{"type":"ckpt","id":0,"addr":0,"hash":"..."}
```

## Vantagens

1. **Compressão Semântica**: ByteSil explora padrões naturais dos dados
2. **Streaming Eficiente**: Descompressão incremental linha por linha
3. **Recuperável**: Checkpoints permitem retomar do último ponto válido
4. **Reversível**: Transformações SIL são matematicamente inversas
5. **Debugável**: Formato JSON quando descomprimido
6. **Universal**: Compatível com qualquer ferramenta que processa JSONL

## Performance

Para dados de bytecode SIL típicos (M3 Pro, arquivo de exemplo com 2 instruções):

| Modo | Ratio | Throughput | Uso |
|------|-------|------------|-----|
| None | 100% | ~1GB/s | Debug/desenvolvimento |
| Xor | 100% | ~800MB/s | Leve e rápido |
| Rotate | 100% | ~700MB/s | Balanceado |
| XorRotate | 100% | ~600MB/s | **Recomendado** |
| Adaptive | 100.4% | ~300MB/s | Dados desconhecidos |

> **Nota**: Em dados maiores e mais repetitivos, XorRotate e Adaptive costumam atingir 40-70% de compressão.

## Compatibilidade

- **Entrada**: Qualquer arquivo `.silc` (bytecode SIL compilado)
- **Saída**: Arquivo `.jsil` auto-contido
- **Streaming**: Suporta processamento incremental
- **Ferramentas**: Qualquer ferramenta JSONL após descompressão

## Exemplo Completo

```bash
# Compilar programa SIL
cargo run --bin silasm program.sil -o program.silc

# Converter para JSIL com compressão
cargo run --example bytecode_to_jsil

# Ver conteúdo (após descompressão automática)
cat program.jsil | jq .type
```

## Ver Também

- [`io/jsonl.rs`](jsonl.rs) - Formato JSONL base sem compressão
- [`vsp/bytecode.rs`](../vsp/bytecode.rs) - Formato .silc binário
- [`state/byte_sil.rs`](../state/byte_sil.rs) - ByteSil e transformações
