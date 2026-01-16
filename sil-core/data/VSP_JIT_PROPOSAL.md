# 🚀 VSP JIT Compilation - Design Proposal

**Problema:** VSP interpretado tem overhead de 46,400x comparado a operações nativas  
**Meta:** Reduzir overhead para <10x mantendo portabilidade  
**Abordagem:** Just-In-Time compilation usando Cranelift

---

## 🎯 Objetivos

1. **Performance:** Overhead <10x vs código nativo
2. **Portabilidade:** Cross-platform (macOS, Linux, Windows, WASM)
3. **Compatibilidade:** Manter API VSP existente
4. **Segurança:** Sandbox de execução
5. **Startup rápido:** Compilação <1ms para programas típicos

---

## 🏗️ Arquitetura

```text
┌─────────────────────────────────────────────────────────────────┐
│                          VSP Pipeline                           │
│                                                                 │
│  .sil source                                                    │
│       ↓                                                         │
│  ┌──────────┐                                                   │
│  │Assembler │  → .silc bytecode                                │
│  └──────────┘                                                   │
│       ↓                                                         │
│  ┌──────────┐                                                   │
│  │  Loader  │  → Load bytecode                                 │
│  └──────────┘                                                   │
│       ↓                                                         │
│  ┌──────────────────────────────────────────┐                  │
│  │         Execution Strategy               │                  │
│  │  ┌────────┐  ┌─────────┐  ┌─────────┐   │                  │
│  │  │Interp. │  │   JIT   │  │   AOT   │   │                  │
│  │  │(current)│  │(planned)│  │(future) │   │                  │
│  │  └────────┘  └─────────┘  └─────────┘   │                  │
│  └──────────────────────────────────────────┘                  │
│       ↓              ↓              ↓                           │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                     │
│  │  Loop    │  │ Cranelift│  │  Static  │                     │
│  │Interpret │  │  Compile │  │   .so    │                     │
│  └──────────┘  └──────────┘  └──────────┘                     │
│       ↓              ↓              ↓                           │
│  Native execution (CPU/GPU/NPU)                                │
└─────────────────────────────────────────────────────────────────┘
```

---

## 🔧 Implementation Plan

### Phase 1: Cranelift Integration (2 semanas)

**Dependencies:**
```toml
[dependencies]
cranelift = "0.108"
cranelift-module = "0.108"
cranelift-jit = "0.108"
target-lexicon = "0.12"
```

**Core JIT Module:**
```rust
// src/vsp/jit/mod.rs
use cranelift::prelude::*;
use cranelift_module::{Module, FuncId};
use cranelift_jit::{JITModule, JITBuilder};

pub struct VspJit {
    module: JITModule,
    ctx: codegen::Context,
    func_ids: HashMap<String, FuncId>,
}

impl VspJit {
    pub fn new() -> Result<Self, JitError> {
        let builder = JITBuilder::new(cranelift_module::default_libcall_names())?;
        let module = JITModule::new(builder);
        
        Ok(Self {
            module,
            ctx: module.make_context(),
            func_ids: HashMap::new(),
        })
    }
    
    /// Compila bytecode VSP para código nativo
    pub fn compile(&mut self, bytecode: &[u8]) -> Result<CompiledFunction, JitError> {
        // Decodificar bytecode
        let instructions = decode_all(bytecode)?;
        
        // Criar função Cranelift
        let mut builder_ctx = FunctionBuilderContext::new();
        let mut func_builder = FunctionBuilder::new(&mut self.ctx.func, &mut builder_ctx);
        
        // Entry block
        let entry_block = func_builder.create_block();
        func_builder.append_block_params_for_function_params(entry_block);
        func_builder.switch_to_block(entry_block);
        
        // Traduzir cada instrução VSP → Cranelift IR
        for inst in instructions {
            self.translate_instruction(&mut func_builder, &inst)?;
        }
        
        // Finalizar
        func_builder.seal_all_blocks();
        func_builder.finalize();
        
        // Compilar para código nativo
        let func_id = self.module.declare_function(
            "vsp_main",
            cranelift_module::Linkage::Local,
            &self.ctx.func.signature
        )?;
        
        self.module.define_function(func_id, &mut self.ctx)?;
        self.module.finalize_definitions()?;
        
        // Obter ponteiro de função
        let code_ptr = self.module.get_finalized_function(func_id);
        
        Ok(CompiledFunction { code_ptr, func_id })
    }
    
    /// Traduz instrução VSP para Cranelift IR
    fn translate_instruction(
        &mut self,
        builder: &mut FunctionBuilder,
        inst: &Instruction
    ) -> Result<(), JitError> {
        match inst.opcode {
            Opcode::Add => {
                // ADD R0, R1 → result = r0 + r1
                let r0 = builder.use_var(Variable::from_u32(inst.r0 as u32));
                let r1 = builder.use_var(Variable::from_u32(inst.r1 as u32));
                let result = builder.ins().iadd(r0, r1);
                builder.def_var(Variable::from_u32(inst.rd as u32), result);
            }
            
            Opcode::Mul => {
                let r0 = builder.use_var(Variable::from_u32(inst.r0 as u32));
                let r1 = builder.use_var(Variable::from_u32(inst.r1 as u32));
                let result = builder.ins().imul(r0, r1);
                builder.def_var(Variable::from_u32(inst.rd as u32), result);
            }
            
            Opcode::Movi => {
                // MOVI R0, #imm → r0 = imm
                let imm = builder.ins().iconst(types::I64, inst.immediate as i64);
                builder.def_var(Variable::from_u32(inst.rd as u32), imm);
            }
            
            Opcode::Jmp => {
                // JMP label → goto label
                let target_block = self.get_or_create_block(inst.target_addr);
                builder.ins().jump(target_block, &[]);
            }
            
            Opcode::Hlt => {
                // HLT → return
                builder.ins().return_(&[]);
            }
            
            // ... outros opcodes
            _ => return Err(JitError::UnsupportedOpcode(inst.opcode)),
        }
        
        Ok(())
    }
}
```

### Phase 2: Otimizações (1 semana)

**Hot Path Detection:**
```rust
impl Vsp {
    fn detect_hot_paths(&mut self) -> Vec<HotPath> {
        // Instrumentar interpretador para contar execuções
        let mut hotness = HashMap::new();
        
        for &pc in &self.executed_pcs {
            *hotness.entry(pc).or_insert(0) += 1;
        }
        
        // Identificar loops/funções hot (>1000 execuções)
        hotness.into_iter()
            .filter(|(_, count)| *count > 1000)
            .map(|(pc, count)| HotPath { start_pc: pc, count })
            .collect()
    }
    
    fn compile_hot_path(&mut self, path: &HotPath) -> Result<(), VspError> {
        let bytecode = self.memory.code_segment(path.start_pc..path.end_pc);
        let compiled = self.jit.compile(bytecode)?;
        self.compiled_cache.insert(path.start_pc, compiled);
        Ok(())
    }
}
```

**Tiered Compilation:**
```rust
pub enum ExecutionTier {
    Interpreter,   // Execução inicial (fria)
    JitTier1,      // Compilação rápida (-O0)
    JitTier2,      // Compilação otimizada (-O2)
}

impl Vsp {
    fn execute_instruction(&mut self, inst: Instruction) -> Result<(), VspError> {
        let hotness = self.get_hotness(self.state.pc);
        
        match hotness {
            0..=100 => {
                // Cold: Interpretar
                self.interpret(inst)
            }
            101..=1000 => {
                // Warm: Compilar Tier 1 (rápido)
                self.jit_compile_tier1(inst)
            }
            _ => {
                // Hot: Compilar Tier 2 (otimizado)
                self.jit_compile_tier2(inst)
            }
        }
    }
}
```

### Phase 3: Integration com Backend Selector (1 semana)

**Backend Dispatch via JIT:**
```rust
impl VspJit {
    fn translate_backend_call(
        &mut self,
        builder: &mut FunctionBuilder,
        backend: ProcessorType,
        operation: Operation
    ) -> Result<Value, JitError> {
        match backend {
            ProcessorType::Cpu => {
                // Gerar chamada direta para CpuContext
                let cpu_fn = self.import_function("cpu_gradient");
                builder.ins().call(cpu_fn, &[state_ptr, learning_rate])
            }
            
            ProcessorType::Gpu => {
                // Gerar chamada para GpuContext (se disponível)
                if GpuContext::is_available() {
                    let gpu_fn = self.import_function("gpu_gradient_batch");
                    builder.ins().call(gpu_fn, &[batch_ptr, batch_size])
                } else {
                    // Fallback CPU
                    self.translate_backend_call(builder, ProcessorType::Cpu, operation)
                }
            }
            
            ProcessorType::Npu => {
                // Gerar chamada para NpuContext
                let npu_fn = self.import_function("npu_infer");
                builder.ins().call(npu_fn, &[model_ptr, input_ptr])
            }
        }
    }
}
```

---

## 📊 Performance Targets

| Métrica | Interpretado | JIT Tier 1 | JIT Tier 2 | Native |
|---------|--------------|------------|------------|--------|
| ADD operação | 679µs | 50µs | 20µs | 14.6ns |
| Overhead vs native | 46,400x | ~3,400x | ~1,370x | 1x |
| Compile time | N/A | <1ms | <10ms | N/A |
| Loop (1000 iter) | ~680ms | ~50ms | ~20ms | ~14µs |

---

## 🧪 Validation Strategy

**Benchmark Suite:**
```rust
#[cfg(test)]
mod jit_benchmarks {
    #[bench]
    fn bench_vsp_add_interpreted(b: &mut Bencher) {
        let vsp = Vsp::new_interpreted();
        b.iter(|| vsp.execute_add())
    }
    
    #[bench]
    fn bench_vsp_add_jit_tier1(b: &mut Bencher) {
        let vsp = Vsp::new_jit(JitTier::Fast);
        b.iter(|| vsp.execute_add())
    }
    
    #[bench]
    fn bench_vsp_add_jit_tier2(b: &mut Bencher) {
        let vsp = Vsp::new_jit(JitTier::Optimized);
        b.iter(|| vsp.execute_add())
    }
}
```

**Correctness Tests:**
```rust
#[test]
fn test_jit_correctness() {
    let program = assemble("
        MOVI R0, 5
        MOVI R1, 10
        ADD R0, R1
        HLT
    ").unwrap();
    
    // Executar interpretado
    let mut vsp_interp = Vsp::new_interpreted();
    vsp_interp.load_bytes(&program);
    let result_interp = vsp_interp.run().unwrap();
    
    // Executar JIT
    let mut vsp_jit = Vsp::new_jit();
    vsp_jit.load_bytes(&program);
    let result_jit = vsp_jit.run().unwrap();
    
    // Devem ser idênticos
    assert_eq!(result_interp, result_jit);
    assert_eq!(result_jit.r[0].to_u8(), 15);
}
```

---

## 🗺️ Roadmap

### Sprint 1 (2 semanas) - MVP JIT
- [ ] Integrar Cranelift
- [ ] Implementar tradutor básico (MOVI, ADD, MUL, HLT)
- [ ] Benchmark vs interpretador
- [ ] Target: <10,000x overhead

### Sprint 2 (2 semanas) - Full ISA
- [ ] Suportar todos os 70+ opcodes
- [ ] Implementar jumps/branches
- [ ] Suportar calls/returns
- [ ] Target: <5,000x overhead

### Sprint 3 (1 semana) - Otimizações
- [ ] Hot path detection
- [ ] Tiered compilation
- [ ] Register allocation
- [ ] Target: <1,000x overhead

### Sprint 4 (1 semana) - Integration
- [ ] Integrar com backend selector
- [ ] Modo híbrido (JIT + interpreted)
- [ ] Performance profiling
- [ ] Target: <100x overhead

### Future
- [ ] AOT compilation (.silc → .so)
- [ ] SIMD vectorization
- [ ] GPU kernel generation
- [ ] Target: <10x overhead

---

## 🔗 References

- [Cranelift Documentation](https://cranelift.dev/)
- [Writing a JIT Compiler](https://github.com/bytecodealliance/wasmtime/tree/main/cranelift/docs)
- [JavaScriptCore FTL JIT](https://webkit.org/blog/3362/introducing-the-webkit-ftl-jit/)
- [LuaJIT Architecture](http://luajit.org/luajit.html)
- [PyPy JIT Design](https://doc.pypy.org/en/latest/jit/)

---

**Próxima Revisão:** 25 de Janeiro de 2026  
**Owner:** VSP Performance Team
