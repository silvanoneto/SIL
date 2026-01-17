# ML Distribuído com Paebiru/LIS

A arquitetura SIL/LIS oferece vantagens estruturais únicas para distribuição de modelos. Vou explicar os mecanismos-chave:

## 1. Representação Compacta: ByteSil

A unidade fundamental é **1 byte por peso** (vs 32 bits em float32):

```
┌─────────────────────────────────────────────────────────────┐
│  Modelo tradicional          │  Modelo em ByteSil          │
├─────────────────────────────────────────────────────────────┤
│  1M parâmetros × 4 bytes     │  1M parâmetros × 1 byte     │
│  = 4 MB por transmissão      │  = 1 MB por transmissão     │
│                              │  + compressão JSIL (~42%)   │
│                              │  ≈ 420 KB efetivos          │
└─────────────────────────────────────────────────────────────┘
```

**Impacto:** Modelos ~10× menores para transmitir, armazenar e sincronizar.

---

## 2. Métrica ρ_sil: Roteamento Inteligente

O sistema decide automaticamente **onde processar** baseado na complexidade:

```lis
use paebiru::edge::rho_sil::{rho_sil, should_offload};
use paebiru::edge::router::{route_compute, ROUTE_LOCAL, ROUTE_NEAR, ROUTE_FAR};

fn distributed_inference(input: State, model: State) -> State {
    let complexity = rho_sil(input);
    
    // Roteamento automático por zona cromática
    if complexity < 0.1 {
        // UltraLocal: processa no próprio dispositivo
        return local_forward(input, model);
    }
    
    if complexity < 0.3 {
        // Local: edge node próximo
        return edge_forward(input, model);
    }
    
    if complexity < 0.5 {
        // Near: distribui entre peers
        return mesh_forward(input, model);
    }
    
    // Far/HPC: offload para cluster
    return cloud_forward(input, model);
}
```

**Impacto:** Cada inferência vai para o hardware mais adequado, não o mais disponível.

---

## 3. Federated Learning Nativo

A biblioteca Paebiru traz primitivas de primeira classe:

```lis
use paebiru::distributed::fedavg::{fedavg_aggregate};
use paebiru::distributed::privacy::{dp_config_create, add_gaussian_noise};
use paebiru::distributed::byzantine::{krum_aggregate};

// Agregação de N clientes com privacidade diferencial
fn secure_federated_round(
    client_models: [State; 4],
    epsilon: Float,
    delta: Float
) -> State {
    // 1. Detectar clientes maliciosos (Byzantine-robust)
    let filtered = krum_aggregate(client_models, 1);  // tolera 1 bizantino
    
    // 2. Agregar modelos válidos
    let aggregated = fedavg_aggregate(filtered);
    
    // 3. Adicionar ruído para privacidade diferencial
    let dp = dp_config_create(epsilon, delta, 1.0);
    let sigma = gaussian_noise_scale(epsilon, delta, 1.0);
    
    return add_gaussian_noise(aggregated, sigma, 42);
}
```

**Impacto:** O modelo **nunca sai do dispositivo** — apenas gradientes/atualizações anonimizados viajam.

---

## 4. Sharding por Camadas Semânticas

As 16 camadas têm significado — isso permite **distribuição inteligente**:

```
┌─────────────────────────────────────────────────────────────┐
│  MODELO DISTRIBUÍDO POR CAMADAS SEMÂNTICAS                  │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  L0-L4 (Percepção)     → Processado no SENSOR               │
│  ├─ Câmera processa L0 (Fotônico)                          │
│  ├─ Microfone processa L1 (Acústico)                       │
│  └─ Dados já chegam pré-processados                        │
│                                                             │
│  L5-L7 (Processamento) → Processado no EDGE NODE           │
│  ├─ Fusão multimodal                                       │
│  └─ Inferência principal                                   │
│                                                             │
│  L8-LA (Interação)     → Processado na MESH                │
│  ├─ Consenso entre peers                                   │
│  └─ Governança do cluster                                  │
│                                                             │
│  LB-LC (Emergência)    → Emerge do ENXAME                  │
│  └─ Comportamento coletivo                                 │
│                                                             │
│  LD-LF (Meta)          → Controle de PROTOCOLO             │
│  └─ Fork, sync, checkpoint                                 │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

```lis
use paebiru::lis::{lis_config_create, lis_enable_modality};

// Configurar nó para processar apenas suas modalidades
fn configure_edge_node(node_type: Int) -> State {
    let config = lis_config_create(2026, 1);
    
    if node_type == 0 {
        // Nó de percepção visual
        return lis_enable_modality(config, 0);  // L0 apenas
    }
    
    if node_type == 1 {
        // Nó de processamento
        let c = lis_enable_modality(config, 5);
        let c = lis_enable_modality(c, 6);
        return lis_enable_modality(c, 7);  // L5-L7
    }
    
    // Nó completo
    return lis_config_full();
}
```

---

## 5. Operações O(1) para Agregação

A representação log-polar torna agregação trivial:

```lis
// Média de modelos = média de logs
fn model_average(models: [State; N]) -> State {
    // Em log-polar: média(ρ) = média aritmética
    // Equivale a média geométrica das magnitudes!
    
    let sum = state_vacuum();
    let i = 0;
    loop {
        if i >= N { break; }
        let sum = state_add(sum, models[i]);
        let i = i + 1;
    }
    
    // Divide por N (escala ρ)
    return transform_magnitude_scale(sum, 1.0 / N);
}
```

**Matemática:** `average(e^ρ₁, e^ρ₂, ...) = e^(average(ρ₁, ρ₂, ...))` — soma de logs!

---

## 6. Checkpoints e Sincronização Eficientes

O formato JSIL permite streaming de estados:

```
┌─────────────────────────────────────────────────────────────┐
│  SINCRONIZAÇÃO INCREMENTAL                                  │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Frame 0: Estado completo (16 bytes + header)              │
│  Frame 1: XOR delta (apenas diferenças)                    │
│  Frame 2: XOR delta                                        │
│  ...                                                        │
│                                                             │
│  Se modelo mudou 5% → transmite ~5% dos bytes              │
│  Compressão XorRotate → ~42% adicional                     │
│                                                             │
│  Resultado: updates de 1MB modelo ≈ 20KB transmitidos      │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## 7. Arquitetura Mesh para Treinamento

```lis
use paebiru::distributed::mesh::{mesh_node_create, mesh_broadcast, mesh_receive};
use paebiru::distributed::cluster::{cluster_join, cluster_role};

fn distributed_training_loop(local_data: State, epochs: Int) {
    // Criar nó na mesh
    let node = mesh_node_create();
    let cluster = cluster_join(node, "training-cluster");
    
    let model = state_neutral();  // Inicialização
    let epoch = 0;
    
    loop {
        if epoch >= epochs { break; }
        
        // 1. Treinar localmente
        let gradients = compute_gradients(model, local_data);
        let model = sgd_step(model, gradients, 0.01);
        
        // 2. Sincronizar com peers (a cada N steps)
        if epoch % 10 == 0 {
            // Broadcast modelo local
            mesh_broadcast(node, model);
            
            // Receber modelos de peers
            let peer_models = mesh_receive(node);
            
            // Agregar
            let model = fedavg_aggregate(peer_models);
        }
        
        let epoch = epoch + 1;
    }
    
    return model;
}
```

---

## Resumo: Por Que LIS/Paebiru Distribui Melhor

| Aspecto | Tradicional | LIS/Paebiru |
|:--------|:------------|:------------|
| **Tamanho do peso** | 32 bits (float32) | 8 bits (ByteSil) |
| **Transmissão** | Modelo completo | Delta XOR comprimido |
| **Decisão de onde processar** | Manual/fixo | Automático (ρ_sil) |
| **Privacidade** | Add-on (CKKS, etc) | Nativa (DP first-class) |
| **Tolerância a falhas** | Framework específico | Byzantine-robust builtin |
| **Agregação** | O(n) multiplicações | O(n) somas (log-polar) |
| **Semântica** | Tensores opacos | 16 camadas com significado |

**A filosofia central:** O modelo não é um blob monolítico que vai para a nuvem — é um **estado distribuído** que vive nas bordas, sincroniza por diferenças, e processa onde faz sentido.
