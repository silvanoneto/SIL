# 📚 SIL-Core Documentation

Documentação técnica do projeto SIL-Core — Performance optimization e JIT compilation.

## 📑 Índice

### Performance Optimization
- **[PERFORMANCE_INDEX.md](PERFORMANCE_INDEX.md)** - Navegação principal para documentos de performance
- **[PERFORMANCE_SUMMARY.md](PERFORMANCE_SUMMARY.md)** - Resumo executivo das otimizações (4/4 fixes)
- **[PERFORMANCE_INVESTIGATION.md](PERFORMANCE_INVESTIGATION.md)** - Investigação detalhada dos 4 problemas críticos
- **[PERFORMANCE_VALIDATION.md](PERFORMANCE_VALIDATION.md)** - Resultados dos benchmarks e validação
- **[PERFORMANCE_COMPLETED.md](PERFORMANCE_COMPLETED.md)** - Relatório final de conclusão
- **[PERFORMANCE_README.md](PERFORMANCE_README.md)** - Guia do desenvolvedor para performance
- **[QUICKSTART_PERFORMANCE.md](QUICKSTART_PERFORMANCE.md)** - Início rápido com exemplos
- **[BENCHMARK_REPORT.md](BENCHMARK_REPORT.md)** - Relatório completo de benchmarks

### VSP JIT Compilation
- **[VSP_JIT_COMPLETE.md](VSP_JIT_COMPLETE.md)** - Relatório de implementação completa ⭐
- **[VSP_JIT_QUICKREF.md](VSP_JIT_QUICKREF.md)** - Referência rápida para desenvolvedores ⚡
- **[VSP_JIT_TECHNICAL.md](VSP_JIT_TECHNICAL.md)** - Documentação técnica detalhada
- **[VSP_JIT_STATUS.md](VSP_JIT_STATUS.md)** - Status atual do projeto
- **[VSP_JIT_PROPOSAL.md](VSP_JIT_PROPOSAL.md)** - Proposta original e roadmap

### GPU & Shaders
- **[GPU_CONTEXT.md](GPU_CONTEXT.md)** - Inicialização wgpu, backends, device selection
- **[GPU_BATCHING.md](GPU_BATCHING.md)** - Operações assíncronas em lote, auto-batching ⭐
- **[SHADER_PRECOMPILATION.md](SHADER_PRECOMPILATION.md)** - Build-time WGSL validation

## 🎯 Quick Links

### Para Começar
1. 📖 Leia [PERFORMANCE_SUMMARY.md](PERFORMANCE_SUMMARY.md) para entender as otimizações
2. 🚀 Use [QUICKSTART_PERFORMANCE.md](QUICKSTART_PERFORMANCE.md) para exemplos práticos
3. 🔥 Veja [VSP_JIT_QUICKREF.md](VSP_JIT_QUICKREF.md) para usar o JIT compiler

### Para Desenvolvedores
- 🔍 [PERFORMANCE_INVESTIGATION.md](PERFORMANCE_INVESTIGATION.md) - Root cause analysis
- ✅ [PERFORMANCE_VALIDATION.md](PERFORMANCE_VALIDATION.md) - Como validar fixes
- 🏗️ [VSP_JIT_TECHNICAL.md](VSP_JIT_TECHNICAL.md) - Arquitetura do JIT

### Resultados
- **GPU Context Cache**: 4,457x speedup (4.67µs → 1.05ns)
- **Processor Detection**: 217x speedup (4.80µs → 22ns)
- **Auto-Selection**: 1.9x improvement para small batches
- **JIT Compilation**: 10x target (<60µs vs 587µs) ⚠️ Requer x86_64

## 📊 Status do Projeto

### Concluído ✅
- ✅ Investigação de 4 performance regressions
- ✅ Implementação de fixes com caching
- ✅ Validação via benchmarks
- ✅ Auto-selection APIs
- ✅ JIT compiler PoC completo

### Em Progresso 🔄
- 🔄 Testes em arquitetura x86_64 (JIT)

### Próximos Passos 📋
- Expandir opcodes do JIT
- Suporte ARM64 para JIT
- Otimizações Tier 2

## 🏗️ Estrutura do Código

```
sil-core/
├── src/
│   ├── processors/
│   │   ├── performance_fixes.rs  - Caching & singleton patterns
│   │   └── auto.rs               - Auto-selection heuristics
│   └── vsp/
│       └── jit/                  - JIT compiler (Cranelift)
│           ├── mod.rs
│           ├── compiler.rs
│           └── runtime.rs
├── benches/
│   └── vsp_jit.rs                - JIT benchmarks
├── examples/
│   ├── auto_selection.rs         - Demo auto-selection
│   └── vsp_jit_poc.rs            - JIT PoC
└── docs/                         - Esta pasta
```

## 🔧 Build & Test

```bash
# Build com todas features
cargo build --all-features

# Build com JIT
cargo build --features jit

# Run benchmarks
cargo bench

# Run exemplo JIT (requer x86_64)
cargo run --features jit --example vsp_jit_poc
```

## 📞 Contato

Para questões sobre performance ou JIT, consulte os documentos técnicos ou abra uma issue no repositório.

---

**Última atualização**: Janeiro 2026
**Versão SIL-Core**: 2026.1.0
