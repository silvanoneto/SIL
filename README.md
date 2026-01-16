# SIL - Signal Intermediate Language

> **Linguagem de programação para sistemas não-lineares e IA edge-first**

[![Version](https://img.shields.io/badge/version-2026.1.16-blue.svg)](https://github.com/silvanoneto/SIL)
[![License](https://img.shields.io/badge/license-AGPL--3.0-green.svg)](LICENSE)

---

## Visão Geral

**SIL** (Signal Intermediate Language) é uma linguagem intermediária otimizada para processamento de sinais complexos usando representação log-polar. **LIS** (Language for Intelligent Systems) é a linguagem de alto nível que compila para SIL.

### Características Principais

| Feature | Descrição |
|---------|-----------|
| **ByteSil** | Unidade computacional de 8 bits representando números complexos em coordenadas log-polar |
| **State** | Container de 16 camadas (L0-LF) de ByteSils para computação multimodal |
| **Edge-native** | Otimizado para dispositivos de baixo consumo e computação distribuída |
| **Privacy-centric** | Suporte nativo a privacidade diferencial e aprendizado federado |

---

## Estrutura do Projeto

```
SIL/
├── lis-core/           # Compilador LIS (Rust)
│   ├── src/
│   │   ├── lexer.rs    # Tokenização
│   │   ├── parser.rs   # Parser
│   │   ├── compiler.rs # Geração de código SIL
│   │   └── types/      # Sistema de tipos
│   └── Cargo.toml
│
├── lis-cli/            # CLI do compilador
│   └── src/main.rs
│
├── sil-core/           # VM SIL (Rust)
│   ├── src/
│   │   ├── vm.rs       # Máquina virtual
│   │   ├── bytesil.rs  # Tipo ByteSil
│   │   └── state.rs    # Tipo State (16 camadas)
│   └── Cargo.toml
│
├── paebiru/            # Biblioteca ML em LIS
│   ├── lis.toml        # Manifest do projeto
│   └── src/            # 46 arquivos .lis
│       ├── lib.lis     # Entry point
│       ├── core/       # Operações fundamentais
│       ├── layers/     # Camadas de redes neurais
│       ├── arch/       # Arquiteturas (Transformer, KAN, SSM, etc.)
│       ├── distributed/# Treinamento federado
│       ├── edge/       # Computação edge
│       └── ...
│
├── sil-vscode/         # Extensão VSCode
│   └── src/extension.ts
│
└── docs/               # Documentação
    └── LIS_SIL_DOCUMENTATION.md
```

---

## Paebiru - Biblioteca ML

**Paebiru** é uma biblioteca completa de Machine Learning escrita em LIS, projetada para computação edge-first e privacy-centric.

### Módulos

| Módulo | Descrição | Arquivos |
|--------|-----------|----------|
| **core** | ByteSil, State, ativações, loss, otimizadores | 7 |
| **layers** | Dense, Norm, Dropout, Residual | 5 |
| **arch** | Transformer, KAN, SSM/Mamba, LNN, RNN, SNN | 9 |
| **distributed** | FedAvg, Byzantine, Cluster, Mesh, Privacy | 7 |
| **edge** | Device detection, ρ_sil routing | 4 |
| **inference** | Backend selection, auto-routing | 1 |
| **memory** | Cache, pools, tensor storage | 1 |
| **emergence** | Hebbian, Kuramoto, DFT | 1 |
| **observability** | Metrics, tracing | 1 |
| **security** | Audit, permissions | 1 |
| **protocols** | gRPC, REST, MQTT, WebSocket | 1 |
| **hardware** | GPU, TPU, NPU detection | 1 |
| **streaming** | Transforms, windowing | 1 |
| **storage** | Checkpoints, sharding, DHT | 1 |
| **mlops** | Model registry, A/B testing | 1 |
| **scaling** | Resource limits, load balancer | 1 |
| **lis** | 16 modalities (L0-LF) | 1 |

### Arquiteturas Suportadas

```lis
// Transformer
use arch::transformer::{transformer_encoder_block, transformer_decoder_block};

// Kolmogorov-Arnold Networks (KAN)
use arch::kan::{kan_forward, bspline_basis_1};

// State Space Models (Mamba)
use arch::ssm::{ssm_state_update, mamba_selective_scan};

// Liquid Neural Networks
use arch::lnn::{ltc_step, ctrnn_step};

// Recurrent Networks
use arch::recurrent::{rnn_cell, lstm_cell, gru_cell};

// Spiking Neural Networks
use arch::snn::{lif_step, generate_spikes};
```

### Exemplo: Forward Pass

```lis
use paebiru::layers::dense::{dense_forward, dense_relu};
use paebiru::core::activations::{softmax};

fn mlp_forward(input: State, w1: State, b1: State, w2: State, b2: State) -> State {
    let h1 = dense_relu(input, w1, b1);
    let out = dense_forward(h1, w2, b2);
    return softmax(out);
}
```

### Exemplo: Federated Learning

```lis
use paebiru::distributed::fedavg::{fedavg_4};
use paebiru::distributed::privacy::{dp_config_create, add_gaussian_noise};

fn federated_round(m0: State, m1: State, m2: State, m3: State) -> State {
    // Aggregate client models
    let aggregated = fedavg_4(m0, m1, m2, m3);

    // Add differential privacy noise
    let dp_config = dp_config_create(1.0, 0.00001, 1.0);
    let sigma = gaussian_noise_scale(1.0, 0.00001, 1.0);

    return add_gaussian_noise(aggregated, sigma, 42);
}
```

---

## ByteSil - Representação Log-Polar

O `ByteSil` é a unidade fundamental de computação em SIL:

```
┌───────────────────┐
│    ByteSil (8b)   │
├─────────┬─────────┤
│ rho (4b)│theta(4b)│
│ -8 a +7 │ 0 a 15  │
│   mag   │  phase  │
└─────────┴─────────┘
```

- **rho**: Logaritmo da magnitude (signed 4-bit: -8 a +7)
- **theta**: Fase normalizada (unsigned 4-bit: 0-15 → 0 a 2π)

### Operações

```lis
// Criar ByteSil
let b = bytesil_new(8, 4);  // rho=8, theta=4

// Magnitude e fase
let mag = bytesil_magnitude(b);
let phase = bytesil_phase_radians(b);

// Operações complexas
let product = bytesil_mul(a, b);
let quotient = bytesil_div(a, b);
let conjugate = bytesil_conjugate(b);
```

---

## State - Container de 16 Camadas

O `State` organiza dados em 16 camadas semânticas:

```
┌─────────────────────────────────────┐
│              State                   │
├─────┬─────┬─────┬─────┬─────┬─────┤
│ L0  │ L1  │ L2  │ L3  │ L4  │ ... │
│ 8b  │ 8b  │ 8b  │ 8b  │ 8b  │     │
└─────┴─────┴─────┴─────┴─────┴─────┘
       │           │           │
       ▼           ▼           ▼
   Percepção   Processo    Emergência
   (L0-L4)     (L5-L7)     (LB-LF)
```

| Camadas | Função |
|---------|--------|
| L0-L4 | Percepção (input sensorial) |
| L5-L7 | Processamento (computação) |
| L8-LA | Interação (comunicação) |
| LB-LC | Emergência (comportamento coletivo) |
| LD-LF | Meta (reflexão/checkpoint) |

### Operações

```lis
// Criar State
let s = state_vacuum();   // Todas camadas zeradas
let s = state_neutral();  // Valores neutros

// Acessar camadas
let layer0 = state_get_layer(s, 0);
let s = state_set_layer(s, 0, new_value);

// Operações entre States
let combined = state_xor(s1, s2);
let summed = state_add(s1, s2);
let product = state_tensor(s1, s2);
```

---

## LIS Modalities (L0-LF)

O modelo LIS define 16 modalidades para sistemas inteligentes:

| Código | Modalidade | Descrição |
|--------|------------|-----------|
| L0 | Photonic | Visual |
| L1 | Acoustic | Audio |
| L2 | Olfactory | Smell |
| L3 | Gustatory | Taste |
| L4 | Dermic | Touch |
| L5 | Electronic | Signals |
| L6 | Psychomotor | Movement |
| L7 | Environmental | Context |
| L8 | Cybernetic | Digital systems |
| L9 | Geopolitical | Social context |
| LA | Cosmopolitical | Global context |
| LB | Synergic | Combined effects |
| LC | Quantum | Quantum effects |
| LD | Superposition | Multiple states |
| LE | Entanglement | Correlated states |
| LF | Collapse | State resolution |

```lis
use paebiru::lis::{lis_config_create, lis_enable_modality, MOD_PHOTONIC, MOD_ACOUSTIC};

// Criar configuração com modalidades específicas
let config = lis_config_create(2026, 1);
let config = lis_enable_modality(config, MOD_PHOTONIC());
let config = lis_enable_modality(config, MOD_ACOUSTIC());

// Ou usar configurações pré-definidas
let minimal = lis_config_minimal();   // Apenas Electronic
let standard = lis_config_standard(); // Perception + Processing
let full = lis_config_full();         // Todas as 13 modalidades
```

---

## Instalação

### Requisitos

- Rust 1.75+
- Cargo

### Build

```bash
# Clonar repositório
git clone https://github.com/silvanoneto/SIL.git
cd SIL

# Build todos os componentes
cargo build --release

# Instalar CLI
cargo install --path lis-cli
```

### VSCode Extension

```bash
cd sil-vscode
npm install
npm run compile
# Instalar via VSIX ou carregar em modo de desenvolvimento
```

---

## Uso

### Compilar arquivo LIS

```bash
# Compilar para SIL assembly
lis compile programa.lis -o programa.sil

# Compilar projeto
cd meu-projeto
lis build
```

### Criar novo projeto

```bash
lis new meu-projeto
cd meu-projeto

# Estrutura criada:
# meu-projeto/
# ├── lis.toml
# └── src/
#     └── main.lis
```

### Exemplo lis.toml

```toml
[package]
name = "meu-projeto"
version = "2026.1.16"
authors = ["Nome <email@example.com>"]
description = "Meu projeto LIS"
license = "MIT"

[dependencies]
paebiru = { path = "../paebiru" }

[build]
entry = "src/main.lis"
target_mode = "SIL-128"
```

---

## Documentação

- [LIS & SIL Documentation](docs/LIS_SIL_DOCUMENTATION.md) - Documentação completa da linguagem
- [Paebiru Library](paebiru/src/lib.lis) - Entry point da biblioteca ML

---

## ρ_sil - Métrica de Complexidade Edge

O sistema usa a métrica ρ_sil para decisões de roteamento:

```lis
use paebiru::edge::rho_sil::{rho_sil, should_offload};
use paebiru::edge::router::{route_simple, ROUTE_LOCAL, ROUTE_OFFLOAD_NEAR};

fn process_with_routing(input: State) -> State {
    let rho = rho_sil(input);

    if should_offload(rho, 0.5) {
        // Offload para hardware mais potente
        return offload_inference(input);
    }

    // Processar localmente
    return local_inference(input);
}
```

### Zonas Cromáticas

| Zona | ρ_sil | Ação |
|------|-------|------|
| UltraLocal | < 0.1 | Processar no dispositivo |
| Local | 0.1-0.3 | Processar em edge node |
| Near | 0.3-0.5 | Distribuir entre nodes |
| Far | 0.5-0.8 | Offload para cloud |
| HPC | > 0.8 | Requer datacenter |

---

## Contribuindo

1. Fork o repositório
2. Crie uma branch: `git checkout -b feature/nova-feature`
3. Commit suas mudanças: `git commit -m 'Add nova feature'`
4. Push para a branch: `git push origin feature/nova-feature`
5. Abra um Pull Request

---

## Licença

Este projeto está licenciado sob a licença AGPL-3.0 - veja o arquivo [LICENSE](LICENSE) para detalhes.

---

## Autor

**Silvano Neto** - [dev@silvanoneto.com](mailto:dev@silvanoneto.com)

---

*SIL/LIS - Language for Intelligent Systems - 2026*
