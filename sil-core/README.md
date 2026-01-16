# 🌀 SIL-Core

> *"Linguagem intermediária otimizada para processamento de sinais complexos em representação log-polar."*

## O Padrão SIL

**SIL** = **Signal Intermediate Language** — Linguagem intermediária para processamento de sinais complexos

SIL é um design pattern onde:

1. Todo estado é um **vetor de 16 camadas**
2. Cada camada é um **número complexo** (ρ, θ)
3. O programa é uma **transformação de estados**
4. O ciclo é **fechado** (output → input)

```
┌─────────────────────────────────────────────────────────────────┐
│                      ESTADO SIL (128 bits)                      │
├─────────────────────────────────────────────────────────────────┤
│  L(0)  L(1)  L(2)  L(3)  L(4)  L(5)  L(6)  L(7)               │
│  ════  ════  ════  ════  ════  ════  ════  ════               │
│  FOT   ACU   OLF   GUS   DER   ELE   PSI   AMB                │
│  ◄───────── PERCEPÇÃO ─────────►◄─── PROCESSO ──►              │
│                                                                 │
│  L(8)  L(9)  L(A)  L(B)  L(C)  L(D)  L(E)  L(F)               │
│  ════  ════  ════  ════  ════  ════  ════  ════               │
│  CIB   GEO   COS   SIN   QUA   SUP   ENT   COL                │
│  ◄─── INTERAÇÃO ───►◄── EMERGE ─►◄────── META ──────►          │
└─────────────────────────────────────────────────────────────────┘
```

## Estrutura do Projeto

```
sil-core/
├── src/
│   ├── state/           # ByteSil, SilState
│   │   ├── mod.rs
│   │   ├── byte_sil.rs  # Unidade fundamental (ρ, θ)
│   │   └── sil_state.rs # Estado completo (16 camadas)
│   │
│   ├── transforms/      # Transformações por fase
│   │   ├── mod.rs
│   │   ├── perception.rs   # L(0-4)
│   │   ├── processing.rs   # L(5-7)
│   │   ├── interaction.rs  # L(8-A)
│   │   ├── emergence.rs    # L(B-C)
│   │   └── meta.rs         # L(D-F)
│   │
│   ├── patterns/        # Design patterns SIL
│   │   ├── mod.rs
│   │   ├── observer.rs     # Padrão Observer (percepção)
│   │   ├── strategy.rs     # Padrão Strategy (processamento)
│   │   ├── mediator.rs     # Padrão Mediator (interação)
│   │   └── emergent.rs     # Padrão Emergent (auto-organização)
│   │
│   ├── processors/      # Backends de hardware
│   │   ├── mod.rs
│   │   ├── traits.rs       # Traits comuns
│   │   ├── cpu/            # CPU backend (SIMD)
│   │   ├── gpu/            # GPU backend (wgpu)
│   │   └── npu/            # NPU backend (CoreML)
│   │
│   ├── vsp/             # Virtual Sil Processor
│   │   ├── mod.rs          # VM principal
│   │   ├── opcode.rs       # 70+ opcodes
│   │   ├── instruction.rs  # Decode + Builder
│   │   ├── state.rs        # Registradores, flags
│   │   ├── memory.rs       # Segmentos de memória
│   │   ├── backend.rs      # Abstração CPU/GPU/NPU
│   │   ├── bytecode.rs     # Formato .silc
│   │   └── error.rs        # Tipos de erro
│   │
│   ├── cycle/           # Loop fechado
│   │   ├── mod.rs
│   │   └── loop.rs      # sil_loop principal
│   │
│   ├── lib.rs           # Raiz do crate
│   └── prelude.rs       # Re-exportações convenientes
│
├── Cargo.toml
└── README.md
```

## Instalação e Requisitos

- Requer Rust `1.92+` e `cargo` instalado
- Suporte cruzado: macOS, Linux e Windows (CPU/Interpreter)
- JIT DynASM: apenas `ARM64` (Apple Silicon/macOS, Linux)

Instalação local (desenvolvimento):

```bash
git clone https://github.com/silvanoneto/
cd sil-core
cargo build
```

## Executar Exemplos

Alguns exemplos úteis disponíveis no diretório `examples/`:

```bash
# Interpreter universal (CPU)
cargo run --example vsp_interpreter

# DynASM JIT (ARM64, requer feature)
cargo run --example vsp_dynasm --features dynasm

# Pipeline JSIL (I/O)
cargo run --example jsil_pipeline

# GPU batching (quando habilitar feature gpu)
cargo run --example gpu_batching --features gpu
```

Benchmarks (opcionais):

```bash
# CPU
cargo bench --bench cpu

# GPU (requer --features gpu)
cargo bench --bench gpu --features gpu

# DynASM (ARM64, requer --features dynasm)
cargo bench --bench dynasm_comparison --features dynasm

# Cranelift JIT (requer --features jit)
cargo bench --bench jit_comparison --features jit
```

## Quick Start

```rust
use sil_core::prelude::*;

// Criar estado inicial
let state = SilState::neutral();

// Criar pipeline de transformações
let pipeline = Pipeline::new(vec![
    Box::new(PhaseShift(4)),
    Box::new(MagnitudeScale(2)),
]);

// Executar ciclo SIL
let final_state = sil_loop(state, &pipeline, 100);
```

Build e execução mínima:

```bash
cargo build
cargo run --example vsp_interpreter
```

## VSP: Virtual Sil Processor

> *"A JVM só que realmente aberta."*

**VSP** é a máquina virtual que torna a compatibilidade transparente — tanto a nível de hardware quanto de software.

### Arquitetura

```
┌─────────────────────────────────────────────────────────────────┐
│                         VSP (Máquina Virtual)                   │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │  Bytecode   │  │   Estado    │  │        Memória          │  │
│  │   (.silc)   │  │  (R0-R15)   │  │  Code│Stack│Heap│I/O    │  │
│  └──────┬──────┘  └──────┬──────┘  └───────────┬─────────────┘  │
│         │                │                     │                │
│         ▼                ▼                     ▼                │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │               Execution Engine (fetch-decode-execute)    │   │
│  └──────────────────────────────────────────────────────────┘   │
│                              │                                  │
│         ┌────────────────────┼────────────────────┐             │
│         ▼                    ▼                    ▼             │
│  ┌─────────────┐      ┌─────────────┐      ┌─────────────┐      │
│  │ CPU Backend │      │ GPU Backend │      │ NPU Backend │      │
│  │  (default)  │      │   (wgpu)    │      │ (Core ML)   │      │
│  └─────────────┘      └─────────────┘      └─────────────┘      │
└─────────────────────────────────────────────────────────────────┘
```

### ISA (70+ opcodes)

| Categoria | Opcodes | Descrição |
|:----------|:--------|:----------|
| Control | `NOP`, `HALT`, `JMP`, `CALL`, `RET` | Fluxo de execução |
| Data | `LOAD`, `STORE`, `MOVE`, `PUSH`, `POP` | Manipulação de dados |
| Arithmetic | `ADD`, `SUB`, `MUL`, `DIV`, `XOR` | Operações em ByteSil |
| Layer | `LGET`, `LSET`, `LXOR`, `LSHIFT` | Acesso às 16 camadas |
| Transform | `PHASE`, `MAG`, `LERP`, `COLLAPSE` | Transformações SIL |
| Compat | `PROMOTE`, `DEMOTE`, `SETMODE` | Modos de compatibilidade |
| System | `SYSCALL`, `IO`, `SYNC` | Interface com sistema |
| Hints | `PREFETCH`, `HINT_GPU`, `HINT_NPU` | Otimização de backend |

### Modos de Compatibilidade

```
SIL-8   ─► 1 camada   (8 bits)   ─► IoT mínimo
SIL-16  ─► 2 camadas  (16 bits)  ─► Microcontroladores
SIL-32  ─► 4 camadas  (32 bits)  ─► Embedded
SIL-64  ─► 8 camadas  (64 bits)  ─► Desktop
SIL-128 ─► 16 camadas (128 bits) ─► Full SIL
```

### Execution Engines

VSP oferece **dois backends de execução** com diferentes trade-offs:

#### 1. Native Rust Interpreter (Universal)

**Threaded dispatch** com function pointers e inline optimization:

```rust
use sil_core::vsp::interpreter::VspInterpreter;

let mut interp = VspInterpreter::new();
interp.compile(&program)?;
interp.execute(&mut state)?;
```

✅ **Vantagens:**

- Funciona em **qualquer arquitetura** (x86-64, ARM64, RISC-V, WASM)
- **257M ops/sec** em Apple Silicon M3 Pro
- Zero dependências externas
- Hot-swapping de backends (CPU/GPU/NPU)

#### 2. DynASM JIT (ARM64 only)

**Native assembly generation** com zero overhead:

```rust
#[cfg(target_arch = "aarch64")]
use sil_core::vsp::dynasm::VspDynasmJit;

let mut jit = VspDynasmJit::new()?;
jit.compile(&program)?;
jit.execute(&mut state)?;
```

✅ **Vantagens:**

- **326M ops/sec** em Apple Silicon (27% mais rápido)
- Zero overhead de dispatch
- Otimizações específicas ARM64

⚠️ **Limitações:**

- Apenas ARM64 (macOS/Linux)
- Feature `dynasm` implementada; habilite com `--features dynasm`
- Ganho máximo em programas simples (NOPs, loads); mixed workloads mostram speedup de ~5–15×

Build rápido (Apple Silicon):

```bash
cargo run --example vsp_dynasm --features dynasm
```

#### Performance Comparison (M3 Pro)

| Engine | Warm Latency/exec | Throughput (10k execs) | Portability |
|:-------|:--:|:--:|:------------|
| **Interpreter** | ~82.5 ns | ~24.5M execs/sec | ✅ x86/ARM/RISC-V/WASM |
| **DynASM JIT** | ~4.2 ns | ~357M execs/sec | ⚠️ ARM64 only |
| **Speedup** | **~19.6×** | **~14.6×** | - |

**Benchmark methodology:** NOP-heavy program (64 bytes), warm cache, single iteration + loop of 10,000 executions. Speedup varies with instruction mix; pure NOPs favor JIT (less dispatch overhead), while mixed workloads show ~5–15× speedup.

#### Choosing an Engine

```rust
// Opção 1: Interpreter (recomendado)
let mut engine = VspInterpreter::new();

// Opção 2: Auto-select (JIT se disponível, senão interpreter)
#[cfg(all(target_arch = "aarch64", feature = "dynasm"))]
let mut engine = VspDynasmJit::new().unwrap_or_else(|_| {
    VspInterpreter::new()
});

#[cfg(not(all(target_arch = "aarch64", feature = "dynasm")))]
let mut engine = VspInterpreter::new();
```

**Recomendação:**

- **Interpreter DEFAULT**: Oferece portabilidade universal (~24M execs/sec); use para qualquer arquitetura
- **DynASM opcional**: Use apenas no ARM64 se precisar de máxima performance (~357M execs/sec, ~15× speedup em NOPs)

### Uso (Legacy API)

```rust
use sil_core::vsp::{Vsp, VspConfig};

// Carregar bytecode
let bytecode = include_bytes!("programa.silc");

// Configurar VM
let config = VspConfig::default()
    .with_memory_size(1024 * 1024)
    .with_mode(SilMode::Sil128);

// Criar e executar
let mut vm = Vsp::new(config);
vm.load(bytecode)?;
let result = vm.run()?;
```

### Bytecode (.silc)

```
┌────────────────────────────────────────┐
│ Header (32 bytes)                      │
│  ├─ Magic: "SILC"                      │
│  ├─ Version: 1.0                       │
│  ├─ Mode: SIL-128                      │
│  └─ Segment offsets                    │
├────────────────────────────────────────┤
│ Code Segment                           │
├────────────────────────────────────────┤
│ Data Segment                           │
├────────────────────────────────────────┤
│ Symbol Table (opcional)                │
├────────────────────────────────────────┤
│ Debug Info (opcional)                  │
└────────────────────────────────────────┘
```

## Princípios SOLID × SIL

| SOLID | Princípio SIL | Implementação |
|:------|:--------------|:--------------|
| **S** | Camadas ortogonais | Uma camada = uma semântica |
| **O** | Estado imutável | Novas transforms sem modificar |
| **L** | Transformação pura | Qualquer impl substitui outra |
| **I** | Traits por fase | Sensor ≠ Processor ≠ Mediator |
| **D** | Estado abstrato | Depende de traits, não structs |

## Performance: CPU vs GPU vs NPU

> **Regra:** GPU compensa apenas para **batches ≥ 10.000 estados** ou operações muito intensivas.

### Benchmarks (Apple Silicon M3)

#### Operações Básicas (CPU)

| Operação | Tempo | Throughput |
|:---------|------:|:-----------|
| `ByteSil::mul` | 586 ps | 1.7B ops/s |
| `ByteSil::xor` | 601 ps | 1.6B ops/s |
| `SilState::tensor` | 21 ns | 47M ops/s |
| `SilState::xor` | 10 ns | 100M ops/s |
| `SilState::collapse` | 1.7 ns | 588M ops/s |

#### Gradientes (CPU)

| Operação | Tempo |
|:---------|------:|
| `Gradient::compute_cpu` (1 estado) | 77 ns |
| `Gradient::compute_cpu` (1000 estados) | 73 µs |
| `Gradient::apply_to` (descent step) | 26 ns |
| `gradient_descent` (10 iterações) | 952 ns |
| `gradient_descent` (100 iterações) | 9.5 µs |

#### Interpolação (CPU)

| Operação | Tempo |
|:---------|------:|
| `lerp_states` | 12 ns |
| `slerp_states` | 16 ns |
| `bezier_quadratic` | 38 ns |
| `bezier_cubic` | 78 ns |

#### Quantização: Quantizable vs NPU

| Operação | Quantizable | NPU Tensor | Overhead |
|:---------|------------:|-----------:|---------:|
| `to_int8` / `INT8` | 27 ns | 49 ns | +82% |
| `to_fp16` / `FP16` | 27 ns | 83 ns | +207% |

> **Nota:** O overhead NPU é esperado — `NpuTensor` prepara dados para inferência em ANE/Core ML.

#### Scaling de Interpolação (CPU)

| Steps | Tempo | Latência/step |
|:------|------:|--------------:|
| 10 | 141 ns | 14.1 ns |
| 50 | 629 ns | 12.6 ns |
| 100 | 1.24 µs | 12.4 ns |
| 500 | 6.14 µs | 12.3 ns |
| 1000 | 12.2 µs | 12.2 ns |

#### Inferência NPU (Core ML / ANE)

| Operação | Tempo |
|:---------|------:|
| `NpuContext::infer` (classifier, 10 classes) | 430 ns |

#### Detecção de Processadores

| Operação | Tempo |
|:---------|------:|
| `ProcessorType::available()` | 22 ns |
| `ProcessorType::Cpu::is_available()` | 296 ps |
| `ProcessorType::Gpu::is_available()` | 296 ps |
| `ProcessorType::Npu::is_available()` | 296 ps |

#### Batch Processing (CPU)

| Batch Size | Tempo | Latência/estado |
|:-----------|------:|----------------:|
| 100 estados (gradient) | 7.32 µs | 73.2 ns |

#### GPU Context

| Operação | Tempo |
|:---------|------:|
| `GpuContext::new_sync` | **583 µs** |

#### VSP Execution Engines

| Engine | Latency per exec | Peak Throughput | Overhead vs Native Rust |
|:-------|:--:|:--:|:--:|
| **Interpreter (Rust)** | ~82.5 ns | ~24.5M execs/s | ~46x |
| **DynASM JIT (ARM64)** | ~4.2 ns | ~357M execs/s | ~2.3x |
| **Native Rust loop** | ~1.8 ns | ~1.6B ops/s | 1x |

> **Nota:** O overhead VSP é esperado — trata-se de uma máquina virtual completa com fetch-decode-execute, registradores, memória segmentada e abstração de backend. O custo compensa quando:

> - Portabilidade de bytecode é necessária (Interpreter ubíquo; DynASM ARM64-only)
> - Hot-swapping de backends (CPU→GPU→NPU)
> - Debugging avançado (DAP/LSP)
> - Compatibilidade entre SIL-8 até SIL-128
>
> **Overhead reduzido com JIT:** DynASM JIT elimina dispatch indireto, trazendo latência próxima ao nativo (4.2 ns vs 1.8 ns em puro loop).

### Regra de Decisão

```
┌─────────────────────────────────────────────────────────────────┐
│                    QUANDO USAR.   ?                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   Batch Size < 10.000 ────────────► CPU  (overhead GPU > ganho) │
│                                                                 │
│   Batch Size ≥ 10.000 ────────────► GPU  (paralelismo compensa) │
│                                                                 │
│   Operação única ─────────────────► CPU  (sempre)               │
│                                                                 │
│   sil_loop (100 ciclos) ──────────► CPU  (1.8 µs, muito rápido) │
│                                                                 │
│   Treinamento/Otimização ─────────► GPU  (batches grandes)      │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### API de Seleção Automática

```rust
use sil_core::gpu::{GpuContext, SilGradient};

// CPU: operações individuais
let grad = SilGradient::compute_cpu(&state, 0.01);

// GPU: batches grandes (quando implementado)
// let ctx = GpuContext::new_sync()?;
// let grads = ctx.compute_gradients_batch(&states).await;
```

## Features

```toml
[dependencies]
sil-core = { version = "2026.1", features = ["gpu", "npu"] }
```

| Feature | Descrição |
|:--------|:----------|
| `default` | Apenas CPU, sem dependências extras |
| `gpu` | Habilita wgpu para compute shaders |
| `npu` | Habilita Core ML / ANE para inferência |
| `simd` | Otimizações SIMD (requer nightly) |
| `python` | Bindings PyO3 para Python |
| `jit` | Habilita JIT/AOT via Cranelift (bin `vsp-aot`) |
| `dynasm` | Habilita JIT nativo ARM64 via DynASM |

Uso com `cargo` (local):

```bash
# GPU
cargo run --example gpu_pipeline_pool --features gpu

# JIT Cranelift (AOT/JIT)
cargo run --bin vsp-aot --features jit

# DynASM ARM64
cargo run --example vsp_dynasm --features dynasm
```

## Roadmap

- [x] ByteSil (unidade fundamental)
- [x] SilState (16 camadas)
- [x] Transformações por fase
- [x] Processadores CPU/GPU/NPU
- [x] VSP (Virtual Sil Processor)
- [x] Assembler (`silasm`: .sil → .silc)
- [x] REPL interativo
- [x] Debugger visual (DAP)
- [x] Distributed sync (entanglement)
- [x] Language Server Protocol (LSP)
- [x] VS Code Extension

## IDE Support

### Language Server Protocol (LSP)

O servidor LSP completo para arquivos `.sil`:

```rust
use sil_core::vsp::lsp::{SilLanguageServer, LspConfig};

let server = SilLanguageServer::new(LspConfig::default());
server.run_stdio();
```

**Funcionalidades:**

- 🎯 **IntelliSense** — Auto-complete para opcodes, registradores, diretivas
- 📖 **Hover Info** — Documentação inline de opcodes e registradores
- 🔍 **Go to Definition** — Navegação para labels
- 📋 **Document Symbols** — Outline de código
- ⚠️ **Diagnostics** — Erros e warnings em tempo real
- 🎨 **Semantic Tokens** — Syntax highlighting avançado
- ✨ **Formatting** — Formatação automática de código

### VS Code Extension

Extensão completa em `sil-vscode/`:

```bash
cd sil-vscode
npm install
npm run compile
# F5 para Extension Development Host
```

**Inclui:**

- 🌈 Syntax Highlighting (TextMate grammar)
- 📝 Snippets para templates comuns
- 🐛 Debug Adapter Protocol (DAP)
- ⚡ Comandos: Assemble, Run, Debug, REPL
- ⚙️ Configurações: mode, format, diagnostics

## Licença

AGPL-3.0
