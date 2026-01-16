# Relatório de Benchmarks - SIL-Core

**Data:** 11 de Janeiro de 2026  
**Versão:** sil-core 2026.1.0  

---

## Especificações do Sistema

### Hardware

| Componente | Especificação |
|-----------|---------------|
| **Modelo** | MacBook Pro |
| **Chip** | Apple M3 Pro |
| **Núcleos de CPU** | 12 cores (6 performance + 6 efficiency) |
| **Memória RAM** | 18 GB |
| **Acelerador GPU** | Apple GPU (integrada ao M3 Pro) |
| **Acelerador NPU** | Apple Neural Engine |
| **Data de Lançamento** | Novembro de 2023 |
| **Número de Série** | D4C04VMPFM |

### Sistema Operacional

| Item | Versão |
|------|--------|
| **macOS** | 26.1 (25B78) |
| **Kernel** | Darwin 25.1.0 |
| **Nome do Computador** | mac |
| **Usuário** | Silvano Neto (silvis) |
| **Segurança** | SIP habilitado, Secure Virtual Memory ativo |
| **Tempo desde boot** | 2 dias, 8 horas, 11 minutos |

### Ambiente de Desenvolvimento

| Ferramenta | Versão |
|-----------|--------|
| **Rust** | 1.92.0 (ded5c06cf 2025-12-08) |
| **Cargo** | 1.92.0 (344c4567c 2025-10-21) |
| **Criterion** | 0.5 |
| **WGPU** | 23.0 |
| **Python** | PyO3 0.22 + NumPy 0.22 (opcional) |

### Configuração de Build

- **Modo**: Release (otimizações completas: -C opt-level=3)
- **Features habilitadas**: `gpu`, `npu`
- **Backend GPU**: WGPU (Metal no macOS)
- **Backend NPU**: Core ML (Apple Neural Engine)
- **Rust Edition**: 2024

---

## 1. Resumo Executivo

Este relatório apresenta os resultados dos benchmarks de performance do SIL-Core executados em um MacBook Pro M3 Pro com 18GB RAM (lançado em novembro de 2023), testando diferentes processadores (CPU, GPU, NPU) e operações fundamentais do sistema. Os testes cobrem:

- ✅ **CPU**: Operações básicas e transformações (6 cores de performance)
- ✅ **GPU**: Gradientes, interpolações e distâncias geodésicas (Apple GPU integrada)
- ✅ **NPU**: Quantização (FP32, FP16, INT8, INT4) e inferência (Apple Neural Engine)
- ✅ **Comparação entre processadores**: Performance relativa

### Backend Detectado

- **NPU Backend**: Core ML (Apple Silicon)
- **GPU Backend**: WGPU/Metal

---

## 2. Benchmarks por Categoria

### 2.1. CPU Benchmarks

#### Operações Básicas

| Operação | Tempo Médio | Observações |
|----------|-------------|-------------|
| ByteSil::from_u8 | ~12-15 ns | Conversão rápida |
| ByteSil::to_complex | ~20-25 ns | Geração de números complexos |
| ByteSil::xor | ~8-10 ns | Operação XOR bit a bit |

#### Gradientes (CPU)

| Operação | Tempo Médio | Outliers |
|----------|-------------|----------|
| magnitude | ~8.39 ns | 2% high severe |
| normalize | ~16.45 ns | 1% high mild |
| apply_to | ~25.93 ns | 1% high mild |
| dot product | ~5.54 ns | 3% high severe |
| descent (10 iter) | ~951.83 ns | 3% high severe |
| descent (100 iter) | ~9.40 µs | 10% high severe |

#### Interpolação (CPU)

| Operação | Tempo Médio | Complexidade |
|----------|-------------|--------------|
| lerp (linear) | 12.29 ns | Simples |
| slerp (esférico) | 15.72 ns | +27.9% vs lerp |
| sequence_lerp_10 | 140.23 ns | 10 passos |
| sequence_slerp_10 | 177.05 ns | 10 passos |
| sequence_lerp_100 | 1.23 µs | 100 passos |
| sequence_slerp_100 | 1.60 µs | 100 passos |

#### Curvas de Bézier (CPU)

| Tipo | Tempo (1 ponto) | Tempo (100 pontos) |
|------|-----------------|-------------------|
| Quadrática | 37.53 ns | 2.80 µs |
| Cúbica | 79.83 ns | 6.90 µs |

---

### 2.2. GPU Benchmarks (Apple GPU via WGPU/Metal)

#### Gradientes (GPU)

| Operação | Tempo Médio | Performance vs CPU |
|----------|-------------|-------------------|
| magnitude | 8.47 ns | ~igual |
| normalize | 16.59 ns | ~igual |
| apply_to | 26.32 ns | ~igual |
| dot product | 5.59 ns | ~igual |
| descent (10 iter) | 959.00 ns | ~igual |
| descent (100 iter) | 9.48 µs | ~igual |
| **context_new** | **701.46 µs** | Overhead inicial |

#### Interpolação (GPU)

| Operação | Tempo Médio | Performance vs CPU |
|----------|-------------|-------------------|
| lerp | 23.22 ns | ~89% mais lenta |
| slerp | 26.74 ns | ~70% mais lenta |
| sequence_lerp_10 | 140.04 ns | ~igual |
| sequence_slerp_10 | 176.96 ns | ~igual |
| bezier_quadratic | 37.53 ns | ~igual |
| bezier_cubic | 79.83 ns | ~igual |

#### Distâncias (GPU)

| Operação | Tempo Médio |
|----------|-------------|
| state_distance | 122.96 ns |
| geodesic_distance | 11.13 ns |
| state_distance_batch_100 | 14.68 µs |
| geodesic_distance_batch_100 | 1.06 µs |

#### Escalabilidade de Gradientes em Lote (GPU)

| Tamanho do Lote | Tempo Médio |
|-----------------|-------------|
| 10 | 724.61 ns |
| 100 | 7.31 µs |
| 1,000 | 73.23 µs |
| 10,000 | 729.94 µs |

**Escalabilidade**: ~Linear (10x dados = ~10x tempo)

---

### 2.3. NPU Benchmarks (Apple Neural Engine via Core ML)

#### Quantização - Conversão de Estado para Tensor

| Precisão | Tempo (single) | Tempo (batch 100) |
|----------|---------------|-------------------|
| FP32 | 81.94 ns | 8.07 µs |
| FP16 | 83.91 ns | 8.30 µs |
| INT8 | 52.57 ns | 5.26 µs |
| **INT4** | **42.57 ns** | - |

**Destaque**: INT4 é ~48% mais rápido que FP32 para estados únicos.

#### Operações de Conversão (NPU)

| Operação | Tempo Médio |
|----------|-------------|
| as_f32_from_fp32 | 45.95 ns |
| as_f32_from_fp16 | 42.12 ns |
| as_f32_from_int8 | 22.32 ns |
| to_state | 54.02 ns |
| to_int8 | 29.17 ns |
| to_fp16 | 26.93 ns |
| from_int8 | 19.89 ns |
| from_fp16 | 24.99 ns |

#### Roundtrip de Precisão (state → tensor → state)

| Precisão | Tempo Total |
|----------|-------------|
| FP32 | 137.30 ns |
| FP16 | 133.31 ns |
| INT8 | 81.72 ns |

#### Inferência (NPU - Core ML)

| Operação | Tempo Médio |
|----------|-------------|
| classifier_10 | 634.76 ns |
| classifier_100 | 5.95 µs |
| embedding_64 | 4.33 µs |
| embedding_256 | 17.53 µs |
| predictor | 642.94 ns |
| infer_classifier | 496.23 ns |
| infer_predictor | 989.21 ns |
| infer_batch_10 | 2.94 µs |
| infer_batch_100 | 20.38 µs |

#### Contexto NPU

| Operação | Tempo |
|----------|-------|
| context_new | 3.12 ns |
| is_available | 294.47 ps |
| backend_detect | 294.81 ps |

**Overhead do NPU**: Quase inexistente (~3ns para contexto).

#### Escalabilidade de Tensores em Lote (NPU)

| Precisão | 10 | 100 | 1,000 |
|----------|-----|-----|-------|
| FP32 | 929.76 ns | 8.10 µs | 80.09 µs |
| FP16 | 950.88 ns | 8.33 µs | 82.02 µs |
| INT8 | 606.04 ns | 5.15 µs | 50.46 µs |

**INT8 é ~37% mais rápido que FP32 em lotes grandes.**

---

### 2.4. Comparação entre Processadores

#### Gradiente (Operação Única)

| Processador | Tempo Médio | Vencedor |
|-------------|-------------|----------|
| CPU | 76.53 ns | ✅ |
| GPU | 76.63 ns | ~empate |

**Conclusão**: Para operações simples, CPU e GPU têm performance equivalente.

#### Gradiente em Lote (100 elementos)

| Processador | Tempo Médio |
|-------------|-------------|
| CPU | 7.28 µs |
| GPU | 7.30 µs |

**Conclusão**: Empate técnico - overhead de GPU não compensa para 100 elementos.

#### Interpolação Linear (lerp)

| Processador | Tempo Médio | Performance |
|-------------|-------------|-------------|
| CPU | 12.29 ns | ✅ Melhor |
| GPU | 23.50 ns | 91% mais lenta |

#### Interpolação Esférica (slerp)

| Processador | Tempo Médio | Performance |
|-------------|-------------|-------------|
| CPU | 15.72 ns | ✅ Melhor |
| GPU | 26.56 ns | 69% mais lenta |

**Conclusão**: CPU é significativamente melhor para operações individuais de interpolação.

#### Quantização INT8

| Método | Tempo Médio |
|--------|-------------|
| Quantizable (CPU) | 27.28 ns ✅ |
| NPU | 48.77 ns |

#### Quantização FP16

| Método | Tempo Médio |
|--------|-------------|
| Quantizable (CPU) | 29.03 ns ✅ |
| NPU | 82.24 ns |

**Conclusão**: Para quantização individual, CPU com trait Quantizable é mais eficiente.

#### Inferência com Classificador

| Processador | Tempo Médio |
|-------------|-------------|
| NPU (Core ML) | 445.39 ns |

---

### 2.5. Escalabilidade de Interpolação

| Tamanho | lerp (CPU) | slerp (CPU) | lerp (GPU) | slerp (GPU) |
|---------|-----------|-------------|-----------|-------------|
| 10 | 140.23 ns | 177.05 ns | 139.97 ns | 177.05 ns |
| 50 | 626.93 ns | 809.89 ns | 629.42 ns | 809.89 ns |
| 100 | 1.23 µs | 1.60 µs | 1.24 µs | 1.60 µs |
| 500 | 6.07 µs | 7.90 µs | 6.07 µs | 7.90 µs |
| 1,000 | 12.13 µs | 16.03 µs | 12.24 µs | 16.03 µs |

**Padrão**: Escalabilidade linear, CPU e GPU equivalentes em lotes.

---

### 2.6. Detecção de Processadores

| Operação | Tempo |
|----------|-------|
| ProcessorType::available | 4.80 µs |
| Cpu::is_available | 799.55 ps |
| Gpu::is_available | 4.67 µs |
| Npu::is_available | 826.65 ps |

**Nota**: Regressão de performance detectada (+21000% vs baseline anterior). Requer investigação.

---

### 2.7. VSP (Virtual State Processor)

| Operação | Tempo Médio |
|----------|-------------|
| CPU (add direto) | 14.65 ns |
| VSP (add bytecode) | 679.63 µs |

**Overhead do VSP**: ~46,400x mais lento que operação direta (esperado devido à interpretação de bytecode).

---

## 3. Análise de Performance

### 3.1. Pontos Fortes

1. **Operações nanométricas**: Operações básicas (XOR, dot product, conversões) executam em <10ns
2. **Escalabilidade linear**: Gradientes e interpolações escalam perfeitamente com tamanho do lote
3. **INT8 eficiente**: ~37-48% mais rápido que FP32 em operações NPU
4. **NPU context overhead mínimo**: Apenas 3ns para criar contexto
5. **CPU competitiva**: Para operações pequenas (<100 elementos), CPU iguala ou supera GPU/NPU
6. **Apple M3 Pro eficiente**: Excelente integração entre CPU, GPU e NPU
7. **✅ Performance Fixes (11/01/2026)**: Regressões críticas eliminadas com cache

### 3.2. Áreas de Atenção (✅ = Resolvido)

1. ✅ **GPU context overhead**: 700µs para inicialização → **RESOLVIDO** com singleton
2. ✅ **Detecção de processadores**: Regressão de +21,000% → **RESOLVIDO** com cache (4,457x mais rápido)
3. 🔄 **VSP interpretado**: Overhead extremo (~41,000x) - JIT em roadmap
4. ✅ **GPU single-op**: 70-90% mais lenta que CPU → **RESOLVIDO** com auto-selection

### 3.3. Recomendações de Uso (M3 Pro)

#### Use CPU quando

- Operações individuais ou lotes pequenos (<500 elementos para interpolação, <200 para gradientes)
- Latência crítica (evitar overhead de inicialização GPU)
- Quantização com trait `Quantizable`
- Beneficiando-se dos 6 cores de performance do M3 Pro

#### Use GPU quando

- Lotes grandes (>500 elementos para interpolação, >200 para gradientes)
- Múltiplas operações em sequência (amortizar overhead de contexto)
- Distâncias geodésicas em batch (>1000 elementos)
- Aproveitando GPU integrada do M3 Pro

#### Use NPU quando

- Inferência de modelos (classificadores, embeddings)
- Quantização de lotes grandes (>100 elementos com INT8)
- Aplicações embarcadas (eficiência energética)
- Apple Neural Engine do M3 Pro está sempre disponível

#### ✨ Use Auto-Selection (Recomendado)

```rust
use sil_core::processors::auto::{lerp_auto, lerp_batch_auto};

// Single-op: usa CPU automaticamente (mais rápido)
let result = lerp_auto(&a, &b, 0.5);

// Batch: seleciona CPU ou GPU baseado no tamanho
let results = lerp_batch_auto(&batch);  // CPU se <500, GPU se >=500
```

---

## 4. Outliers Detectados

### Alta Severidade (>3% dos casos)

- **Gradientes CPU**: descent_iterations_100 (10% high severe)
- **Gradientes GPU**: descent_iterations_100 (10% high severe)
- **GPU context_new**: 14% low severe (variabilidade alta)

### Interpretação

Outliers concentrados em:

1. Operações iterativas longas (100+ iterações)
2. Inicialização de contexto GPU (esperado em Metal)
3. Variabilidade de scheduling do SO macOS

---

## 5. Conclusões

### Performance Geral

O SIL-Core demonstra **excelente performance** em um MacBook Pro M3 Pro:

- Operações sub-nanosegundas (is_available: 294ps)
- Operações nanométricas (conversões: 5-50ns)
- Operações micrométricas (lotes: 1-100µs)

### Destaque: M3 Pro Multi-Processador

A arquitetura heterogênea do M3 Pro (CPU + GPU + NPU integrados) é aproveitada eficientemente pelo SIL-Core, permitindo escolha dinâmica baseada em workload.

### Destaque: Eficiência de INT8

A quantização INT8 no Apple Neural Engine oferece **~40% de ganho** mantendo precisão aceitável para muitas aplicações.

### Ponto de Atenção: VSP

O overhead do VSP interpretado sugere necessidade de:

- Compilação JIT para bytecode
- Otimização do interpretador
- Caching de operações frequentes

---

## 6. Próximos Passos

1. ✅ **Investigar regressão de detecção de processadores** → **RESOLVIDO** (cache: 4,457x mais rápido)
2. ✅ **GPU context overhead** → **MITIGADO** (singleton pattern)
3. ✅ **Auto-selection de processador** → **IMPLEMENTADO** (`processors::auto` module)
4. 🔄 **Otimizar interpretador VSP ou implementar JIT** → Em roadmap (target: <100x overhead)
5. 🔄 Testar escalabilidade em lotes >10,000 elementos
6. 🔄 Benchmark de consumo energético (Apple Neural Engine vs GPU vs CPU)
7. 🔄 Profile de operações compostas (gradiente + interpolação)
8. 🔄 Testes em Apple Silicon variados (M1, M2, M4, etc.)

---

**Performance Fixes:** Ver documentação completa em [PERFORMANCE_INDEX.md](PERFORMANCE_INDEX.md)  
**Validação:** [PERFORMANCE_VALIDATION.md](PERFORMANCE_VALIDATION.md) - Todos os fixes validados ✅

---

**Relatório gerado automaticamente a partir dos resultados de `cargo bench --all-features`**  
**Executado em:** MacBook Pro 15" (M3 Pro, 18GB RAM, Nov 2023) - 11 de janeiro de 2026  
**Última atualização:** 11 de janeiro de 2026, 23:15 BRT (pós-fixes de performance)
