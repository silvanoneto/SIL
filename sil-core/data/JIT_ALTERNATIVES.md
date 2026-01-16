# Alternativas de JIT para VSP

## 📊 Análise Comparativa

Atualmente usamos **Cranelift** para AOT. Para JIT, temos várias opções:

---

## 1. 🦀 **Cranelift JIT** (Atual no feature gate)

### ✅ Prós
- **Já integrado**: Usamos Cranelift para AOT
- **Rust-native**: Zero-cost abstractions, segurança de memória
- **Compilation rápida**: ~1-5ms por função
- **Tier do Wasmtime**: Produção-ready
- **Multi-platform**: x86_64, ARM64, RISC-V
- **Ótimo documentation**: https://cranelift.dev

### ❌ Contras
- **Performance JIT**: ~3-5x mais lento que LLVM JIT
- **Código emitido**: Menos otimizado que LLVM
- **Overhead**: ~500KB de runtime

### 📐 Use Case
- **Ideal para**: Compilation rápida, startup baixo, embedded
- **Evitar**: Quando precisa máxima performance de execução

### 💻 Implementação Estimada
```rust
use cranelift_jit::{JITBuilder, JITModule};

pub struct VspJit {
    module: JITModule,
    functions: HashMap<String, *const u8>,
}

impl VspJit {
    pub fn compile(&mut self, bytecode: &SilcFile) -> VspResult<()> {
        // Similar ao AOT, mas com JITModule
        let mut ctx = self.module.make_context();
        // ... build IR
        let id = self.module.declare_function("main", Linkage::Export, &sig)?;
        self.module.define_function(id, &mut ctx)?;
        self.module.finalize_definitions()?;
        
        // Get pointer
        let ptr = self.module.get_finalized_function(id);
        self.functions.insert("main".into(), ptr);
        Ok(())
    }
    
    pub fn execute(&self, name: &str, state: &mut SilState) -> i32 {
        let func: fn(*mut SilState) -> i32 = unsafe {
            std::mem::transmute(self.functions[name])
        };
        func(state)
    }
}
```

**Tempo de implementação**: 2-3 dias

---

## 2. 🔥 **LLVM JIT** (via inkwell)

### ✅ Prós
- **Performance máxima**: Código ~2x mais rápido que Cranelift
- **Otimizações agressivas**: Inlining, loop unrolling, vectorização
- **Industry standard**: Usado por Julia, Python (Numba), Rust (rustc)
- **Debug info**: DWARF completo para debugging

### ❌ Contras
- **Compilation lenta**: ~50-100ms por função (10-20x mais lento)
- **Overhead gigante**: ~50-100MB de runtime
- **Complexidade**: API verbosa, lifetimes difíceis
- **Build time**: LLVM demora para compilar

### 📐 Use Case
- **Ideal para**: Long-running computations, HPC, quando tempo de compile não importa
- **Evitar**: Aplicações interativas, low-latency, embedded

### 💻 Implementação Estimada
```rust
use inkwell::context::Context;
use inkwell::execution_engine::ExecutionEngine;

pub struct LlvmJit<'ctx> {
    context: &'ctx Context,
    engine: ExecutionEngine<'ctx>,
}

impl<'ctx> LlvmJit<'ctx> {
    pub fn compile(&mut self, bytecode: &SilcFile) -> VspResult<()> {
        let module = self.context.create_module("vsp");
        let builder = self.context.create_builder();
        
        // Define function signature
        let i64_type = self.context.i64_type();
        let fn_type = i64_type.fn_type(&[i64_type.into()], false);
        let function = module.add_function("main", fn_type, None);
        
        // Build IR
        let entry = self.context.append_basic_block(function, "entry");
        builder.position_at_end(entry);
        // ... translate opcodes
        
        self.engine.add_module(&module)?;
        Ok(())
    }
}
```

**Tempo de implementação**: 1-2 semanas (API complexa)

**Dependência**: 
```toml
inkwell = { version = "0.5", features = ["llvm18-0"] }
```

---

## 3. ⚡ **LuaJIT-style Tracing JIT**

### 💡 Conceito
Não compila bytecode diretamente. Interpreta primeiro, detecta **hot loops**, compila apenas o caminho quente.

### ✅ Prós
- **Startup zero**: Interpretador puro no início
- **Performance excelente**: Em hot paths, ~5-10x mais rápido
- **Memory efficient**: Só compila o que usa
- **Adaptive**: Re-compila com novos tipos/valores

### ❌ Contras
- **Complexidade altíssima**: Precisa implementar tracer completo
- **Profile-guided**: Performance varia com workload
- **Debugging difícil**: Código híbrido interpretado/compilado

### 📐 Use Case
- **Ideal para**: Aplicações com loops quentes claros (ML training, games)
- **Evitar**: Código linear, execuções únicas

### 💻 Implementação Estimada
```rust
pub struct TracingJit {
    interpreter: VspInterpreter,
    hot_spots: HashMap<u32, HotSpot>, // PC -> contador
    compiled: HashMap<u32, CompiledTrace>,
    threshold: u32, // execuções antes de compilar
}

impl TracingJit {
    pub fn execute(&mut self, state: &mut SilState) -> VspResult<i32> {
        loop {
            let pc = state.pc;
            
            // Check se tem versão compilada
            if let Some(trace) = self.compiled.get(&pc) {
                trace.execute(state)?;
                continue;
            }
            
            // Interpretar e contar
            self.interpreter.step(state)?;
            
            // Detectar hot spot
            let count = self.hot_spots.entry(pc).or_insert(0);
            *count += 1;
            
            if *count >= self.threshold {
                // Compilar trace!
                self.compile_trace(pc)?;
            }
        }
    }
    
    fn compile_trace(&mut self, start_pc: u32) -> VspResult<()> {
        // Record trace execution
        let trace = self.record_trace(start_pc)?;
        // Compile to native
        let compiled = cranelift_compile(trace)?;
        self.compiled.insert(start_pc, compiled);
        Ok(())
    }
}
```

**Tempo de implementação**: 3-4 semanas (muito complexo)

---

## 4. 🎯 **DynASM** (Runtime Assembly)

### 💡 Conceito
Gera código assembly diretamente em runtime, sem IR intermediário.

### ✅ Prós
- **Compilation ultra-rápida**: ~0.1-0.5ms por função
- **Overhead mínimo**: ~50KB runtime
- **Performance ótima**: Código manual assembly
- **Controle total**: Sem abstrações

### ❌ Contras
- **Portabilidade zero**: Precisa escrever para x86_64, ARM64, etc separadamente
- **Unsafe**: Muito código `unsafe`, fácil quebrar
- **Manutenção**: Assembly é difícil de manter
- **Debug**: Sem stack traces, sem DWARF

### 📐 Use Case
- **Ideal para**: Quando precisa de compilation instantânea + performance máxima
- **Evitar**: Se portabilidade importa, se equipe não sabe assembly

### 💻 Implementação Estimada
```rust
use dynasmrt::{dynasm, DynasmApi, DynasmLabelApi};

pub struct DynasmJit {
    ops: dynasmrt::x64::Assembler,
}

impl DynasmJit {
    pub fn compile(&mut self, bytecode: &SilcFile) -> VspResult<*const u8> {
        dynasm!(self.ops
            ; .arch x64
            ; ->main:
            ; push rbp
            ; mov rbp, rsp
        );
        
        for inst in &bytecode.instructions {
            match inst.opcode {
                Opcode::Nop => {
                    dynasm!(self.ops
                        ; nop
                    );
                }
                Opcode::Add { dst, src } => {
                    dynasm!(self.ops
                        ; mov rax, [rdi + (dst * 8) as i32]
                        ; add rax, [rdi + (src * 8) as i32]
                        ; mov [rdi + (dst * 8) as i32], rax
                    );
                }
                // ... outros opcodes
            }
        }
        
        dynasm!(self.ops
            ; xor rax, rax
            ; pop rbp
            ; ret
        );
        
        let buf = self.ops.finalize().unwrap();
        Ok(buf.ptr(AssemblyOffset(0)))
    }
}
```

**Tempo de implementação**: 2-3 semanas (assembly para cada opcode)

**Dependência**:
```toml
dynasmrt = "2.0"
```

---

## 5. 🌐 **WebAssembly (Wasmtime)**

### 💡 Conceito
Compila bytecode VSP → WASM → JIT execution via Wasmtime

### ✅ Prós
- **Sandboxing**: Isolamento de memória, segurança
- **Portabilidade**: Roda em qualquer lugar (browser, server, edge)
- **Tooling**: wasm-opt, wasm-pack, debugging maduro
- **Interop**: Fácil chamar de JS/Python/etc

### ❌ Contras
- **Overhead de tradução**: VSP → WASM = custo extra
- **Performance**: ~10-20% mais lento que native
- **Limitações**: Linear memory, sem threads nativos
- **Complexidade**: Mais uma camada

### 📐 Use Case
- **Ideal para**: Plugin systems, sandboxed execution, web deployment
- **Evitar**: High-performance computing, quando latência crítica

### 💻 Implementação Estimada
```rust
use wasmtime::*;

pub struct WasmJit {
    engine: Engine,
    store: Store<()>,
}

impl WasmJit {
    pub fn compile(&mut self, bytecode: &SilcFile) -> VspResult<Module> {
        // Translate VSP → WAT (WebAssembly Text)
        let wat = self.vsp_to_wat(bytecode)?;
        
        // Compile to WASM
        let module = Module::new(&self.engine, wat)?;
        Ok(module)
    }
    
    fn vsp_to_wat(&self, bytecode: &SilcFile) -> VspResult<String> {
        let mut wat = String::from("(module\n");
        wat.push_str("  (memory 1)\n");
        wat.push_str("  (func (export \"main\") (param $state i32) (result i32)\n");
        
        for inst in &bytecode.instructions {
            match inst.opcode {
                Opcode::Nop => wat.push_str("    nop\n"),
                Opcode::Add { .. } => wat.push_str("    i32.add\n"),
                // ...
            }
        }
        
        wat.push_str("    i32.const 0\n  )\n)");
        Ok(wat)
    }
}
```

**Tempo de implementação**: 1 semana

**Dependência**:
```toml
wasmtime = "26.0"
```

---

## 6. 🚀 **YJIT-style Lazy Basic Block Versioning**

### 💡 Conceito
Usado pelo Ruby 3.1+. Compila blocos básicos sob demanda, versiona por tipo.

### ✅ Prós
- **Compilation incremental**: Só compila o que executa
- **Type specialization**: Código diferente para Int64 vs Float
- **Memory efficient**: Blocos pequenos
- **Fast tier-up**: Interpretador → JIT suave

### ❌ Contras
- **Fragmentação**: Muitas versões do mesmo código
- **Overhead de dispatch**: Precisa escolher versão certa
- **Complexidade**: Code cache management

### 📐 Use Case
- **Ideal para**: Dynamic languages, quando tipos variam
- **Evitar**: Linguagens estaticamente tipadas (como VSP)

**Nota**: Provavelmente **overkill** para VSP, pois bytecode é estaticamente tipado.

---

## 📊 Tabela Comparativa

| Critério | Cranelift JIT | LLVM JIT | Tracing JIT | DynASM | WASM |
|----------|--------------|----------|-------------|---------|------|
| **Compile Speed** | ⚡⚡⚡⚡ (1-5ms) | ⚡ (50-100ms) | ⚡⚡⚡⚡⚡ (0.1ms) | ⚡⚡⚡⚡⚡ (0.1ms) | ⚡⚡⚡ (10ms) |
| **Runtime Speed** | ⚡⚡⚡ (3x) | ⚡⚡⚡⚡⚡ (10x) | ⚡⚡⚡⚡ (5-10x) | ⚡⚡⚡⚡⚡ (10x) | ⚡⚡⚡ (5x) |
| **Memory** | 500KB | 50-100MB | 1-5MB | 50KB | 2-5MB |
| **Portabilidade** | ✅✅✅✅✅ | ✅✅✅✅ | ✅✅✅ | ❌ | ✅✅✅✅✅ |
| **Complexidade** | 🟢 Média | 🔴 Alta | 🔴 Muito Alta | 🟠 Alta | 🟢 Média |
| **Debugging** | ✅ Bom | ✅✅ Excelente | ⚠️ Difícil | ❌ Muito Difícil | ✅ Bom |
| **Impl. Time** | 2-3 dias | 1-2 semanas | 3-4 semanas | 2-3 semanas | 1 semana |

---

## 🎯 Recomendação para VSP

### 🥇 **Primeira Escolha: Cranelift JIT**

**Razões**:
1. ✅ **Já temos AOT**: Reutilizar 80% do código
2. ✅ **Balance ideal**: Compile rápida + performance boa
3. ✅ **Rust-native**: Segurança de memória, zero-cost
4. ✅ **Produção**: Wasmtime usa em produção
5. ✅ **Implementação rápida**: 2-3 dias

**Arquitetura proposta**:
```rust
// src/vsp/jit.rs
pub struct VspJit {
    module: JITModule,
    state: HashMap<String, *const u8>,
}

// Shared com AOT
fn build_function_ir(
    builder: &mut FunctionBuilder,
    bytecode: &SilcFile,
) -> VspResult<()> {
    // Mesma lógica de src/vsp/aot.rs
    // quando implementarmos compile_instructions()
}
```

### 🥈 **Segunda Escolha: DynASM**

Se Cranelift for muito lento (improvável), **DynASM** oferece compilation instantânea.

**Quando considerar**:
- Latency < 1ms é crítico
- VSP roda em hot path (milhões de execuções/segundo)
- Só precisa suportar x86_64 + ARM64

### 🥉 **Terceira Escolha: WASM (via Wasmtime)**

Se precisar de **sandboxing** ou **portabilidade máxima** (incluindo browser).

**Use cases**:
- Plugin system (executar código third-party)
- Edge computing (CloudFlare Workers, etc)
- Web deployment (VSP no browser via WASM)

---

## 📝 Plano de Implementação (Cranelift JIT)

### Fase 1: Setup (2-4 horas)
```toml
# Cargo.toml - já temos!
cranelift-jit = { version = "0.113", optional = true }
```

### Fase 2: Core JIT (1 dia)
```rust
// src/vsp/jit.rs
use cranelift_jit::{JITBuilder, JITModule};

pub struct VspJit {
    builder: JITBuilder,
    module: JITModule,
    compiled_functions: HashMap<String, *const u8>,
}

impl VspJit {
    pub fn new() -> VspResult<Self> {
        let mut builder = JITBuilder::new(cranelift_module::default_libcall_names())?;
        builder.hotswap(true); // Permite recompilação
        let module = JITModule::new(builder);
        
        Ok(Self {
            builder,
            module,
            compiled_functions: HashMap::new(),
        })
    }
    
    pub fn compile(&mut self, name: &str, bytecode: SilcFile) -> VspResult<()> {
        // Reusar código de aot.rs
        let mut ctx = self.module.make_context();
        let mut fn_builder_ctx = FunctionBuilderContext::new();
        
        // Build function (compartilhar com AOT)
        build_vsp_function(&mut ctx, &mut fn_builder_ctx, &bytecode)?;
        
        let id = self.module.declare_function(name, Linkage::Export, &ctx.func.signature)?;
        self.module.define_function(id, &mut ctx)?;
        self.module.clear_context(&mut ctx);
        
        // Finalizar e obter ponteiro
        self.module.finalize_definitions()?;
        let code_ptr = self.module.get_finalized_function(id);
        self.compiled_functions.insert(name.to_string(), code_ptr);
        
        Ok(())
    }
    
    pub fn execute(&self, name: &str, state: &mut SilState) -> VspResult<i32> {
        let func_ptr = self.compiled_functions.get(name)
            .ok_or_else(|| VspError::Other(format!("Function {} not compiled", name)))?;
        
        let func: unsafe extern "C" fn(*mut SilState) -> i32 = unsafe {
            std::mem::transmute(*func_ptr)
        };
        
        Ok(unsafe { func(state) })
    }
}
```

### Fase 3: Shared IR Builder (1 dia)
```rust
// src/vsp/codegen.rs (novo arquivo)
// Compartilhado entre JIT e AOT

pub fn build_vsp_function(
    ctx: &mut Context,
    fn_ctx: &mut FunctionBuilderContext,
    bytecode: &SilcFile,
) -> VspResult<()> {
    let mut builder = FunctionBuilder::new(&mut ctx.func, fn_ctx);
    
    let entry = builder.create_block();
    builder.append_block_params_for_function_params(entry);
    builder.switch_to_block(entry);
    builder.seal_block(entry);
    
    // Traduzir opcodes para IR
    compile_instructions(&mut builder, bytecode)?;
    
    let zero = builder.ins().iconst(types::I32, 0);
    builder.ins().return_(&[zero]);
    builder.finalize();
    
    Ok(())
}

fn compile_instructions(
    builder: &mut FunctionBuilder,
    bytecode: &SilcFile,
) -> VspResult<()> {
    // TODO: Implementar tradução de cada opcode
    // (Mesmo código para JIT e AOT)
    Ok(())
}
```

### Fase 4: Benchmarks (4 horas)
```rust
// benches/jit_vs_interpreter.rs
#[bench]
fn bench_interpreter(b: &mut Bencher) { ... }

#[bench]
fn bench_jit_cold(b: &mut Bencher) { ... } // Primeira execução

#[bench]
fn bench_jit_warm(b: &mut Bencher) { ... } // Pós-compilação
```

### Fase 5: Examples (2 horas)
```rust
// examples/vsp_jit.rs
fn main() {
    let mut jit = VspJit::new()?;
    let bytecode = SilcFile::from_file("program.silc")?;
    
    println!("⏱️  Compiling...");
    let start = Instant::now();
    jit.compile("main", bytecode)?;
    println!("✓ Compiled in {:?}", start.elapsed());
    
    println!("🚀 Executing...");
    let mut state = SilState::new();
    let result = jit.execute("main", &mut state)?;
    println!("✓ Result: {}", result);
}
```

**Tempo total estimado**: 2-3 dias de trabalho focado

---

## 🔬 Experimentos Futuros

### Hybrid JIT/AOT
```rust
pub enum CompileMode {
    Interpret,           // Cold path
    JIT(OptLevel::None), // Warm-up
    AOT(OptLevel::Speed), // Hot path
}

// Auto tier-up baseado em contadores
```

### Multi-tier JIT (como V8)
1. **Ignition** (interpreter) → cold start
2. **Sparkplug** (fast JIT, sem otimização) → warm-up
3. **TurboFan** (optimizing JIT) → hot code

Para VSP:
1. Interpreter → 0ms startup
2. Cranelift JIT (O0) → primeira execução, ~1ms compile
3. Cranelift JIT (O2) → hot loops, ~5ms recompile

---

## 🎓 Referências

- [Cranelift JIT Tutorial](https://github.com/bytecodealliance/wasmtime/blob/main/cranelift/docs/index.md)
- [LuaJIT Tracing](http://wiki.luajit.org/SSA-IR-2.0)
- [YJIT Design](https://shopify.engineering/yjit-just-in-time-compiler-cruby)
- [DynASM Examples](https://censoredusername.github.io/dynasm-rs/language/index.html)
- [Inkwell (LLVM) Tutorial](https://github.com/TheDan64/inkwell)

---

**Conclusão**: Para VSP, **Cranelift JIT** é a escolha óbvia. Mesmo código do AOT, implementação rápida, performance sólida. 🎯
