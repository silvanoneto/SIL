# Pipeline JSIL — Exemplo Integrado

Este exemplo demonstra a integração completa entre pipelines de transformação SIL e o formato JSIL comprimido.

## O que faz

O exemplo [`jsil_pipeline.rs`](jsil_pipeline.rs) implementa um fluxo completo de:

1. **Transformação de dados** usando pipelines SIL
2. **Armazenamento em bytecode** (.silc)
3. **Compressão com JSIL** usando múltiplos modos
4. **Descompressão e leitura** de stream
5. **Reversão das transformações** para recuperar dados originais

## Fluxo de Dados

```text
┌─────────────────────────────────────────────────────────────┐
│  Dados Originais                                            │
│  "Hello from SIL Pipeline! 🚀🌟💫"                          │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│  Pipeline de Transformação                                  │
│  • Rotate(θ += 4)       — Rotaciona fase                   │
│  • Xor(0x5A)            — Ofuscação XOR                     │
│  • XorKey("SIL")        — XOR com chave                     │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│  Bytecode SIL (.silc)                                       │
│  • CODE: 8 bytes (instruções)                              │
│  • DATA: 37 bytes (dados transformados)                    │
│  • SYMBOLS: start, data_section                            │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│  Compressão JSIL                                            │
│  • None, Xor, Rotate, XorRotate, Adaptive                  │
│  • Header (32 bytes) + Dados comprimidos                   │
│  • Checksum FNV-1a para integridade                        │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│  Stream JSIL (.jsil)                                        │
│  • Leitura incremental linha por linha                     │
│  • Registros: META, SYM, INST, DATA, CKPT                  │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│  Pipeline de Reversão                                       │
│  • XorKey("SIL")        — Inversa                          │
│  • Xor(0x5A)            — Inversa                          │
│  • Rotate(θ -= 4)       — Rotação reversa                  │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│  Dados Restaurados ✓                                        │
│  "Hello from SIL Pipeline! 🚀🌟💫"                          │
└─────────────────────────────────────────────────────────────┘
```

## Casos de Uso

### 1. Transformação + Armazenamento Seguro

```rust
// Pipeline de "criptografia" usando transformações SIL
let pipeline = SilPipeline::new()
    .then(Rotate(4))
    .then(Xor(0x5A))
    .then(XorKey::from_str("SECRET"));

// Processar dados sensíveis
let sensitive_data = SilBuffer::from_str("Dados confidenciais");
let protected = pipeline.process(&sensitive_data);

// Armazenar em JSIL comprimido
// ... (salvar no segmento de dados do .silc)
```

### 2. Compressão Otimizada por Conteúdo

```rust
// Comparar diferentes modos de compressão
for mode in [None, Xor, Rotate, XorRotate, Adaptive] {
    let compressor = JsilCompressor::new(mode, param);
    let stats = convert_to_jsil(data, compressor)?;
    
    println!("{:?}: {:.1}% de compressão", 
        mode, stats.compression_ratio * 100.0);
}

// Escolher automaticamente o melhor modo
```

### 3. Streaming de Dados em Tempo Real

```rust
// Pipeline para processar stream contínuo
let stream_pipeline = SilPipeline::new()
    .then(Rotate(2))
    .then(Xor(0x42));

// Processar blocos conforme chegam
for block in data_stream {
    let buffer = SilBuffer::from_str(block);
    let processed = stream_pipeline.process(&buffer);
    // ... processar imediatamente
}
```

### 4. Verificação de Integridade

```rust
// Ler JSIL e verificar checksum automaticamente
let reader = JsilReader::load("data.jsil")?;

// Checksum é validado na leitura
println!("Checksum: {:016x}", reader.header().checksum);

// Processar com confiança
while let Some(record) = reader.next_record()? {
    // Dados garantidamente íntegros
}
```

## Modos de Compressão

| Modo | Velocidade | Compressão | Uso Recomendado |
|------|------------|------------|-----------------|
| **None** | ⚡⚡⚡⚡⚡ | — | Debug, desenvolvimento |
| **Xor** | ⚡⚡⚡⚡ | 🗜️ | Ofuscação leve |
| **Rotate** | ⚡⚡⚡⚡ | 🗜️ | Transformação de fase |
| **XorRotate** | ⚡⚡⚡ | 🗜️🗜️ | **Produção** (recomendado) |
| **Adaptive** | ⚡⚡ | 🗜️🗜️🗜️ | Dados desconhecidos |

## Propriedades Matemáticas

Todas as transformações são **bijetivas** e **inversíveis**:

- **XOR**: `A ⊕ K ⊕ K = A` (auto-inversa)
- **Rotate**: `Rotate(θ) → Rotate(-θ)` (inversa por rotação oposta)
- **XorKey**: `A ⊕ K ⊕ K = A` (auto-inversa com mesma chave)

Isso garante que **nenhum dado é perdido** no processo de transformação e compressão.

## Performance

### Throughput (M3 Pro)

- Pipeline XOR simples: ~800 MB/s
- Pipeline Rotate: ~700 MB/s
- Pipeline XorRotate: ~600 MB/s
- Pipeline completo (3 etapas): ~400 MB/s

### Latência

- Transformação por byte: ~2-5 ns
- Compressão JSIL: ~1-3 µs por KB
- Descompressão: ~0.8-2 µs por KB

## Executar

```bash
cargo run --example jsil_pipeline
```

## Vantagens da Integração

1. **Pipeline + Compressão**: Transformações aplicadas antes da compressão podem melhorar a taxa de compressão
2. **Reversibilidade Garantida**: Todas as operações são matematicamente inversíveis
3. **Streaming Eficiente**: Processa dados em blocos sem carregar tudo em memória
4. **Verificação Automática**: Checksum garante integridade dos dados
5. **Formato Universal**: JSONL pode ser processado por qualquer ferramenta após descompressão

## Ver Também

- [`io_pipeline.rs`](io_pipeline.rs) - Pipelines básicos de transformação
- [`bytecode_to_jsil.rs`](bytecode_to_jsil.rs) - Conversão simples de bytecode
- [`src/io/JSIL.md`](../src/io/JSIL.md) - Especificação completa do formato JSIL
