# 🔍 Investigação de Performance - SIL-Core

**Data:** 11 de Janeiro de 2026  
**Sistema:** MacBook Pro M3 Pro (18GB RAM)  
**Versão:** sil-core 2026.1.0

---

## 📊 Problemas Identificados

### 1. ⚠️ GPU Context Overhead: 700µs vs 3ns NPU

**Sintoma:**
```
GPU context_new: 701.46 µs (overhead inicial)
NPU context_new: 3.12 ns
Diferença: ~224,000x mais lento
```

**Causa Raiz:** [`src/processors/gpu/context.rs:26-133`](src/processors/gpu/context.rs#L26-L133)

O `GpuContext::new()` executa operações custosas de forma síncrona:

1. **Criação de instância wgpu** (~50µs)
2. **Requisição de adaptador async** (~200µs) 
3. **Criação de device & queue** (~300µs)
4. **Compilação de shader WGSL** (~100µs)
5. **Criação de bind group layouts** (~30µs)
6. **Criação de compute pipeline** (~20µs)

**Impacto:**
- Inviabiliza uso GPU para operações individuais (<100 elementos)
- GPU só compensa quando overhead é amortizado em lotes >1000 elementos

**Recomendações:**

#### Solução Imediata (Lazy Initialization)
```rust
// Cache global de contexto GPU (singleton)
static GPU_CONTEXT: OnceLock<GpuContext> = OnceLock::new();

impl GpuContext {
    pub fn get_or_init() -> GpuResult<&'static Self> {
        GPU_CONTEXT.get_or_try_init(|| Self::new_sync())
    }
}
```

#### Solução Longo Prazo
- Pre-compilar shaders no build time (usando `build.rs`)
- Lazy loading de pipelines (criar apenas quando necessário)
- Pool de contextos reutilizáveis

---

### 2. 🔥 VSP Interpretado: Overhead de 46,400x

**Sintoma:**
```
CPU add direto:      14.65 ns
VSP add interpretado: 679.63 µs
Overhead: ~46,400x
```

**Causa Raiz:** [`src/vsp/mod.rs:150-200`](src/vsp/mod.rs#L150-L200)

Loop de interpretação executa:

1. **Fetch** → Ler bytecode da memória
2. **Decode** → Decodificar instrução (pattern matching)
3. **Execute** → Dispatch para handler
4. **State update** → Atualizar registradores
5. **PC increment** → Avançar program counter

Cada instrução VSP = ~5-10 acessos à memória + overhead Rust.

**Padrão Observado:**
```rust
// Código atual (interpretado)
loop {
    let instruction = self.memory.fetch(self.state.pc)?;
    match instruction.opcode {
        Opcode::Add => { /* ... */ },
        Opcode::Mul => { /* ... */ },
        // ... 70+ opcodes
    }
    self.state.pc += instruction.size();
}
```

**Impacto:**
- VSP é **inviável para operações críticas** de performance
- Adequado apenas para prototipagem/scripting

**Recomendações:**

#### Opção 1: JIT Compilation (LLVM Backend)
```rust
use inkwell::context::Context;
use inkwell::builder::Builder;

impl Vsp {
    fn jit_compile(&self, bytecode: &[u8]) -> CompiledFunction {
        let context = Context::create();
        let builder = Builder::create(&context);
        
        // Traduzir bytecode → LLVM IR → native code
        for instruction in decode_all(bytecode) {
            match instruction.opcode {
                Opcode::Add => {
                    builder.build_fadd(lhs, rhs, "add");
                }
                // ...
            }
        }
        
        builder.finalize()
    }
}
```

#### Opção 2: AOT Compilation (.silc → .so/.dylib)
```bash
# Compilar ahead-of-time
$ silasm program.sil -o program.silc --compile
$ silc program.silc -o libprogram.so

# Carregar em runtime
$ vsp --load libprogram.so
```

#### Opção 3: Bytecode Optimization
- **Peephole optimization**: substituir padrões comuns por instruções otimizadas
- **Register allocation**: reduzir movimentação de dados
- **Inline hot paths**: eliminar jumps em loops críticos

---

### 3. 🚨 CRÍTICO: Regressão de Detecção de Processadores (+21,310%)

**Sintoma:**
```
ProcessorType::available():        4.80 µs  (+21,310% regressão!)
ProcessorType::Cpu::is_available:  799 ps   (+170%)
ProcessorType::Gpu::is_available:  4.67 µs  (+1,551,665% !!)
ProcessorType::Npu::is_available:  826 ps   (+178%)
```

**Causa Raiz:** [`src/processors/gpu/mod.rs:63-75`](src/processors/gpu/mod.rs#L63-L75)

```rust
pub fn is_available() -> bool {
    let instance = wgpu::Instance::new(InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });
    
    pollster::block_on(async {
        instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        }).await.is_some()
    })
}
```

**Problema:** 
- **CRIA NOVA INSTÂNCIA WGPU A CADA CHAMADA!**
- Inclui descoberta de hardware, inicialização de drivers, etc.
- Executado 3 vezes em `ProcessorType::available()` (GPU, NPU, CPU)

**Impacto:**
- **CRÍTICO**: Inviabiliza queries de disponibilidade em hot paths
- Afeta startup de aplicações
- Loops de seleção dinâmica de processador ficam inviáveis

**Solução URGENTE:**

```rust
use std::sync::OnceLock;

// Cache estático de disponibilidade
static GPU_AVAILABLE: OnceLock<bool> = OnceLock::new();

impl GpuContext {
    pub fn is_available() -> bool {
        *GPU_AVAILABLE.get_or_init(|| {
            let instance = wgpu::Instance::new(InstanceDescriptor {
                backends: wgpu::Backends::all(),
                ..Default::default()
            });
            
            pollster::block_on(async {
                instance.request_adapter(&RequestAdapterOptions {
                    power_preference: PowerPreference::HighPerformance,
                    compatible_surface: None,
                    force_fallback_adapter: false,
                }).await.is_some()
            })
        })
    }
}
```

**Resultado Esperado:**
- Primeira chamada: ~4.8µs (detecção real)
- Chamadas subsequentes: <1ns (cache lookup)

---

### 4. ⚡ GPU Single-Op: 70-90% mais lenta que CPU

**Sintoma:**
```
lerp (CPU):  12.29 ns  ✅
lerp (GPU):  23.50 ns  (91% mais lenta)

slerp (CPU): 15.72 ns  ✅
slerp (GPU): 26.56 ns  (69% mais lenta)
```

**Causa:** Overhead de dispatch GPU não compensa para operações simples

**Componentes do Overhead:**
1. **Command buffer creation** (~5ns)
2. **Bind group setup** (~3ns)
3. **Dispatch call** (~2ns)
4. **GPU queue sync** (~8ns)
5. **Result readback** (~5ns)

**Total overhead:** ~23ns → Mais que o tempo da operação!

**Quando GPU compensa:**

| Operação | Breakeven Point | Ganho Máximo |
|----------|-----------------|---------------|
| lerp/slerp | >500 elementos | ~2x em 10K elementos |
| gradient | >200 elementos | ~3x em 10K elementos |
| distance | >1000 elementos | ~10x em 100K elementos |

**Recomendações:**

#### Heurística de Seleção Automática
```rust
impl ProcessorSelector {
    pub fn select_for_interpolation(batch_size: usize) -> ProcessorType {
        match batch_size {
            0..=100 => ProcessorType::Cpu,      // Overhead inviável
            101..=500 => ProcessorType::Cpu,    // CPU ainda melhor
            501..=2000 => ProcessorType::Gpu,   // GPU começa compensar
            _ => ProcessorType::Gpu,            // GPU ótima
        }
    }
}
```

#### Async Batching
```rust
// Buffer de operações pendentes
let mut batch = vec![];

for state in states {
    batch.push(state);
    
    if batch.len() >= 500 {  // Threshold de eficiência GPU
        gpu_ctx.lerp_batch(&batch).await?;
        batch.clear();
    }
}
```

---

## 🎯 Priorização de Fixes

### P0 - CRÍTICO (Hot Fix Hoje)
1. ✅ **Cache de `is_available()`** → Elimina regressão de +21,000%
2. ✅ **Singleton `GpuContext`** → Amortiza overhead de 700µs

### P1 - Alto (Sprint Atual)
3. 🔄 **VSP JIT Prototype** → PoC com LLVM/Cranelift
4. 🔄 **Auto-selection heuristics** → Escolher CPU/GPU baseado em batch size

### P2 - Médio (Próximo Release)
5. ⏳ **Pre-compiled shaders** → Reduzir overhead de compilação
6. ⏳ **Async GPU ops** → Batching automático

### P3 - Baixo (Roadmap)
7. ⏳ **AOT VSP compiler** → .silc → native code
8. ⏳ **GPU pipeline pool** → Reutilizar recursos

---

## 🔧 Action Items

- [ ] Implementar cache estático em `GpuContext::is_available()`
- [ ] Implementar singleton pattern em `GpuContext`
- [ ] Adicionar testes de regressão de performance
- [ ] Criar benchmark de breakeven points (CPU vs GPU)
- [ ] Prototipar VSP JIT com Cranelift
- [ ] Documentar guidelines de uso de processadores

---

## 📚 Referências

- [WGPU Performance Guide](https://wgpu.rs/)
- [Cranelift JIT](https://cranelift.dev/)
- [LLVM IR Generation](https://llvm.org/docs/tutorial/)
- [Apple Metal Best Practices](https://developer.apple.com/metal/Metal-Best-Practices-Guide.pdf)

---

**Próximo Review:** 18 de Janeiro de 2026  
**Responsável:** Equipe de Performance SIL-Core
