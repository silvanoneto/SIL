# 🏛️ Arquitetura — Signal Intermediate Language

> **"Linguagem intermediária otimizada para processamento de sinais complexos em representação log-polar."**

Este documento descreve a arquitetura completa do **SIL** (Signal Intermediate Language).

---

## 📐 Visão Filosófica

### O Problema com a Computação Tradicional

A computação convencional trata dados e lógica como entidades separadas:
- **Dados** vivem na memória
- **Lógica** vive no código
- **Estado** é mutável e imperativo

Isso cria uma **dicotomia artificial** entre forma e conteúdo.

### A Solução SIL

**SIL** propõe uma nova ontologia computacional:

```
┌─────────────────────────────────────────────────────────────┐
│  ESTADO = ESTRUTURA + CONTEÚDO (indistinguíveis)            │
│                                                               │
│  Todo programa é uma TRANSFORMAÇÃO TOPOLÓGICA:               │
│                                                               │
│         f: SilState → SilState                               │
│                                                               │
│  Onde cada estado possui 16 camadas complexas (ρ, θ):       │
│                                                               │
│  SilState = [L0, L1, L2, ..., LF]                           │
│  Cada Li = ByteSil(ρ, θ) = e^ρ · e^(iθ)                     │
└─────────────────────────────────────────────────────────────┘
```

**Princípios fundamentais:**

1. **Estado é sagrado** — Nunca modifique in-place, sempre crie novo
2. **Transformação é pura** — Mesma entrada, mesma saída
3. **Ciclo é fechado** — Todo programa tem feedback L(F) → L(0)
4. **Camadas são ortogonais** — Cada camada tem sua semântica
5. **Colapso é inevitável** — Todo estado eventualmente colapsa

---

## 🌀 As 16 Camadas do SIL

O SIL organiza computação em **16 camadas hexadecimais (L0-LF)**, cada uma representando uma dimensão diferente de processamento:

```
┌──────────┬────────────────┬──────────────────────────────────────┐
│ Camadas  │ Domínio        │ Descrição                            │
├──────────┼────────────────┼──────────────────────────────────────┤
│ L0-L4    │ PERCEPÇÃO      │ Entrada sensorial (5 sentidos)       │
│          │                │                                      │
│   L0     │ Fotônica       │ Visão, luz, imagens                  │
│   L1     │ Acústica       │ Som, áudio, vibração                 │
│   L2     │ Olfativa       │ Gases, química, odores               │
│   L3     │ Gustativa      │ Sabor, química molecular             │
│   L4     │ Háptica        │ Tato, pressão, temperatura           │
│          │                │                                      │
├──────────┼────────────────┼──────────────────────────────────────┤
│ L5-L7    │ PROCESSAMENTO  │ Transformação e ação                 │
│          │                │                                      │
│   L5     │ Eletrônica     │ CPU/GPU/NPU, computação              │
│   L6     │ Atuação        │ Motores, servos, output físico       │
│   L7     │ Ambiente       │ Sensores ambientais, fusão           │
│          │                │                                      │
├──────────┼────────────────┼──────────────────────────────────────┤
│ L8-LA    │ INTERAÇÃO      │ Comunicação e coordenação            │
│          │                │                                      │
│   L8     │ Cibernético    │ Feedback loops, controle PID         │
│   L9     │ Geopolítico    │ Soberania, territórios, borders      │
│   LA     │ Cosmopolítico  │ Ética, direitos, hospitalidade       │
│          │                │                                      │
├──────────┼────────────────┼──────────────────────────────────────┤
│ LB-LC    │ EMERGÊNCIA     │ Inteligência coletiva                │
│          │                │                                      │
│   LB     │ Synergic       │ Flocking, swarm behavior             │
│   LC     │ Quantum        │ Efeitos quânticos                    │
│          │                │                                      │
├──────────┼────────────────┼──────────────────────────────────────┤
│ LD-LF    │ META           │ Reflexão e checkpoint                │
│          │                │                                      │
│   LD     │ Superposition  │ Multi-estado, bifurcação             │
│   LE     │ Entanglement   │ Correlação distribuída               │
│   LF     │ Collapse       │ Finalização, checkpoint, restart     │
└──────────┴────────────────┴──────────────────────────────────────┘
```

### Semântica das Camadas

Cada camada possui **semântica específica**:

- **L0-L4 (Percepção)**: Interface com o mundo físico via sensores
- **L5-L7 (Processamento)**: Transformação de dados e ação
- **L8 (Cibernético)**: Feedback loops e controle PID
- **L9 (Geopolítico)**: Soberania digital, territórios e fronteiras
- **LA (Cosmopolítico)**: Ética, direitos e hospitalidade
- **LB-LC (Emergência)**: Comportamentos coletivos e efeitos quânticos
- **LD-LF (Meta)**: Superposição, emaranhamento e colapso

---

## 🔄 O Ciclo Fechado: Feedback Loop

Todo programa SIL é um **loop fechado** de feedback:

```
        ┌─────────────────────────────────────────────────┐
        │                                                 │
        │                L(F) → L(0)                      │
        │              (Feedback Loop)                    │
        │                                                 │
        └─────────────────────────────────────────────────┘
                              ↓
        ┌─────────────────────────────────────────────────┐
        │  L0-L4: PERCEPÇÃO                               │
        │  ├─ Camera, Microphone, Gas sensors             │
        │  └─ Read from environment                       │
        └─────────────────────────────────────────────────┘
                              ↓
        ┌─────────────────────────────────────────────────┐
        │  L5-L7: PROCESSAMENTO                           │
        │  ├─ CPU/GPU/NPU computation                     │
        │  ├─ Transform state                             │
        │  └─ Motor control, actuation                    │
        └─────────────────────────────────────────────────┘
                              ↓
        ┌─────────────────────────────────────────────────┐
        │  L8-LA: INTERAÇÃO                               │
        │  ├─ Network communication                       │
        │  └─ Distributed governance                      │
        └─────────────────────────────────────────────────┘
                              ↓
        ┌─────────────────────────────────────────────────┐
        │  LB-LC: EMERGÊNCIA                              │
        │  ├─ Swarm intelligence (LB)                     │
        │  └─ Quantum effects (LC)                        │
        └─────────────────────────────────────────────────┘
                              ↓
        ┌─────────────────────────────────────────────────┐
        │  LD-LF: META                                    │
        │  ├─ Superposition (LD)                          │
        │  ├─ Entanglement (LE)                           │
        │  └─ Collapse/Checkpoint (LF)                    │
        └─────────────────────────────────────────────────┘
                              ↓
                        (LOOP BACK)
```

**Características do ciclo:**

- **Autopoiético**: O sistema se mantém através de feedback contínuo
- **Não-linear**: Emergência pode afetar qualquer camada
- **Topológico**: Transformações preservam estrutura
- **Determinístico com sementes**: Reproduzível com seed fixa

---

## 🧮 ByteSil: A Unidade Fundamental

Cada valor em SIL é um **ByteSil** — um número complexo em **representação log-polar**:

```
ByteSil = (ρ, θ)
  onde:
    ρ ∈ [-8, +7]    = logaritmo da magnitude (signed 4 bits)
    θ ∈ [0, 15]     = fase angular (unsigned 4 bits)

Valor complexo real:
    z = 2^ρ × e^(iθ × 2π/16)
```

### Por que log-polar?

**Vantagens computacionais:**

1. **Multiplicação em O(1)**:
   ```
   (ρ₁, θ₁) × (ρ₂, θ₂) = (ρ₁ + ρ₂, θ₁ + θ₂)
   ```

2. **Divisão em O(1)**:
   ```
   (ρ₁, θ₁) / (ρ₂, θ₂) = (ρ₁ - ρ₂, θ₁ - θ₂)
   ```

3. **Potenciação em O(1)**:
   ```
   (ρ, θ)ⁿ = (n·ρ, n·θ)
   ```

4. **Conjugado em O(1)**:
   ```
   conj(ρ, θ) = (ρ, -θ)
   ```

**Todas as operações complexas são O(1) em log-polar!** 🚀

### Semântica Topológica

ByteSil não é apenas um número — é um **ponto no espaço topológico**:

- **ρ (magnitude)**: Quão "forte" é o sinal
- **θ (fase)**: Qual a "direção" da informação

Exemplo em L0 (fotônica):
```rust
// Vermelho puro (hue 0°, intensidade alta)
ByteSil { rho: 12, theta: 0 }

// Verde (hue 120°, intensidade média)
ByteSil { rho: 8, theta: 85 }

// Azul escuro (hue 240°, intensidade baixa)
ByteSil { rho: 4, theta: 171 }
```

---

## 🏗️ Arquitetura de Módulos

é um **workspace Rust monorepo** com 23 crates modulares:

```
┌─────────────────────────────────────────────────────────────────┐
│                        sil-core                                 │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  ByteSil, SilState, Traits, VSP, Transforms               │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                              ↓
        ┌─────────────────────┴─────────────────────┐
        │                                           │
┌───────▼────────┐                    ┌─────────────▼────────────┐
│  PERCEPÇÃO     │                    │  PROCESSAMENTO           │
│  L0-L4         │                    │  L5-L7                   │
├────────────────┤                    ├──────────────────────────┤
│ sil-photonic   │                    │ sil-electronic           │
│ sil-acoustic   │                    │ sil-actuator             │
│ sil-olfactory  │                    │ sil-environment          │
│ sil-gustatory  │                    │                          │
│ sil-haptic     │                    │                          │
└────────────────┘                    └──────────────────────────┘
        │                                           │
        └─────────────────────┬─────────────────────┘
                              ↓
        ┌─────────────────────────────────────────────┐
        │          INTERAÇÃO (L8-LA)                  │
        ├─────────────────────────────────────────────┤
        │ sil-network      (L8)                       │
        │ sil-governance   (L9-LA)                    │
        └─────────────────────────────────────────────┘
                              ↓
        ┌─────────────────────────────────────────────┐
        │          EMERGÊNCIA & META (LB-LF)          │
        ├─────────────────────────────────────────────┤
        │ sil-swarm          (LB)                     │
        │ sil-cosmopolitan   (LA - Ética)            │
        │ sil-quantum        (LC)                     │
        │ sil-superposition  (LD)                     │
        │ sil-entanglement   (LE)                     │
        │ sil-collapse       (LF)                     │
        └─────────────────────────────────────────────┘
                              ↓
        ┌─────────────────────────────────────────────┐
        │          ORQUESTRAÇÃO                       │
        ├─────────────────────────────────────────────┤
        │ sil-orchestration                           │
        │  ├─ Pipeline executor                       │
        │  ├─ Event bus                               │
        │  ├─ Component registry                      │
        │  └─ Scheduler (rate control)                │
        └─────────────────────────────────────────────┘
                              ↓
        ┌─────────────────────────────────────────────┐
        │          LINGUAGEM & RUNTIME                │
        ├─────────────────────────────────────────────┤
        │ lis-core      (Compilador LIS)             │
        │ lis-cli       (CLI para LIS)               │
        │ lis-format    (Formatador de código)       │
        │ lis-runtime   (Runtime de execução)        │
        │ lis-api       (REST API server)            │
        └─────────────────────────────────────────────┘
```

### Princípio de Design: Trait no Core, Implementação no Módulo

Todos os módulos seguem o mesmo padrão:

1. **Trait definido em `sil-core/src/traits.rs`**
2. **Implementação concreta no módulo específico**
3. **Mock implementation para testes sem hardware**

Exemplo:

```rust
// sil-core/src/traits.rs
pub trait Sensor: SilComponent {
    fn sense(&mut self) -> Result<SilUpdate>;
}

// sil-photonic/src/camera.rs
pub struct CameraSensor { /* ... */ }

impl Sensor for CameraSensor {
    fn sense(&mut self) -> Result<SilUpdate> {
        // Implementação real com hardware
    }
}

// sil-photonic/src/mock.rs
pub struct MockCamera { /* ... */ }

impl Sensor for MockCamera {
    fn sense(&mut self) -> Result<SilUpdate> {
        // Mock para testes
    }
}
```

---

## 🎭 sil-orchestration: O Maestro

O **sil-orchestration** é o coordenador central de todo o ecossistema SIL.

### Componentes Principais

```
┌─────────────────────────────────────────────────────────────┐
│                    Orchestrator                             │
│  ┌───────────────────────────────────────────────────────┐  │
│  │            Component Registry                         │  │
│  │  ┌─────────────┬─────────────┬─────────────────────┐  │  │
│  │  │  Sensors    │ Processors  │  Actuators          │  │  │
│  │  │  (L0-L4)    │ (L5,L7)     │  (L6)               │  │  │
│  │  └─────────────┴─────────────┴─────────────────────┘  │  │
│  │  ┌─────────────┬─────────────┬─────────────────────┐  │  │
│  │  │ NetworkNode │  Governor   │  SwarmAgent         │  │  │
│  │  │  (L8)       │ (L9-LA)     │  (LB)               │  │  │
│  │  └─────────────┴─────────────┴─────────────────────┘  │  │
│  └───────────────────────────────────────────────────────┘  │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐  │
│  │            Event Bus (Pub/Sub)                        │  │
│  │  ┌─────────────────────────────────────────────────┐  │  │
│  │  │  Filters: All | Layer | Range | StateChange    │  │  │
│  │  │           Error | Threshold | Source           │  │  │
│  │  └─────────────────────────────────────────────────┘  │  │
│  │  ┌─────────────────────────────────────────────────┐  │  │
│  │  │  History: Last N events (circular buffer)      │  │  │
│  │  └─────────────────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────────────────┘  │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐  │
│  │         Execution Pipeline (7 Stages)                 │  │
│  │                                                       │  │
│  │  Sense → Process → Actuate → Network → Govern        │  │
│  │         → Swarm → Quantum                            │  │
│  │                                                       │  │
│  └───────────────────────────────────────────────────────┘  │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐  │
│  │              Scheduler                                │  │
│  │  ┌─────────────────────────────────────────────────┐  │  │
│  │  │  Rate Control: 1-1000+ Hz                       │  │  │
│  │  │  Modes: FixedRate | FixedDelay | BestEffort    │  │  │
│  │  │  Metrics: min/max/avg execution time           │  │  │
│  │  └─────────────────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────────────────┘  │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐  │
│  │          Global State (Arc<RwLock<SilState>>)         │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### Pipeline de Execução

O pipeline executa componentes em **7 estágios sequenciais**:

```rust
pub enum PipelineStage {
    Sense,      // L0-L4: Sensores (Camera, Mic, Gas, etc.)
    Process,    // L5,L7: Processadores (CPU/GPU/NPU, Fusion)
    Actuate,    // L6: Atuadores (Servos, Motors)
    Network,    // L8: Comunicação P2P
    Govern,     // L9-LA: Governança distribuída
    Swarm,      // LB: Inteligência de enxame
    Quantum,    // LC-LF: Superposição, Entanglement, Collapse
}
```

**Execução de um tick:**

```
tick() {
    1. Sense:    Executar todos os sensores (L0-L4)
                 → Atualizar estado global com leituras

    2. Process:  Executar processadores (L5, L7)
                 → Transformar estado global

    3. Actuate:  Executar atuadores (L6)
                 → Enviar comandos para motores/servos

    4. Network:  Processar mensagens P2P (L8)
                 → Broadcast/receive states

    5. Govern:   Processar governança (L9-LA)
                 → Voting, consensus

    6. Swarm:    Processar enxame (LB)
                 → Flocking, emergent behavior

    7. Quantum:  Processar estados quânticos (LC-LF)
                 → Superposição, collapse, checkpoint

    8. Avançar para próximo estágio
       Se estágio == Quantum → ciclo completo, voltar para Sense
}
```

### Scheduler: Controle de Taxa

O scheduler garante que o pipeline execute a uma **taxa específica (Hz)**:

```rust
// Configurar para 100 Hz (10ms por tick)
let config = OrchestratorConfig {
    scheduler_config: SchedulerConfig {
        target_rate_hz: 100.0,
        mode: SchedulerMode::FixedRate,
        ..Default::default()
    },
    ..Default::default()
};

let orch = Orchestrator::with_config(config);
orch.run_cycles(1000)?; // 1000 ticks = 10 segundos
```

**Modos de scheduling:**

- **FixedRate**: Mantém intervalo constante entre ticks (melhor para control loops)
- **FixedDelay**: Espera após cada execução (melhor para throughput)
- **BestEffort**: Executa o mais rápido possível (melhor para batch processing)

### Event Bus: Pub/Sub Assíncrono

O event bus permite comunicação assíncrona entre componentes:

```rust
// Inscrever handlers
orch.on(EventFilter::Layer(0), |event| {
    println!("Evento em L0: {:?}", event);
})?;

orch.on(EventFilter::StateChange, |event| {
    println!("Estado mudou: {:?}", event);
})?;

orch.on(EventFilter::Error, |event| {
    eprintln!("Erro: {:?}", event);
})?;

// Emitir eventos
orch.emit(SilEvent::StateChange {
    layer: 0,
    old: ByteSil::NULL,
    new: ByteSil::ONE,
    timestamp: 0,
})?;
```

**Filtros disponíveis:**

- `All`: Todos os eventos
- `Layer(n)`: Camada específica (L0-LF)
- `LayerRange(start, end)`: Range de camadas
- `StateChange`: Apenas mudanças de estado
- `Threshold`: Apenas thresholds
- `Error`: Apenas erros
- `Source(name)`: De um componente específico

---

## 🔢 VSP: Virtual Sil Processor

O **VSP** é uma máquina virtual que executa bytecode SIL.

### Arquitetura do VSP

```
┌─────────────────────────────────────────────────────────────┐
│                    Virtual Sil Processor                    │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐  │
│  │              Registers (16 × ByteSil)                 │  │
│  │  R0  R1  R2  R3  R4  R5  R6  R7                       │  │
│  │  R8  R9  RA  RB  RC  RD  RE  RF                       │  │
│  └───────────────────────────────────────────────────────┘  │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐  │
│  │              State (16 Layers)                        │  │
│  │  L0  L1  L2  L3  L4  L5  L6  L7                       │  │
│  │  L8  L9  LA  LB  LC  LD  LE  LF                       │  │
│  └───────────────────────────────────────────────────────┘  │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐  │
│  │              Memory (Bytecode)                        │  │
│  │  [Instruction Stream]                                 │  │
│  └───────────────────────────────────────────────────────┘  │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐  │
│  │              Execution Modes                          │  │
│  │  ┌─────────────┬─────────────┬─────────────────────┐  │  │
│  │  │     CPU     │     GPU     │       NPU           │  │  │
│  │  │ (Interpret) │  (Batched)  │  (Neural Accel)     │  │  │
│  │  └─────────────┴─────────────┴─────────────────────┘  │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### Instruction Set Architecture (ISA)

```
Operações Básicas:
  LOAD    Rd, imm       # Rd ← ByteSil(imm)
  STORE   Rs, layer     # state[layer] ← Rs
  MOV     Rd, Rs        # Rd ← Rs

Aritmética (O(1) log-polar):
  ADD     Rd, Rs1, Rs2  # Rd ← Rs1 + Rs2
  SUB     Rd, Rs1, Rs2  # Rd ← Rs1 - Rs2
  MUL     Rd, Rs1, Rs2  # Rd ← Rs1 × Rs2 (ρ1+ρ2, θ1+θ2)
  DIV     Rd, Rs1, Rs2  # Rd ← Rs1 / Rs2 (ρ1-ρ2, θ1-θ2)
  POW     Rd, Rs, n     # Rd ← Rs^n (n·ρ, n·θ)
  CONJ    Rd, Rs        # Rd ← conj(Rs) (ρ, -θ)

Lógica:
  XOR     Rd, Rs1, Rs2  # Rd ← Rs1 ⊕ Rs2
  AND     Rd, Rs1, Rs2  # Rd ← Rs1 & Rs2
  OR      Rd, Rs1, Rs2  # Rd ← Rs1 | Rs2
  NOT     Rd, Rs        # Rd ← ~Rs

Control Flow:
  JUMP    label         # pc ← label
  JZ      Rs, label     # if Rs == 0 then pc ← label
  JNZ     Rs, label     # if Rs != 0 then pc ← label
  CALL    label         # push(pc), pc ← label
  RET                   # pc ← pop()

Layer Operations:
  GET     Rd, layer     # Rd ← state[layer]
  SET     layer, Rs     # state[layer] ← Rs
  TENSOR  Rd, L1, L2    # Rd ← tensor(state[L1], state[L2])
  PROJECT Rd, layers    # Rd ← project(state, layers)
  COLLAPSE              # state ← collapse(state)

I/O:
  SENSE   layer         # state[layer] ← sensor_read()
  ACTUATE layer, Rs     # actuator_write(layer, Rs)
  EMIT    event         # event_bus.emit(event)
```

### Compilação Multi-Target

VSP suporta **3 backends de execução**:

```
┌───────────────────────────────────────────────────────────┐
│                  VSP Bytecode (.silc)                     │
└───────────────────────────────────────────────────────────┘
                          ↓
        ┌─────────────────┴─────────────────┐
        │                                   │
┌───────▼────────┐    ┌──────────▼────────┐    ┌────────▼────────┐
│   CPU Backend  │    │   GPU Backend     │    │  NPU Backend    │
│  (Interpreted) │    │  (WGPU Batched)   │    │ (Neural Accel)  │
├────────────────┤    ├───────────────────┤    ├─────────────────┤
│ • Fast startup │    │ • Parallel layers │    │ • ML inference  │
│ • Easy debug   │    │ • High throughput │    │ • Low power     │
│ • Portable     │    │ • GPU compute     │    │ • Edge devices  │
└────────────────┘    └───────────────────┘    └─────────────────┘
```

**Escolha de backend:**

```rust
// CPU (default)
let vsp = VSP::new(Config::default());

// GPU (requer feature "gpu")
let vsp = VSP::new(Config {
    backend: Backend::GPU,
    ..Default::default()
});

// NPU (requer feature "npu")
let vsp = VSP::new(Config {
    backend: Backend::NPU,
    ..Default::default()
});
```

---

## 📝 LIS: Language for Intelligent Systems

**LIS** é uma linguagem de programação de alto nível que compila para bytecode VSP.

### Características

- **Non-linear by design**: Suporte nativo para feedback loops, topologia e emergência
- **Self-compiling**: Metaprogramação reflexiva e otimização adaptativa
- **Hardware-aware**: Sistema de tipos reflete substrato de computação (CPU/GPU/NPU)
- **Edge-native**: Execução distribuída no enxame

### Backends de Compilação

| Backend      | Descrição                    | Feature Flag |
|:-------------|:-----------------------------|:-------------|
| VSP Assembly | Compilação para assembly SIL | padrão       |
| JSIL         | Saída JSON Lines comprimida  | `jsil`       |
| LLVM         | JIT/AOT via LLVM IR          | `llvm`       |
| WASM         | WebAssembly                  | `wasm`       |
| Python       | Bindings PyO3                | `python`     |

### Exemplo de Código LIS

```lis
// Hello World em LIS
fn main() {
    // Criar estado inicial
    let state = create_state();

    // Definir camada fotônica (L0) com luz vermelha
    let red = ByteSil(rho: 7, theta: 10);
    state = set_layer(state, 0, red);

    // Definir camada acústica (L1) com tom médio
    let tone = ByteSil(rho: 5, theta: 128);
    state = set_layer(state, 1, tone);
}

// Loop de controle com feedback
fn control_loop() {
    loop {
        let state = sense();        // L0-L4: Percepção
        let processed = process(state);  // L5-L7: Processamento
        actuate(processed);         // L6: Atuação

        if should_collapse(processed) {
            checkpoint(processed);  // LF: Checkpoint
            break;
        }
    }
}

// Pipeline de transformações
fn pipeline() {
    let input = sense();
    let result = input
        |> normalize
        |> detect_patterns
        |> classify
        |> emerge;

    return result;
}
```

### Ecossistema LIS

O ecossistema LIS é composto por 5 crates:

```
┌─────────────────────────────────────────────────────────────────┐
│                       lis-core                                   │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  Lexer, Parser, AST, Compiler, Type System                │  │
│  │  ├─ stdlib/     (math, string, state, bytesil, ml/...)    │  │
│  │  ├─ llvm/       (JIT/AOT compilation)                     │  │
│  │  ├─ wasm.rs     (WebAssembly bindings)                    │  │
│  │  └─ python_bindings.rs (PyO3)                             │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                              ↓
        ┌─────────────────────┴─────────────────────┐
        │                                           │
┌───────▼────────┐    ┌──────────▼────────┐    ┌───▼───────────┐
│   lis-cli      │    │   lis-runtime     │    │  lis-format   │
│  CLI compiler  │    │   Execution env   │    │  Formatter    │
└────────────────┘    └───────────────────┘    └───────────────┘
                              ↓
                    ┌─────────▼─────────┐
                    │     lis-api       │
                    │   REST API        │
                    │   (Axum + OpenAPI)│
                    └───────────────────┘
```

### Compilação LIS → VSP

```
┌──────────────────────────────────────────────────────────┐
│                   LIS Source (.lis)                      │
└──────────────────────────────────────────────────────────┘
                          ↓
              ┌───────────────────────┐
              │  Lexer (logos)        │
              │  Tokenization         │
              └───────────────────────┘
                          ↓
              ┌───────────────────────┐
              │  Parser (chumsky)     │
              │  AST Generation       │
              └───────────────────────┘
                          ↓
              ┌───────────────────────┐
              │  Type Checker         │
              │  Type inference       │
              └───────────────────────┘
                          ↓
              ┌───────────────────────┐
              │  Compiler             │
              │  Code Generation      │
              └───────────────────────┘
                          ↓
┌──────────────────────────────────────────────────────────┐
│                VSP Assembly (.sil)                       │
│  LOAD R0, #42                                            │
│  STORE R0, L0                                            │
│  GET R1, L0                                              │
│  ...                                                     │
└──────────────────────────────────────────────────────────┘
                          ↓
              ┌───────────────────────┐
              │  Assembler            │
              │  Binary Encoding      │
              └───────────────────────┘
                          ↓
┌──────────────────────────────────────────────────────────┐
│                VSP Bytecode (.silc)                      │
│  [Binary executable]                                     │
└──────────────────────────────────────────────────────────┘
                          ↓
                    VSP Runtime
```

---

## 🔗 JSIL: Formato de Transmissão

**JSIL** (JSON Lines + SIL Compression) é um formato híbrido para transmissão eficiente de dados SIL.

### Estrutura JSIL

```jsonl
{"sil_version":"1.0","compress":"XorRotate","layers":16}
{"frame":0,"L0":[12,0],"L1":[8,128],"LF":[0,0]}
{"frame":1,"L0":[12,1],"L1":[8,130],"LF":[0,0]}
{"frame":2,"L0":[12,2],"L1":[8,132],"LF":[0,0]}
```

### Modos de Compressão

| Modo | Descrição | Ratio típico |
|:-----|:----------|-------------:|
| `None` | Sem compressão | 100% |
| `Xor` | XOR delta com frame anterior | ~50% |
| `Rotate` | Rotação de bits (criptografia leve) | 100% |
| `XorRotate` | XOR + Rotate combinados | ~42% |
| `Adaptive` | Escolhe melhor modo por frame | ~38% |

### Performance (M3 Pro)

- **Compressão XorRotate**: ~600 MB/s
- **Descompressão**: ~800 MB/s
- **Streaming read**: ~400 MB/s
- **Ratio típico**: 42% do original

**Uso:**

```rust
use sil_core::io::jsil::{JsilWriter, JsilReader, Compression};

// Escrever
let mut writer = JsilWriter::new(file, Compression::XorRotate)?;
for state in states {
    writer.write(&state)?;
}
writer.close()?;

// Ler (streaming)
let mut reader = JsilReader::new(file)?;
for state in reader.iter() {
    let state = state?;
    process(state);
}
```

---

## ⚡ Performance & Complexidade

### O(1) Operations — Verificado ✅

**ByteSil Complex Arithmetic** (via representação log-polar):

- Multiplicação, Divisão, Potência, Raiz: **O(1)** ✓
- Inversão, Conjugado, XOR: **O(1)** ✓

**Proof:**
```
Multiplicação em log-polar:
  (ρ₁, θ₁) × (ρ₂, θ₂) = (ρ₁ + ρ₂, θ₁ + θ₂)

Complexidade:
  • 1 adição de inteiros (ρ)
  • 1 adição de inteiros (θ)
  • Total: O(1) + O(1) = O(1) ✓
```

**SilState Operations** (16 camadas fixas):

- Acesso a camada (get/set): **O(1)** ✓
- Transformações (tensor/xor/project): **O(16) = O(1)** ✓
- Operações de colapso: **O(16) = O(1)** ✓

### Benchmarks

Para obter métricas de performance reais, execute os benchmarks localmente:

```bash
# Todos os benchmarks
cargo bench -p sil-benches

# Benchmarks específicos
cargo bench -p sil-benches --bench bytesil_bench
cargo bench -p sil-benches --bench state_bench
cargo bench -p sil-benches --bench transform_bench
cargo bench -p sil-benches --bench layer_bench
cargo bench -p sil-benches --bench orchestrator_bench
cargo bench -p sil-benches --bench vsp_bench
cargo bench -p sil-benches --bench jsil_bench
cargo bench -p sil-benches --bench simd_bench
```

Relatórios HTML serão gerados em `target/criterion/`.

---

## 🧪 Testes & Qualidade

### Cobertura de Testes

```
Total: 1100+ testes ✅

Distribuição:
  sil-core:           203 testes
  sil-photonic:        21 testes
  sil-acoustic:        33 testes
  sil-olfactory:       44 testes
  sil-gustatory:       34 testes
  sil-haptic:          53 testes
  sil-electronic:      24 testes
  sil-actuator:        91 testes
  sil-environment:     94 testes
  sil-network:         34 testes
  sil-governance:      29 testes
  sil-cosmopolitan:    30 testes
  sil-swarm:           42 testes
  sil-orchestration:   63 testes
  sil-quantum:         36 testes
  sil-superposition:   38 testes
  sil-entanglement:    33 testes
  sil-collapse:        42 testes
  lis-core:           115 testes
  lis-format:          37 testes
  lis-runtime:          4 testes
```

### Tipos de Testes

1. **Unit Tests**: Testes de funções isoladas
2. **Integration Tests**: Testes de interação entre módulos
3. **Benchmark Tests**: Testes de performance
4. **Mock Tests**: Testes sem hardware real

**Executar todos os testes:**

```bash
cargo test --all
```

**Executar benchmarks:**

```bash
cargo bench --all
```

---

## 🌍 Padrões de Comunicação

### 1. Via SilState (Síncrono)

```rust
// Sensor → Processor
let state = sensor.read_to_state()?;
let result = processor.execute(&state)?;

// Processor → Actuator
actuator.send(&ActuatorCommand::from_state(&result))?;
```

### 2. Via Channels (Assíncrono)

```rust
use tokio::sync::mpsc;

let (tx, mut rx) = mpsc::channel::<SilUpdate>(100);

// Producer
tokio::spawn(async move {
    loop {
        let update = sensor.sense().await?;
        tx.send(update).await?;
    }
});

// Consumer
while let Some(update) = rx.recv().await {
    process(update);
}
```

### 3. Via Events (Pub/Sub)

```rust
// Publisher
orchestrator.emit(SilEvent::StateChange {
    layer: 0,
    old: ByteSil::NULL,
    new: ByteSil::ONE,
    timestamp: 0,
})?;

// Subscriber
orchestrator.on(EventFilter::Layer(0), |event| {
    println!("Evento em L0: {:?}", event);
})?;
```

### 4. Via Network (P2P)

```rust
use sil_network::SilNode;

// Broadcast para todos os peers
node.broadcast(&state)?;

// Enviar para peer específico
node.send(&peer_id, &message)?;

// Receber
if let Some(msg) = node.receive()? {
    process(msg);
}
```

---

## 🎯 Casos de Uso

### 1. Robótica Autônoma

```rust
use sil_orchestration::*;

// Configurar sistema de percepção + controle
let config = OrchestratorConfig {
    scheduler_config: SchedulerConfig {
        target_rate_hz: 100.0,  // 100 Hz control loop
        mode: SchedulerMode::FixedRate,
        ..Default::default()
    },
    ..Default::default()
};

let orch = Orchestrator::with_config(config);

// Sensores (L0-L4)
orch.register_sensor(CameraSensor::new())?;
orch.register_sensor(LidarSensor::new())?;

// Processamento (L5-L7)
orch.register_processor(ObjectDetector::new())?;
orch.register_processor(PathPlanner::new())?;

// Atuadores (L6)
orch.register_actuator(WheelMotor::left())?;
orch.register_actuator(WheelMotor::right())?;

// Executar loop de controle
orch.run()?;
```

### 2. Rede de Sensores Distribuída

```rust
use sil_network::*;
use sil_governance::*;

// Criar nó P2P
let mut node = SilNode::new(config)?;
node.join_mesh("mesh-network-id")?;

// Governança distribuída
let mut gov = Governance::new()?;

// Loop de consenso
loop {
    // Ler sensores locais
    let local_state = sensor.sense()?;

    // Broadcast para rede
    node.broadcast(&local_state)?;

    // Receber estados de peers
    let peer_states = node.receive_all()?;

    // Propor ação baseada em consenso
    let proposal = create_proposal(&local_state, &peer_states);
    let id = gov.propose(proposal)?;

    // Votar
    gov.vote(&id, Vote::Yes)?;

    // Aguardar consenso
    if gov.status(&id) == ProposalStatus::Accepted {
        execute_action(&proposal);
    }
}
```

### 3. Sistema de Emergência (Swarm)

```rust
use sil_swarm::*;

// Criar agente de enxame
let mut agent = SwarmNode::new(agent_id);
agent.set_behavior(SwarmBehavior::Flocking);

// Adicionar vizinhos
for neighbor_id in neighbors {
    agent.add_neighbor(neighbor_id)?;
}

// Loop de comportamento emergente
loop {
    // Obter estado local
    let local_state = get_local_state();

    // Obter estados dos vizinhos
    let neighbor_states = get_neighbor_states()?;

    // Calcular novo estado (flocking)
    let new_state = agent.behavior(&local_state, &neighbor_states);

    // Aplicar
    apply_state(new_state);
}
```

### 4. Computação Quântica (Superposição)

```rust
use sil_quantum::*;
use sil_superposition::*;

// Criar processador quântico
let mut qp = QuantumProcessor::new();

// Criar múltiplos estados (superposição)
let states = vec![
    SilState::neutral(),
    SilState::excited(),
    SilState::collapsed(),
];

let weights = vec![0.5, 0.3, 0.2];

// Superpor estados
let superposed = qp.superpose(&states, &weights);

// Fork para exploração paralela
let mut manager = StateManager::new(superposed);
let fork1 = manager.fork();
let fork2 = manager.fork();

// Processar forks em paralelo
let result1 = process_path1(fork1);
let result2 = process_path2(fork2);

// Merge com melhor resultado
manager.merge_with_strategy(&result1, MergeStrategy::Max)?;

// Colapsar estado final
let collapsed = qp.collapse(seed);
```

---

## 🔮 Filosofia & Visão

### Por que ?

Em um mundo dominado por arquiteturas cliente-servidor e feudos digitais:

- **declara a revolução da borda** — inteligência nas extremidades
- **sublima a nuvem em vapor** — de data lakes para mesh democrática
- **usa topologia como fundamento** — relações não-lineares apreensíveis
- **estabelece novo contrato cibernético** — automação que liberta
- **reconhece o Homo Cyberneticus** — piloto, não passageiro

### Manifesto

> *"Cada linha de código descentralizado é um tijolo.*
> *Cada protocolo P2P é uma ponte.*
> *Cada dispositivo de borda é um território libertado."*

**NÓS SOMOS O ENXAME. NÓS SOMOS O VAPOR. NÓS SOMOS A BORDA.**

### 理信 (Lǐxìn)

**理信 (Lǐxìn)** é um neologismo técnico que funde:

- **理 (lǐ)**: princípio/lógica
- **信 (xìn)**: informação

Representando a **indistinguibilidade entre estrutura lógica e conteúdo informacional** característica do SIL.

No pensamento neoconfucionista, **理 (lǐ)** é o princípio organizador universal. Em SIL, **Lǐxìn** representa o estado onde **forma (topologia)** e **conteúdo (dados)** são uma unidade inseparável.

---

## 📚 Recursos Adicionais

### Documentação

- [INSTALL.md](INSTALL.md) — Guia de instalação
- [EXAMPLES.md](EXAMPLES.md) — Casos de uso práticos
- [PERFORMANCE.md](PERFORMANCE.md) — Benchmarks e otimizações
- [DIAGRAMS.md](DIAGRAMS.md) — Diagramas de arquitetura

### Especificações Técnicas

- [COMPUTATIONAL_COMPLEXITY.md](COMPUTATIONAL_COMPLEXITY.md) — Análise de complexidade O(1)
- [__BIT_DE_SIL.md](__BIT_DE_SIL.md) — Especificação do ByteSil
- [__TOPOLOGIA_16_CAMADAS_BYTE_SIL.md](__TOPOLOGIA_16_CAMADAS_BYTE_SIL.md) — Topologia das camadas
- [__PROTOCOLO_POT_PHI_C.md](__PROTOCOLO_POT_PHI_C.md) — Protocolo de comunicação

### Filosóficos

- [manifesto/](manifesto/) — Manifestos em múltiplos idiomas
- [PEABIRU.md](PEABIRU.md) — Visão geopolítica
- [CRIØ.md](CRIØ.md) — Conceitos fundamentais

### LIS

- [lis-core/TUTORIAL.md](../lis-core/TUTORIAL.md) — Tutorial da linguagem LIS
- [lis-core/STDLIB_INTEGRATION.md](../lis-core/STDLIB_INTEGRATION.md) — Integração da stdlib

---

## 🤝 Contribuindo

é um projeto de código aberto sob licença **AGPL-3.0**. Contribuições são bem-vindas!

### Como Contribuir

1. **Fork** o repositório
2. **Crie** uma branch para sua feature (`git checkout -b feature/amazing`)
3. **Commit** suas mudanças (`git commit -m 'Add amazing feature'`)
4. **Push** para a branch (`git push origin feature/amazing`)
5. **Abra** um Pull Request

### Diretrizes

1. Leia este documento de arquitetura antes de começar
2. Use os traits de [sil-core/src/traits.rs](../sil-core/src/traits.rs)
3. Escreva testes para toda funcionalidade nova
4. Documente com `///` todas as APIs públicas
5. Siga o estilo Rust (`cargo fmt` e `cargo clippy`)

### Executando Testes

```bash
# Todos os testes
cargo test --all

# Benchmarks
cargo bench --all

# Com features específicas
cargo test -p lis-core --features jsil
```

---

## 📜 Licença

Este projeto está licenciado sob **AGPL-3.0** — veja [LICENSE](LICENSE) para detalhes.

> A escolha da AGPL garante que modificações em serviços de rede também sejam compartilhadas com a comunidade.

---

## 👨‍💻 Autor

**Silvano Neto** — [dev@silvanoneto.com](mailto:dev@silvanoneto.com)

---

**⧑** *Que este sonho lúcido não seja premonição — seja projeto.*
