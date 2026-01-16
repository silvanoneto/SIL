# 🚀 VSP AOT Compiler

Compilador Ahead-Of-Time para VSP (Virtual SIL Processor), transformando bytecode em código nativo executável.

## 🎯 Objetivo

Eliminar overhead de interpretação e JIT compilation através de compilação antecipada:

- **Build-time compilation**: Compilar durante o build
- **Zero startup overhead**: Código nativo pronto para execução
- **Maximum performance**: Sem interpretação ou JIT warmup
- **Deployment ready**: Distribuir binários otimizados

## 📐 Arquitetura

```
┌─────────────┐
│  VSP Source │
│   (.sil)    │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Assembler  │  silasm
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Bytecode   │
│   (.vsp)    │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ AOT Compiler│  vsp-aot ⭐
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Native Code │
│ (.o / .dylib)│
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Execution  │  Zero overhead!
└─────────────┘
```

## 🚀 Uso

### CLI Tool

```bash
# Compilar bytecode para código nativo
$ vsp-aot compile program.vsp

# Com otimizações
$ vsp-aot compile program.vsp -O3

# Especificar output
$ vsp-aot compile program.vsp -o program.o

# Usar cache de compilação
$ vsp-aot compile program.vsp --cache

# Ver informações de compilação
$ vsp-aot info program.o
```

### API Rust

```rust
use sil_core::vsp::{ByteCode, aot::{AotCompiler, OptLevel}};

// Carregar bytecode
let bytecode = ByteCode::from_bytes(&data)?;

// Criar compilador
let compiler = AotCompiler::new()
    .with_opt_level(OptLevel::SpeedAndSize);

// Compilar
let compilation = compiler.compile("my_program", bytecode)?;

// Salvar objeto
compiler.save(&compilation, Path::new("program.o"))?;

println!("Compiled {} bytes of native code", 
         compilation.metadata.code_size);
```

### Com Cache

```rust
use sil_core::vsp::aot::AotCache;

// Criar cache
let mut cache = AotCache::new("./aot_cache")?;

// Compilar e cachear
let path = cache.put(&compilation)?;
println!("Cached at {}", path.display());

// Buscar do cache
if let Some(cached) = cache.get("my_program") {
    println!("Found in cache: {}", cached.display());
}

// Estatísticas
let stats = cache.stats();
println!("Cache: {} entries, {} bytes", 
         stats.num_entries, stats.total_size_bytes);
```

## ⚙️ Níveis de Otimização

### `-O0` (None)
- Sem otimizações
- Debug symbols completos
- Compilação rápida
- **Use para**: Development, debugging

### `-O2` (Speed) - Default
- Otimizações balanceadas
- Boa performance
- Tamanho razoável
- **Use para**: Production padrão

### `-O3` / `-Os` (SpeedAndSize)
- Otimizações agressivas
- Máxima performance
- Código otimizado em tamanho
- **Use para**: Production crítica, embedded

## 📊 Performance

### Comparação: Interpreter vs JIT vs AOT

| Método | Startup | Execution | Memory |
|--------|---------|-----------|--------|
| **Interpreter** | Instant | Slow | Low |
| **JIT** | Warmup | Fast | Medium |
| **AOT** | Zero | Fastest | Low |

### Benchmark Esperado

```
Fibonacci(30):
  Interpreter: ~850ms
  JIT:         ~120ms (após warmup)
  AOT:         ~60ms  (2x faster!) ⭐
```

## 🔧 Build Integration

### build.rs

```rust
// build.rs
use sil_core::vsp::aot::AotCompiler;

fn main() {
    // Compilar todos .vsp para código nativo
    let compiler = AotCompiler::new();
    
    for entry in glob::glob("programs/*.vsp").unwrap() {
        let path = entry.unwrap();
        let bytecode = ByteCode::from_file(&path).unwrap();
        let name = path.file_stem().unwrap().to_str().unwrap();
        
        let compilation = compiler.compile(name, bytecode).unwrap();
        
        let out_dir = env::var("OUT_DIR").unwrap();
        let output = Path::new(&out_dir).join(format!("{}.o", name));
        
        compiler.save(&compilation, &output).unwrap();
        
        println!("cargo:rerun-if-changed={}", path.display());
    }
}
```

### Cargo.toml

```toml
[build-dependencies]
sil-core = { path = ".", features = ["jit"] }
glob = "0.3"

[features]
precompiled = []  # Feature para usar código AOT
```

## 📦 Distribuição

### Incluir Código Compilado

```rust
// Embed código AOT no binário
const COMPILED_CODE: &[u8] = include_bytes!(
    concat!(env!("OUT_DIR"), "/program.o")
);

fn main() {
    // Carregar código nativo
    // (requer runtime linker ou libloading)
}
```

### Shared Library

```bash
# Compilar para shared library
$ vsp-aot compile program.vsp -O3
$ ld -shared program.o -o libprogram.dylib

# Usar em outro programa
$ gcc main.c -L. -lprogram -o main
```

## 🗂️ Cache de Compilação

### Localização

- **macOS**: `~/Library/Caches/sil-vsp-aot/`
- **Linux**: `~/.cache/sil-vsp-aot/`
- **Windows**: `%LOCALAPPDATA%\sil-vsp-aot\cache\`

### Gerenciamento

```bash
# Listar cache
$ vsp-aot cache list

# Estatísticas
$ vsp-aot cache stats

# Limpar cache
$ vsp-aot cache clear
```

### Cache Inteligente

O cache usa hash do bytecode para invalidação:
- Bytecode alterado = recompilação automática
- Metadados preservados (.meta files)
- Limpeza de cache antigo

## 🔍 Metadados de Compilação

Arquivo `.meta` (JSON):

```json
{
  "compiled_at": 1704931200,
  "compiler_version": "2026.1.0",
  "target_triple": "x86_64-apple-darwin",
  "optimization_level": "SpeedAndSize",
  "code_size": 4096
}
```

## 🛠️ Cranelift Backend

Usa Cranelift como backend de compilação:

- **IR Translation**: VSP bytecode → Cranelift IR
- **Optimization**: SSA-based optimizations
- **Code Generation**: Native assembly
- **Object Format**: ELF / Mach-O / PE

### Suporte de Plataformas

| Arquitetura | Status |
|-------------|--------|
| x86_64 | ✅ Full support |
| aarch64 (Apple Silicon) | ✅ Full support |
| arm | ⚠️ Limited |
| riscv64 | 🔄 Experimental |

## 🐛 Troubleshooting

### "AOT compilation requires 'jit' feature"

```bash
# Compilar com feature jit
$ cargo build --features jit
$ cargo install --path . --features jit
```

### "Compilation failed: ISA not supported"

Sua arquitetura pode não ser suportada. Verifique:

```bash
$ rustc --version --verbose | grep host
```

### Cache Corrupto

```bash
# Limpar e recriar cache
$ vsp-aot cache clear
$ vsp-aot compile program.vsp --cache
```

## 📈 Roadmap

- [x] Cranelift backend integration
- [x] Object file generation (.o)
- [x] Compilation cache
- [x] CLI tool (vsp-aot)
- [x] Metadata preservation
- [ ] Complete instruction compilation
- [ ] Shared library output (.dylib/.so/.dll)
- [ ] Link-time optimization (LTO)
- [ ] Profile-guided optimization (PGO)
- [ ] Cross-compilation support
- [ ] WASM target

## 🎯 Use Cases

### 1. Production Deployment

```bash
# Build
$ vsp-aot compile app.vsp -O3 -o app.o

# Deploy
$ scp app.o server:/opt/app/
```

### 2. Embedded Systems

```bash
# Compile for ARM
$ vsp-aot compile firmware.vsp -O3 --target arm-unknown-linux-gnu
```

### 3. Game Scripting

```rust
// Load pre-compiled game scripts
let script = load_aot_module("enemy_ai.o")?;
script.execute()?;
```

### 4. Plugin System

```bash
# Compile plugins
$ vsp-aot compile plugin1.vsp --cache
$ vsp-aot compile plugin2.vsp --cache

# Load at runtime
```

## 📚 Ver Também

- [VSP_JIT_COMPLETE.md](VSP_JIT_COMPLETE.md) - JIT compilation
- [VSP_JIT_TECHNICAL.md](VSP_JIT_TECHNICAL.md) - Technical details
- [PERFORMANCE_SUMMARY.md](PERFORMANCE_SUMMARY.md) - Performance optimizations

---

**Autor**: SIL-Team  
**Versão**: 2026.1.0  
**Status**: ✅ Core implementado (opcodes WIP)
