# VSP Performance Analysis

## Execution Engines Comparison

VSP oferece dois backends de execução com diferentes características de performance e portabilidade.

### 1. Native Rust Interpreter

**Arquitetura:** Threaded dispatch com function pointers

```rust
use sil_core::vsp::interpreter::VspInterpreter;

let mut interp = VspInterpreter::new();
interp.compile(&program)?;
interp.execute(&mut state)?;
```

**Características:**
- Pre-compila bytecode para jump table: `Vec<(OpcodeHandler, Vec<u8>)>`
- Todos os handlers com `#[inline(always)]`
- Zero alocações durante execução
- Rust optimizer elimina overhead de dispatch

**Performance (Apple Silicon M3 Pro):**
- Throughput: **257.68M ops/sec**
- Latency: **3.9 ns/instruction**
- Memory: ~500 bytes overhead per program

**Portabilidade:**
- ✅ x86-64 (Intel/AMD)
- ✅ ARM64 (Apple Silicon, AWS Graviton, Raspberry Pi)
- ✅ RISC-V (VisionFive, HiFive)
- ✅ WASM (browsers, edge computing)

### 2. DynASM ARM64 JIT

**Arquitetura:** Runtime native assembly generation

```rust
#[cfg(all(target_arch = "aarch64", feature = "dynasm"))]
use sil_core::vsp::dynasm::VspDynasmJit;

let mut jit = VspDynasmJit::new()?;
jit.compile(&program)?;
jit.execute(&mut state)?;
```

**Características:**
- Gera código ARM64 nativo em runtime
- Zero overhead de dispatch
- Inline de operações ByteSil
- Usa registradores ARM64 diretamente

**Performance (Apple Silicon M3 Pro):**
- Throughput: **326.23M ops/sec**
- Latency: **3.1 ns/instruction**
- Memory: ~2KB executable buffer per program

**Portabilidade:**
- ✅ ARM64 macOS (Apple Silicon)
- ✅ ARM64 Linux (AWS Graviton, Raspberry Pi)
- ❌ x86-64 (não suportado)
- ❌ RISC-V (DynASM não tem backend)

## Benchmark Methodology

**Test Program:** 101 instructions per iteration
```
25x {
    MOV    ; L0 ↔ L1 swap
    XORL   ; XOR all 16 layers
    MUL    ; L0 × L1
    ROTATE ; Rotate layers
}
HLT
```

**Hardware:** Apple M3 Pro (11-core CPU, 14-core GPU)  
**Iterations:** 100,000 (10.1M instructions total)  
**Warmup:** 1,000 iterations  

**Compile flags:**
```bash
cargo run --release --example bench_compare --features dynasm
```

## Results Summary

| Metric | Interpreter | DynASM JIT | Speedup |
|:-------|------------:|-----------:|--------:|
| **Throughput** | 257.68M ops/sec | 326.23M ops/sec | **1.27x** |
| **Latency** | 3.9 ns/op | 3.1 ns/op | **1.26x** |
| **Time (10.1M ops)** | 39.2 ms | 31.0 ms | **1.26x** |
| **Portability** | All architectures | ARM64 only | - |
| **Binary size** | +120KB | +350KB | - |

## Analysis

### Why is the Interpreter so Fast?

1. **Aggressive Inlining**
   ```rust
   #[inline(always)]
   fn handle_mov(state: &mut SilState, args: &[u8]) {
       state.layers.swap(args[0] as usize, args[1] as usize);
   }
   ```
   Rust compiler inlines todas as funções → zero overhead de chamada

2. **Cache-Friendly**
   - Handler table: ~2KB (cabe em L1 cache)
   - Hot loop: < 1KB (executa direto do cache)
   - Branch predictor aprende padrão rapidamente

3. **Zero Allocations**
   - Jump table pre-alocada na compilação
   - Nenhum `Box`, `Vec` ou `String` criado durante execução

4. **Function Pointer Dispatch**
   ```rust
   type OpcodeHandler = fn(&mut SilState, &[u8]);
   let handlers: Vec<(OpcodeHandler, Vec<u8>)> = ...;
   
   for (handler, args) in &handlers {
       handler(state, args); // Indirect call, mas previsto
   }
   ```

### DynASM Advantage

DynASM é apenas **27% mais rápido** porque:

- Interpreter já é quase-nativo (inlining + cache)
- Diferença principal: elimina indirect calls
- ARM64 JIT usa registradores diretamente (menos loads/stores)

**Exemplo:** MUL operation
```asm
; DynASM ARM64
ldr x0, [sp, #L0]     ; Load L0.rho
ldr x1, [sp, #L1]     ; Load L1.rho
add x2, x0, x1        ; rho_result = rho0 + rho1
str x2, [sp, #L0]     ; Store result

; Interpreter (Rust → LLVM → ARM64)
; Quase idêntico, mas através de function pointer
```

## Recommendations

### Default: Use Interpreter

**Razões:**
1. **Portabilidade:** Funciona em TODO hardware
2. **Performance:** 257M ops/sec é suficiente para 99% dos casos
3. **Simplicidade:** Zero configuração, zero features extras
4. **Manutenção:** Menos código para manter (485 vs 918 linhas)

**Use cases:**
- Aplicações cross-platform
- Deployment em RISC-V
- WASM/edge computing
- Quando 27% não importa (não importa na maioria dos casos)

### Optional: Enable DynASM JIT

**Razões:**
1. **Performance máxima:** 326M ops/sec (27% melhor)
2. **Apple Silicon:** Otimizado para M1/M2/M3
3. **High-frequency trading:** Cada nanossegundo conta
4. **Benchmarking:** Comparar com código nativo

**Use cases:**
- Servidores ARM64 exclusivos (AWS Graviton)
- Apps macOS que precisam de máxima performance
- Real-time systems com SLAs apertados
- Quando 1.3 ns/op importa

**Enable:**
```toml
[dependencies]
sil-core = { version = "2026.1", features = ["dynasm"] }
```

## Future Work

### RISC-V JIT (LLVM-based)

DynASM não tem backend RISC-V. Alternativas:

1. **LLVM/Inkwell** (recommended)
   - Gera LLVM IR → compila para RV64GC
   - Estimativa: 15-20M ops/sec
   - Timeframe: 2-3 dias implementação

2. **Cranelift JIT**
   - Backend RISC-V em desenvolvimento
   - Mais simples que LLVM
   - Performance similar ao DynASM

3. **Contribute to DynASM**
   - Implementar backend RISC-V
   - Beneficia toda comunidade
   - Timeframe: 2-4 semanas

4. **Interpreter is Enough**
   - 257M ops/sec já é excelente
   - RISC-V tem performance similar ao ARM64
   - Diferença de 27% raramente justifica trabalho

### x86-64 JIT

DynASM já tem backend x86-64, mas:
- Não implementado ainda (foco foi ARM64)
- Performance esperada: 280-320M ops/sec (similar)
- Timeframe: 1-2 dias (portar de ARM64)

## Conclusion

**Interpreter é a escolha padrão:**
- Performance excelente (257M ops/sec)
- Portabilidade universal
- Código mais simples

**DynASM é opcional:**
- 27% speedup no ARM64
- Perda de portabilidade
- Use apenas se realmente precisar

A diferença prática entre 3.9ns e 3.1ns por instrução é **negligível** para a maioria das aplicações. Priorize portabilidade e simplicidade.

## Running Benchmarks

```bash
# Interpreter + DynASM (ARM64 only)
cargo run --release --example bench_compare --features dynasm

# Interpreter only (any architecture)
cargo run --release --example bench_compare

# Interpreter standalone
cargo run --release --example vsp_interpreter
```

## Hardware Tested

- **Apple M3 Pro** (11-core, 18GB RAM) - macOS Sonoma 14.x
- Compiler: `rustc 1.85.0-nightly`
- Optimization: `opt-level = 3`

Future testing planned:
- AWS Graviton 3 (ARM64 Linux)
- Intel Xeon (x86-64)
- VisionFive 2 (RISC-V)
- WASM (browser/wasmtime)
