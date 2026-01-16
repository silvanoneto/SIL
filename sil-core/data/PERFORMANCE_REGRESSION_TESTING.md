# 🔍 Performance Regression Testing

Sistema automatizado para detectar regressões de performance no código GPU.

## 🎯 Objetivo

Garantir que mudanças futuras não degradem a performance alcançada pelo sistema de batching GPU assíncrono.

## 📊 Métricas Monitoradas

### 1. **Throughput Mínimo**
- **Baseline**: 50K estados/segundo (mínimo aceitável)
- **Target**: 200K estados/segundo (M3 Pro)
- **Test**: `benchmark_minimum_throughput`

### 2. **Latência Máxima**
- **Baseline**: 5ms para 16 estados
- **Target**: <2ms para 16 estados
- **Test**: `benchmark_maximum_latency`

### 3. **Vantagem de Batching**
- **Baseline**: Batched deve ser >5x mais rápido que sequential
- **Target**: >10x mais rápido
- **Test**: `benchmark_batching_vs_sequential`

### 4. **Paridade Interpolação/Gradientes**
- **Baseline**: Interpolação deve ter ±20% da performance de gradientes
- **Target**: Performance similar
- **Test**: `benchmark_interpolate_parity`

### 5. **Overhead de Batching**
- **Baseline**: Diferentes batch sizes devem escalar linearmente
- **Target**: Overhead <10%
- **Test**: `benchmark_batching_overhead`

### 6. **Escalabilidade Paralela**
- **Baseline**: 2x tasks = ~1.8x throughput (90% eficiência)
- **Target**: >80% eficiência até 8 tasks
- **Test**: `benchmark_parallel_scalability`

## 🚀 Uso

### Rodar Testes de Regressão

```bash
# Rodar benchmarks
cargo bench --features gpu --bench gpu_regression

# Rodar script de validação
./scripts/check_performance_regression.sh

# Rodar com baseline comparison
cargo bench --features gpu --bench gpu_regression -- --baseline baseline
```

### Criar Baseline

```bash
# Primeira vez: criar baseline de referência
cargo bench --features gpu --bench gpu_regression -- --save-baseline baseline

# Comparar com baseline
cargo bench --features gpu --bench gpu_regression -- --baseline baseline
```

### CI/CD Integration

```yaml
# .github/workflows/performance.yml
name: Performance Regression

on: [pull_request]

jobs:
  regression:
    runs-on: macos-latest # ou runner com GPU
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Run regression tests
        run: |
          cargo bench --features gpu --bench gpu_regression -- --save-baseline pr
      
      - name: Check for regressions
        run: ./scripts/check_performance_regression.sh
```

## 📈 Interpretando Resultados

### Exemplo de Output Saudável

```
regression_minimum_throughput/gradient_1k_states
                        time:   [8.234 ms 8.567 ms 8.912 ms]
                        thrpt:  [114.9K elem/s 119.5K elem/s 124.3K elem/s]
                        
✅ Throughput: >100K elem/s (acima do mínimo)
```

### Exemplo de Regressão

```
regression_minimum_throughput/gradient_1k_states
                        time:   [25.12 ms 26.45 ms 27.89 ms]
                        thrpt:  [36.7K elem/s 38.7K elem/s 40.8K elem/s]
                        change: [+192% +209% +226%] (p = 0.00 < 0.05)
                        
❌ REGRESSÃO: Throughput caiu para 40K elem/s (abaixo de 50K mínimo)
```

## 🔧 Thresholds Configurados

```rust
// benches/gpu_regression.rs

// Throughput mínimo: 1024 estados em <20ms
// = 51,200 estados/segundo (baseline conservador)
group.throughput(Throughput::Elements(1024));
group.bench_function("gradient_1k_states", ...);

// Latência máxima: 16 estados em <5ms
// Com config otimizada: <2ms esperado
let config = BatchConfig {
    max_batch_size: 256,
    max_wait_ms: 1,  // Low latency
    channel_size: 64,
};

// Batching advantage: batched vs sequential
// Sequential: N × single_op_time
// Batched: batch_op_time (amortizado)
// Esperado: >5x speedup
```

## 📊 Benchmarks Detalhados

### 1. Individual Operations

**O que testa**: Overhead mínimo do sistema
**Métrica**: Tempo para 1 operação
**Threshold**: <5ms

```rust
benchmark_individual_operations
└── single_gradient: ~2-3ms (M3 Pro)
```

### 2. Batching vs Sequential

**O que testa**: Eficiência do batching
**Métrica**: Speedup (sequential / batched)
**Threshold**: >5x

```rust
benchmark_batching_vs_sequential
├── batched/16:     ~2ms  (16 estados)
├── sequential/16:  ~40ms (16 × 2.5ms)
└── speedup: 20x ✅
```

### 3. Minimum Throughput

**O que testa**: Performance absoluta
**Métrica**: Estados/segundo
**Threshold**: >50K/s

```rust
benchmark_minimum_throughput
└── gradient_1k_states: ~120K/s ✅
```

### 4. Maximum Latency

**O que testa**: Responsividade
**Métrica**: Tempo de resposta
**Threshold**: <5ms (16 estados)

```rust
benchmark_maximum_latency
└── latency_16_states: ~1.5ms ✅
```

### 5. Interpolate Parity

**O que testa**: Consistência entre operações
**Métrica**: Ratio (interpolate / gradient)
**Threshold**: 0.8 - 1.2 (±20%)

```rust
benchmark_interpolate_parity
├── gradient_256: ~8ms
├── lerp_256:     ~8.5ms (ratio: 1.06 ✅)
└── slerp_256:    ~9ms   (ratio: 1.12 ✅)
```

### 6. Batching Overhead

**O que testa**: Escalabilidade de batch size
**Métrica**: Tempo / num_estados (deve ser constante)
**Threshold**: <10% variação

```rust
benchmark_batching_overhead
├── batch_size=64:   15.6µs/estado
├── batch_size=256:  16.2µs/estado (+3.8% ✅)
├── batch_size=1024: 17.1µs/estado (+9.6% ✅)
└── batch_size=4096: 18.9µs/estado (+21% ⚠️)
```

### 7. Parallel Scalability

**O que testa**: Multi-threading efficiency
**Métrica**: Throughput linear com tasks
**Threshold**: >80% efficiency

```rust
benchmark_parallel_scalability
├── 1 task:  120K/s (baseline)
├── 2 tasks: 216K/s (90% efficiency ✅)
├── 4 tasks: 408K/s (85% efficiency ✅)
└── 8 tasks: 720K/s (75% efficiency ⚠️)
```

## 🐛 Troubleshooting

### Benchmark Falhou

```bash
# Verificar se GPU está disponível
cargo run --example gpu_batching --features gpu

# Rodar com mais detalhes
cargo bench --features gpu --bench gpu_regression -- --verbose

# Rodar apenas um benchmark específico
cargo bench --features gpu --bench gpu_regression -- minimum_throughput
```

### Performance Abaixo do Esperado

1. **Verificar GPU**: Certifique-se que está usando GPU dedicada
2. **Verificar temperatura**: Throttling térmico pode reduzir performance
3. **Verificar batch size**: Ajustar `max_batch_size` no config
4. **Verificar concorrência**: Outros processos usando GPU

### Resultados Inconsistentes

```bash
# Aumentar sample size
cargo bench --features gpu --bench gpu_regression -- --sample-size 100

# Rodar múltiplas vezes
for i in {1..5}; do
    cargo bench --features gpu --bench gpu_regression
done
```

## 📝 Checklist de Regressão

Antes de merge, verificar:

- [ ] Todos os benchmarks passam
- [ ] Throughput >50K estados/s
- [ ] Latência <5ms para 16 estados
- [ ] Batching >5x mais rápido que sequential
- [ ] Interpolação tem performance similar a gradientes
- [ ] Overhead de batching <10%
- [ ] Escalabilidade paralela >80% até 4 tasks
- [ ] Script `check_performance_regression.sh` passa

## 🎯 Próximos Passos

- [ ] Adicionar testes para NPU quando implementado
- [ ] Comparar com CPU fallback
- [ ] Testes de stress (>10K estados)
- [ ] Memory profiling (leaks, fragmentação)
- [ ] Power consumption benchmarks

## 📚 Ver Também

- [GPU_BATCHING.md](GPU_BATCHING.md) - Arquitetura do sistema
- [PERFORMANCE_SUMMARY.md](PERFORMANCE_SUMMARY.md) - Otimizações gerais
- [BENCHMARK_REPORT.md](BENCHMARK_REPORT.md) - Resultados completos

---

**Autor**: SIL-Team  
**Versão**: 2026.1.0  
**Status**: ✅ Ativo
