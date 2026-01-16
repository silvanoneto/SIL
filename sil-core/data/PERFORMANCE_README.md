# 🎯 Performance Fixes - README

Este diretório contém a investigação e correções para problemas críticos de performance identificados nos benchmarks de 11/01/2026.

---

## 📁 Documentos

### 1. [PERFORMANCE_SUMMARY.md](PERFORMANCE_SUMMARY.md) - **COMECE AQUI**
Resumo executivo com:
- 4 problemas identificados
- Fixes implementados
- Impacto esperado
- Próximos passos

### 2. [PERFORMANCE_INVESTIGATION.md](PERFORMANCE_INVESTIGATION.md) - Análise Técnica
Investigação detalhada incluindo:
- Causa raiz de cada problema
- Código problemático com links
- Soluções imediatas e de longo prazo
- Action items priorizados

### 3. [VSP_JIT_PROPOSAL.md](VSP_JIT_PROPOSAL.md) - Design JIT
Proposta completa de JIT compilation:
- Arquitetura com Cranelift
- Roadmap de 4 sprints
- Targets de performance
- Código de exemplo

---

## ✅ Fixes Implementados

### Hot Fixes (P0 - Críticos)

#### 1. Cache de `is_available()` - Elimina regressão +1,551,665%
**Arquivo:** `src/processors/gpu/mod.rs`

```rust
// Antes: 4.67µs TODA CHAMADA
pub fn is_available() -> bool {
    let instance = Instance::new(...);  // ❌ Criava nova instância
    // ...
}

// Depois: <1ns (após primeira chamada)
static GPU_AVAILABLE: OnceLock<bool> = OnceLock::new();
pub fn is_available() -> bool {
    *GPU_AVAILABLE.get_or_init(|| { /* detecção real */ })  // ✅ Cache
}
```

#### 2. Cache de `available()` - Elimina regressão +21,310%
**Arquivo:** `src/processors/mod.rs`

```rust
// Antes: 4.80µs TODA CHAMADA
pub fn available() -> Vec<Self> {
    [Self::Gpu, Self::Npu, Self::Cpu]
        .filter(|p| p.is_available())  // ❌ Chamava is_available() 3x
        .collect()
}

// Depois: <1ns (após primeira chamada)
pub fn available_cached() -> &'static [Self] {
    static AVAILABLE: OnceLock<Vec<ProcessorType>> = OnceLock::new();
    AVAILABLE.get_or_init(|| /* detecção 1x */ )  // ✅ Cache
}
```

#### 3. Heurística de Seleção - GPU vs CPU
**Arquivo:** `src/processors/performance_fixes.rs`

```rust
pub struct ProcessorSelector;

impl ProcessorSelector {
    // Seleciona automaticamente baseado em tamanho de lote
    pub fn select_for_interpolation(batch_size: usize) -> ProcessorType {
        match batch_size {
            0..=500 => ProcessorType::Cpu,  // Overhead GPU não compensa
            _ => ProcessorType::Gpu,        // GPU eficiente
        }
    }
}
```

Breakeven points (M3 Pro):
- Interpolação: 500 elementos
- Gradiente: 200 elementos  
- Distâncias: 1000 elementos

#### 4. Singleton GPU Context
**Arquivo:** `src/processors/performance_fixes.rs`

```rust
static GPU_CONTEXT: OnceLock<GpuContext> = OnceLock::new();

pub fn get_gpu_context() -> GpuResult<&'static GpuContext> {
    // Primeira chamada: ~701µs (inicialização)
    // Chamadas subsequentes: <1ns (referência estática)
}
```

---

## 📊 Resultados Esperados

| Operação | Antes | Depois | Melhoria |
|----------|-------|--------|----------|
| `Gpu::is_available()` (2ª+ chamada) | 4.67µs | <1ns | **~4,670,000x** |
| `ProcessorType::available()` (2ª+) | 4.80µs | <1ns | **~4,800,000x** |
| GPU lerp (batch=50) | 23.50ns | 12.29ns | **1.9x** (usa CPU) |
| GPU slerp (batch=100) | 26.56ns | 15.72ns | **1.7x** (usa CPU) |

---

## 🚀 Como Usar

### API Original (mantida para compatibilidade)
```rust
use sil_core::processors::{ProcessorType, GpuContext};

// Funciona, mas lento na primeira chamada
if ProcessorType::Gpu.is_available() {
    let ctx = GpuContext::new_sync()?;
    // ...
}
```

### Nova API (recomendada)
```rust
use sil_core::processors::performance_fixes::{
    available_processors_cached,
    get_gpu_context,
    ProcessorSelector,
};

// RÁPIDO: <1ns após primeira chamada
let processors = available_processors_cached();

// RÁPIDO: Singleton context
let gpu = get_gpu_context()?;

// INTELIGENTE: Seleção automática
let processor = ProcessorSelector::select_for_interpolation(batch.len());
match processor {
    ProcessorType::Cpu => cpu_lerp(&batch),
    ProcessorType::Gpu => get_gpu_context()?.lerp_batch(&batch),
    _ => unreachable!(),
}
```

---

## 🧪 Validação

### Compilar com fixes
```bash
cd sil-core
cargo check --features "gpu,npu"
```

### Rodar benchmarks
```bash
# Benchmark completo
cargo bench --all-features

# Apenas detecção de processadores
cargo bench --features "gpu,npu" --bench processors_compare processor_detection
```

### Esperar melhorias em:
- `processor_detection/ProcessorType::available`: ~4.8µs → <1ns
- `processor_detection/ProcessorType::Gpu::is_available`: ~4.7µs → <1ns
- `compare_interpolation_lerp` (batch pequeno): GPU não usada, CPU automática

---

## 📈 Roadmap

### ✅ Concluído (P0 - Crítico)
- ✅ Cache de `is_available()`
- ✅ Cache de `available()`
- ✅ Heurística de seleção
- ✅ Documentação completa

### 🔄 Em Progresso (P1 - Alto)
- [ ] Validar com novo benchmark run
- [ ] Integrar `ProcessorSelector` em hot paths
- [ ] VSP JIT PoC com Cranelift

### ⏳ Planejado (P2-P3)
- [ ] Pre-compiled shaders
- [ ] Async GPU ops
- [ ] AOT VSP compiler
- [ ] Testes de regressão

---

## 🐛 Problemas Conhecidos

1. **VSP ainda lento (~46,400x)**: JIT em roadmap (P1)
2. **GPU context overhead (~700µs)**: Mitigado com singleton, mas ainda alto na primeira chamada
3. **Warnings de dead_code**: Campos GPU/NPU usados mas não marcados

---

## 📞 Suporte

**Issues:** https://github.com/silvanoneto//issues  
**Docs:** https://docs.sil-core.dev/performance  
**Email:** performance@sil-core.dev

---

## 📚 Leitura Adicional

- [Benchmarks Report](BENCHMARK_REPORT.md) - Resultados completos
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [WGPU Best Practices](https://wgpu.rs/)
- [Cranelift JIT](https://cranelift.dev/)

---

**Status:** ✅ Fixes implementados, aguardando validação  
**Última Atualização:** 11 de Janeiro de 2026  
**Próxima Revisão:** 18 de Janeiro de 2026
