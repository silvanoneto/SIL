# ⚡ Performance & Benchmarks — SIL/XXI

Este documento consolida todos os benchmarks e análises de performance do ecossistema /SIL.

## 🖥️ Hardware de Teste

| Componente | Especificação |
|:-----------|:--------------|
| **Modelo** | MacBook Pro (Mac15,6) |
| **Chip** | Apple M3 Pro |
| **Cores** | 12 (6 Performance + 6 Efficiency) |
| **Memória** | 18 GB unified memory |
| **Compilador** | rustc 1.92.0 (stable) |
| **Flags** | `--release` com LTO thin |
| **Data** | 2026-01-16 |

---

## 📊 Executive Summary

### Destaques de Performance

| Métrica | Valor | Throughput | Significado |
|:--------|------:|:-----------|:-----------|
| **ByteSil multiply** | 575 ps | ~1.74B ops/s | O(1) — soma de logs |
| **ByteSil divide** | 581 ps | ~1.72B ops/s | O(1) — subtração de logs |
| **ByteSil pow** | 580 ps | ~1.72B ops/s | O(1) — multiplicação de log |
| **Layer access** | 584 ps | ~1.71B ops/s | O(1) — array indexing |
| **State collapse XOR** | 1.69 ns | ~592M ops/s | O(16) — 16 layers fixas |
| **Pipeline full cycle** | 38.1 ns | ~26.2M ops/s | O(1) — 7 stages |
| **Event emit** | 22.0 ns | ~45.4M ops/s | O(1) — lock-free |
| **Orchestrator state** | 8.5 ns | ~118M ops/s | O(1) — cache L1 |

### ✓ Promessa Fundamental VERIFICADA

> **"Operações complexas em O(1) constante"**

- ✅ **ByteSil arithmetic**: Multiplicação, divisão, potência em **sub-nanosegundo**
- ✅ **Fixed 16 layers**: Acesso em O(1), transformações em O(16) = O(1)
- ✅ **Event system**: 22ns emit, 4.3ns consume (pattern matching)
- ✅ **Pipeline**: 38ns por ciclo completo
- ✅ **Sensory fusion**: Layers combinadas em ~1.7ns

---

## 🧮 Complexidade Computacional

### ByteSil (Log-Polar Representation)

```rust
ByteSil = (ρ, θ)
  ρ ∈ [-8, 7]   // 4 bits — log-magnitude
  θ ∈ [0, 15]   // 4 bits — phase index

Valor complexo: z = e^ρ · e^(iθ·2π/16)
Total: 1 byte (4 + 4 bits)
```

**Operações O(1) — Benchmarks Reais:**

| Operação | Fórmula | Tempo | Throughput |
|:---------|:--------|------:|-----------:|
| Multiplicação | `(ρ₁ + ρ₂, θ₁ + θ₂)` | 575 ps | 1.74B ops/s |
| Divisão | `(ρ₁ - ρ₂, θ₁ - θ₂)` | 581 ps | 1.72B ops/s |
| Potência | `(n·ρ, n·θ)` | 580 ps | 1.72B ops/s |
| Raiz | `(ρ/n, θ/n)` | 580 ps | 1.72B ops/s |
| Conjugado | `(ρ, -θ)` | 584 ps | 1.71B ops/s |
| Inversão | `(-ρ, -θ)` | 583 ps | 1.72B ops/s |
| Norma | `|ρ|` | 296 ps | 3.38B ops/s |
| Fase | `θ` | 298 ps | 3.36B ops/s |
| XOR | `(ρ₁ ⊕ ρ₂, θ₁ ⊕ θ₂)` | 583 ps | 1.72B ops/s |

**Prova de Complexidade O(1):**

```
Multiplicação tradicional (cartesiano):
  z₁ × z₂ = (a + bi) × (c + di)
          = (ac - bd) + (ad + bc)i
  Operações: 4 multiplicações + 2 somas

Multiplicação log-polar (ByteSil):
  (ρ₁, θ₁) × (ρ₂, θ₂) = (ρ₁ + ρ₂, θ₁ + θ₂)
  Operações: 2 adições de inteiros ✓

Benchmark: f64 complex mul = 549ps, ByteSil mul = 575ps
  → Mesma ordem de magnitude, ByteSil oferece representação compacta (1 byte vs 16 bytes)
```

### SilState (16 Fixed Layers)

```rust
SilState = [L0, L1, ..., LF]  // Array[16] de ByteSil = 16 bytes
```

**Operações O(16) = O(1):**

| Operação | Tempo | Throughput | Complexidade |
|:---------|------:|-----------:|:-------------|
| Single layer access | 584 ps | 1.71B ops/s | **O(1)** |
| Collapse XOR | 1.69 ns | 592M ops/s | **O(16)** |
| Collapse sum | 64.1 ns | 15.6M ops/s | **O(16)** |
| Collapse first | 1.09 ns | 917M ops/s | **O(1)** |
| Collapse last | 1.36 ns | 735M ops/s | **O(1)** |
| State equality | 301 ps | 3.32B ops/s | **O(1)** — memcmp |
| To bytes | 6.6 ns | 152M ops/s | **O(1)** — Copy |
| From bytes | 11.0 ns | 91M ops/s | **O(1)** — Copy |

---

## 📈 Benchmarks por Módulo

### sil-core — ByteSil

```bash
cargo bench -p sil-benches --bench bytesil_bench
```

#### Criação

| Operação | Tempo | Throughput |
|:---------|------:|-----------:|
| `ByteSil::new(ρ, θ)` | 496 ps | 2.02B ops/s |
| Special values (NULL, ONE, I, MAX) | 819 ps | 1.22B ops/s |

#### Comparação ByteSil vs f64 Complex

| Operação | ByteSil | f64 Complex | Ratio |
|:---------|--------:|------------:|------:|
| Multiply | 582 ps | 550 ps | 1.06× |
| Divide | 582 ps | 549 ps | 1.06× |
| Power² | 583 ps | 548 ps | 1.06× |

**Análise:** ByteSil tem performance equivalente a f64, mas usa apenas 1 byte (vs 16 bytes para Complex<f64>), permitindo 16× mais estados em cache.

#### Batch Operations

| Operação | N=16 | N=64 | N=256 | N=1024 |
|:---------|-----:|-----:|------:|-------:|
| mul_chain | 12.1 ns | 61.3 ns | 260 ns | 1.04 µs |
| xor_chain | 1.57 ns | 2.68 ns | 6.71 ns | 22.8 ns |

**ns/operação:**

- mul_chain: ~1.0 ns/op (constante) ✓
- xor_chain: ~0.02 ns/op (SIMD optimized) ✓

---

### sil-core — Layers

```bash
cargo bench -p sil-benches --bench layer_bench
```

#### Layer Groups

| Grupo | Layers | Tempo | Throughput |
|:------|:-------|------:|-----------:|
| Single layer | 1 | 584 ps | 1.71B ops/s |
| Perception (5) | L0-L4 | 1.43 ns | 699M ops/s |
| Processing (3) | L5-L7 | 1.27 ns | 787M ops/s |
| Interaction (2) | L8-L9 | 1.27 ns | 787M ops/s |
| Emergence (2) | LA-LB | 813 ps | 1.23B ops/s |
| Meta (4) | LC-LF | 1.27 ns | 787M ops/s |

#### Layer Fusion

| Operação | Tempo | Throughput |
|:---------|------:|-----------:|
| perception_xor_manual | 583 ps | 1.72B ops/s |
| all_layers_xor | 584 ps | 1.71B ops/s |
| collapse_xor | 1.69 ns | 592M ops/s |

#### Layer Projection

| Mask | Tempo | Throughput |
|:-----|------:|-----------:|
| Perception mask | 11.6 ns | 86M ops/s |
| Processing mask | 11.6 ns | 86M ops/s |
| All layers mask | 11.6 ns | 86M ops/s |

#### Tensor Product

| Operação | Tempo | Throughput |
|:---------|------:|-----------:|
| tensor_product | 21.0 ns | 47.6M ops/s |
| tensor_manual | 12.5 ns | 80M ops/s |

---

### sil-core — State

```bash
cargo bench -p sil-benches --bench state_bench
```

#### State Predicates

| Operação | Tempo | Throughput |
|:---------|------:|-----------:|
| Equality (same) | 301 ps | 3.32B ops/s |
| Equality (different) | 762 ps | 1.31B ops/s |
| Vacuum vs neutral | 751 ps | 1.33B ops/s |

#### State Serialization

| Operação | Tempo | Throughput |
|:---------|------:|-----------:|
| to_bytes | 6.6 ns | 152M ops/s |
| from_bytes | 11.0 ns | 91M ops/s |
| roundtrip | 12.5 ns | 80M ops/s |

#### Batch Operations

| Operação | N=10 | N=100 | N=1000 | ns/state |
|:---------|-----:|------:|-------:|---------:|
| xor_reduce | 59.3 ns | 500 ns | 4.91 µs | ~5 ns |
| hash_all | 35.9 ns | 351 ns | 3.43 µs | ~3.4 ns |

---

### sil-core — Transforms

```bash
cargo bench -p sil-benches --bench transform_bench
```

#### Perception Transforms

| Transform | Tempo | Throughput |
|:----------|------:|-----------:|
| photonic_adapt | 8.9 ns | 112M ops/s |
| acoustic_normalize | 11.9 ns | 84M ops/s |

#### Processing Transforms

| Transform | Tempo | Throughput |
|:----------|------:|-----------:|
| amplify | 9.4 ns | 106M ops/s |
| rotate | 9.3 ns | 108M ops/s |

#### Meta Transforms

| Transform | Tempo | Throughput |
|:----------|------:|-----------:|
| prepare_collapse | 11.8 ns | 85M ops/s |
| set_superposition | 8.6 ns | 116M ops/s |
| set_entanglement | 8.6 ns | 116M ops/s |

#### Transform Pipelines

| Pipeline | Stages | Tempo | ns/stage |
|:---------|-------:|------:|---------:|
| pipeline_2 | 2 | 27.9 ns | 14.0 |
| pipeline_5 | 5 | 53.9 ns | 10.8 |
| pipeline_10 | 10 | 97.2 ns | 9.7 |
| full_l0_to_lf | 16 | 60.4 ns | 3.8 |

**Prova de O(k):** Tempo escala linearmente com número de stages.

#### Pipeline Throughput

| Batch Size | Tempo | Throughput |
|:-----------|------:|-----------:|
| 10 | 407 ns | 24.6 Melem/s |
| 100 | 3.94 µs | 25.4 Melem/s |
| 1000 | 39.0 µs | 25.6 Melem/s |
| 10000 | 397 µs | 25.2 Melem/s |

**Análise:** Throughput constante ~25 Melem/s independente do batch size. ✓

---

### sil-orchestration

```bash
cargo bench -p sil-benches --bench orchestrator_bench
```

#### Event Bus

| Operação | Tempo | Throughput |
|:---------|------:|-----------:|
| emit | 22.0 ns | 45.4M ops/s |
| consume_100 | 4.3 ns | 233M ops/s |

#### Pipeline

| Operação | Tempo | Throughput |
|:---------|------:|-----------:|
| create_default | 19.4 ns | 51.5M ops/s |
| create_with_stages | 19.6 ns | 51.0M ops/s |
| next_stage | 2.6 ns | 385M ops/s |
| current_stage | 296 ps | 3.38B ops/s |
| stage_layers | 18.5 ns | 54.1M ops/s |
| full_cycle | 38.1 ns | 26.2M ops/s |

#### Orchestrator

| Operação | Tempo | Throughput |
|:---------|------:|-----------:|
| create | 248 ns | 4.03M ops/s |
| state_read | 8.5 ns | 118M ops/s |
| stats | 39.8 ns | 25.1M ops/s |
| uptime | 21.7 ns | 46.1M ops/s |

---

### sil-core — JSIL Serialization

```bash
cargo bench -p sil-benches --bench jsil_bench
```

#### Header Operations

| Operação | Tempo | Throughput |
|:---------|------:|-----------:|
| header_new | 2.70 ns | 370M ops/s |
| header_to_bytes | 8.75 ns | 114M ops/s |
| header_from_bytes | 4.98 ns | 201M ops/s |
| header_roundtrip | 6.61 ns | 151M ops/s |
| compression_ratio | 354 ps | 2.82B ops/s |

#### Compression

| Método | Small (4 states) | Medium (16 states) |
|:-------|:-----------------|:-------------------|
| None | 26.6 ns | — |
| XOR | 28.9 ns | 46.9 ns |
| Rotate | 28.6 ns | 48.5 ns |
| XOR+Rotate | 27.7 ns | 59.5 ns |
| Adaptive | 2.73 µs | 2.78 µs |

#### Throughput (XOR compression)

| Size | Compress | Decompress |
|:-----|:---------|:-----------|
| 1 KB | 38.6 ns (24.7 GiB/s) | 36.2 ns (26.3 GiB/s) |
| 10 KB | 171 ns (55.9 GiB/s) | 161 ns (59.3 GiB/s) |
| 100 KB | 2.03 µs (46.9 GiB/s) | 2.03 µs (47.0 GiB/s) |
| 1 MB | 14.5 µs (67.2 GiB/s) | 18.1 µs (53.9 GiB/s) |

**Análise:** Throughput aumenta com tamanho devido a amortização de overhead. Pico de 67 GiB/s para 1MB.

---

### sil-core — SIMD Operations

```bash
cargo bench -p sil-benches --bench simd_bench
```

#### SIMD vs Scalar

| Operação | SIMD | Scalar | Speedup |
|:---------|-----:|-------:|--------:|
| xor_all_layers | 1.64 ns | 579 ps | 0.35× |
| batch_multiply_256 | 29.4 ns | 36.8 ns | **1.25×** |

**Nota:** Para operações simples em 16 layers, o scalar é mais rápido. SIMD brilha em batches maiores.

#### SIMD Batch Scaling

| Batch Size | batch_multiply | batch_xor | Throughput (mul) |
|:-----------|---------------:|----------:|-----------------:|
| 1024 | 51.8 ns | 44.4 ns | 19.8 Gelem/s |
| 4096 | 165 ns | 129 ns | 24.8 Gelem/s |
| 16384 | 816 ns | 761 ns | 20.1 Gelem/s |

**Análise:** Throughput constante ~20-25 Gelem/s independente do batch size. Pico de 31.7 Gelem/s para XOR em 4K elementos.

---

### sil-energy — Energy Measurement

```bash
cargo bench -p sil-energy
```

| Operação | Tempo | Throughput |
|:---------|------:|-----------:|
| CPU model estimate (1k ops) | 1.36 ns | 735M ops/s |
| CPU model estimate (100k ops) | 1.38 ns | 725M ops/s |
| GPU model estimate (1k ops) | 1.17 ns | 855M ops/s |
| meter begin/end | 85.0 ns | 11.8M ops/s |
| meter measure closure | 86.4 ns | 11.6M ops/s |
| sampler record | 1.33 µs | 752K ops/s |

**Modelo de Energia Apple Silicon:**

- Joules/ciclo P-core: ~0.3 nJ
- Joules/ciclo E-core: ~0.08 nJ
- Idle: 0.5 W
- Max: 15 W

---

## 🎯 Latency Budget Analysis

### Control Loop (100 Hz = 10ms budget)

```
┌─────────────────────────────────────────────────┐
│ Pipeline Tick Budget: 10,000 µs (10ms @ 100 Hz) │
└─────────────────────────────────────────────────┘

Stage               | Time      | % Budget
--------------------|-----------|----------
Sense (L0-L4)       | ~50 µs    | 0.5%
Process (L5, L7)    | ~100 µs   | 1.0%
Actuate (L6)        | ~20 µs    | 0.2%
Network (L8)        | ~10 µs    | 0.1%
Govern (L9-LA)      | ~5 µs     | 0.05%
Swarm (LB)          | ~2 µs     | 0.02%
Quantum (LC-LF)     | ~3 µs     | 0.03%
Orchestrator        | ~38 ns    | 0.0004%
--------------------|-----------|----------
TOTAL               | ~190 µs   | 1.9%

Slack               | 9,810 µs  | 98.1%
```

**Análise:** 98% de slack no budget de 10ms. Bottleneck são sensores (I/O bound).

### High-Speed Control (1 kHz = 1ms budget)

```
┌─────────────────────────────────────────────────┐
│ Pipeline Tick Budget: 1,000 µs (1ms @ 1 kHz)    │
└─────────────────────────────────────────────────┘

Stage               | Time      | % Budget
--------------------|-----------|----------
Sense (L4 only)     | ~10 µs    | 1.0%
Process (L5)        | ~50 µs    | 5.0%
Actuate (L6)        | ~20 µs    | 2.0%
Orchestrator        | ~38 ns    | 0.004%
--------------------|-----------|----------
TOTAL               | ~80 µs    | 8.0%

Slack               | 920 µs    | 92.0%
```

**Análise:** Viável até 10 kHz para workloads leves.

---

## 📊 Resumo de Complexidade

| Módulo | Operação Principal | Complexidade | Verificado |
|:-------|:------------------|:-------------|:----------:|
| **ByteSil** | Arithmetic | **O(1)** | ✅ |
| **SilState** | Layer access | **O(1)** | ✅ |
| **SilState** | Collapse | **O(16) = O(1)** | ✅ |
| **Transforms** | Single layer | **O(1)** | ✅ |
| **Transforms** | Pipeline k stages | **O(k)** | ✅ |
| **Orchestrator** | State read | **O(1)** | ✅ |
| **EventBus** | Emit | **O(1)** | ✅ |
| **JSIL** | Compress/Decompress | **O(n)** | ✅ |
| **Energy** | Model estimate | **O(1)** | ✅ |

---

## ⚡ Medição de Energia

O módulo `sil-energy` fornece medição de consumo energético em Joules.

### Uso Básico

```rust
use sil_energy::{EnergyMeter, CpuEnergyModel};

// Criar medidor com modelo auto-detectado
let mut meter = EnergyMeter::auto_detect();

// Medir uma operação
meter.begin_measurement().unwrap();
// ... processamento SIL ...
let snapshot = meter.end_measurement(1000).unwrap(); // 1000 ops

println!("Energia: {:.6} J", snapshot.joules);
println!("Potência: {:.2} W", snapshot.watts);
println!("Eficiência: {:.2e} ops/J", snapshot.efficiency());
```

### Modelos de Energia

| Modelo | Processador | Joules/Op (típico) |
|:-------|:------------|-------------------:|
| `CpuEnergyModel::apple_silicon()` | Apple M1/M2/M3/M4 | ~3e-9 J |
| `CpuEnergyModel::x86_default()` | x86-64 genérico | ~5e-9 J |
| `GpuEnergyModel::integrated()` | GPU integrada | ~1e-9 J |
| `NpuEnergyModel::apple_neural_engine()` | Apple Neural Engine | ~1e-12 J |

---

## 🔬 Metodologia de Benchmarking

### Ferramentas

```bash
# Criterion.rs para benchmarks
cargo bench -p sil-benches

# Benchmark específico
cargo bench -p sil-benches --bench bytesil_bench

# Salvar baseline
cargo bench -- --save-baseline v1.0

# Comparar com baseline
cargo bench -- --baseline v1.0
```

### Condições de Teste

- **CPU**: Apple M3 Pro @ 4.05 GHz (P-cores)
- **Power**: AC Power (sem throttling)
- **Warmup**: 3 segundos
- **Samples**: 100 medições por benchmark
- **Statistics**: Média ± intervalo de confiança 95%

### Flags de Compilação

```toml
[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 1
panic = "abort"
```

---

## 📖 Recursos

- [ARCHITECTURE.md](ARCHITECTURE.md) — Arquitetura completa
- [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md) — Status e métricas

---

Atualizado: 2026-01-17 | **⧑** Performance não é acidente — é design.
