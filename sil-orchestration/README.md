# 🎭 sil-orchestration — Orquestração Central do Ecossistema SIL

[![Tests](https://img.shields.io/badge/tests-41%20passing-brightgreen)]()
[![Rust](https://img.shields.io/badge/rust-1.92%2B-orange)]()
[![License](https://img.shields.io/badge/license-AGPL--3.0-blue)]()

**Coordenador central do ecossistema SIL** que gerencia componentes de todas as 16 camadas, eventos, pipeline de execução e comunicação entre módulos.

## 🎯 Visão Geral

O **sil-orchestration** é o maestro do ecossistema SIL. Ele:

- 🎼 **Coordena execução** de componentes de todas as camadas (L0-LF)
- 📡 **Sistema de eventos** pub/sub com filtros avançados
- ⚡ **Pipeline de execução** com 7 estágios (Sense → Process → Actuate → Network → Govern → Swarm → Quantum)
- ⏱️ **Scheduler** com controle de taxa (Hz) e rate limiting
- 📊 **Métricas e estatísticas** em tempo real
- 🔍 **Registro de componentes** com busca por tipo/camada

## 📦 Instalação

Adicione ao `Cargo.toml`:

```toml
[dependencies]
sil-orchestration = { path = "../sil-orchestration" }
```

## 🚀 Quick Start

### Exemplo Básico

```rust
use sil_orchestration::prelude::*;

// Criar orquestrador
let mut orch = Orchestrator::new();

// Registrar componentes
let sensor_id = orch.register_sensor(my_camera)?;
let proc_id = orch.register_processor(my_processor)?;
let act_id = orch.register_actuator(my_motor)?;

// Executar pipeline
orch.start()?;

// Executar 100 ticks
for _ in 0..100 {
    orch.tick()?;
}

orch.stop()?;
```

### Exemplo com Scheduler (Taxa Controlada)

```rust
use sil_orchestration::*;

// Configurar taxa de 60 Hz (60 ticks/segundo)
let config = OrchestratorConfig {
    scheduler_config: SchedulerConfig {
        target_rate_hz: 60.0,
        mode: SchedulerMode::FixedRate,
        ..Default::default()
    },
    debug: true,
    ..Default::default()
};

let orch = Orchestrator::with_config(config);

// Registrar componentes...

// Executar 100 ciclos completos a 60 Hz
orch.run_cycles(100)?;
```

### Exemplo com Eventos

```rust
use sil_orchestration::*;

let orch = Orchestrator::new();

// Inscrever handlers de eventos
orch.on(EventFilter::Layer(0), |event| {
    println!("Evento na camada L0: {:?}", event);
})?;

orch.on(EventFilter::StateChange, |event| {
    println!("Mudança de estado: {:?}", event);
})?;

orch.on(EventFilter::Error, |event| {
    eprintln!("Erro detectado: {:?}", event);
})?;

// Emitir eventos manualmente
orch.emit(SilEvent::StateChange {
    layer: 0,
    old: ByteSil::NULL,
    new: ByteSil::ONE,
    timestamp: 0,
})?;
```

## 🏗️ Arquitetura

```
┌─────────────────────────────────────────────────────────────┐
│                    Orchestrator                             │
│  ┌───────────────────────────────────────────────────────┐  │
│  │            Component Registry                         │  │
│  │  Sensors | Processors | Actuators | NetworkNodes     │  │
│  │  Governors | SwarmAgents | Quantum | Meta            │  │
│  └───────────────────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────────────────┐  │
│  │            Event Bus                                  │  │
│  │  StateChange | Threshold | Error | Custom            │  │
│  └───────────────────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────────────────┐  │
│  │            Execution Pipeline                         │  │
│  │  Sense → Process → Actuate → Network → Govern        │  │
│  │  → Swarm → Quantum (7 estágios)                      │  │
│  └───────────────────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────────────────┐  │
│  │            Scheduler                                  │  │
│  │  Rate Control (Hz) | Fixed Rate/Delay | Best Effort  │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## 📋 Componentes Principais

### 1. Orchestrator

Coordenador central que amarra todos os subsistemas.

**API Principal:**

```rust
// Ciclo de vida
fn new() -> Self
fn start(&self) -> Result<()>
fn stop(&self) -> Result<()>
fn is_running(&self) -> Result<bool>

// Execução
fn tick(&self) -> Result<()>           // Executa 1 tick
fn run(&self) -> Result<()>            // Loop infinito (bloqueante)
fn run_cycles(&self, n: u64) -> Result<()>  // N ciclos

// Registro de componentes
fn register_sensor<S>(&self, s: S) -> Result<ComponentId>
fn register_processor<P>(&self, p: P) -> Result<ComponentId>
fn register_actuator<A>(&self, a: A) -> Result<ComponentId>
fn unregister(&self, id: &ComponentId) -> Result<()>

// Estado
fn state(&self) -> Result<SilState>
fn update_state(&self, state: SilState) -> Result<()>

// Eventos
fn emit(&self, event: SilEvent) -> Result<()>
fn on<F>(&self, filter: EventFilter, handler: F) -> Result<()>
fn event_history(&self) -> Result<Vec<SilEvent>>

// Métricas
fn stats(&self) -> Result<OrchestratorStats>
fn uptime(&self) -> Duration
```

### 2. Pipeline

Executor de estágios sequenciais do ciclo SIL.

**Estágios:**

| Estágio | Camadas | Descrição |
|:--------|:-------:|:----------|
| `Sense` | L0-L4 | Sensoriamento (Photonic, Acoustic, Olfactory, Gustatory, Haptic) |
| `Process` | L5, L7 | Processamento (Electronic, Environment) |
| `Actuate` | L6 | Atuação (Motors, Servos) |
| `Network` | L8 | Comunicação P2P |
| `Govern` | L9-LA | Governança distribuída |
| `Swarm` | LB | Inteligência de enxame |
| `Quantum` | LC-LF | Superposição, Entanglement, Collapse |

**API:**

```rust
let mut pipeline = Pipeline::new();

pipeline.start();
pipeline.next_stage();
pipeline.current_stage(); // Some(PipelineStage::Sense)
pipeline.cycles();        // Número de ciclos completos
pipeline.reset();
```

### 3. Scheduler

Controle de taxa de execução com precisão de Hz.

**Modos:**

- **FixedRate** — Mantém intervalo constante entre ticks
- **FixedDelay** — Espera após cada execução
- **BestEffort** — Executa o mais rápido possível

**API:**

```rust
let mut scheduler = Scheduler::with_rate_hz(100.0); // 100 Hz

loop {
    let tick_info = scheduler.wait_for_next_tick()?;

    let start = Instant::now();
    // ... executar trabalho ...
    scheduler.record_execution_time(start.elapsed());

    if scheduler.tick_count() % 1000 == 0 {
        let stats = scheduler.stats();
        println!("Rate: {:.1} Hz, Avg: {:?}",
                 stats.actual_rate_hz,
                 stats.avg_execution_time);
    }
}
```

### 4. EventBus

Sistema pub/sub para comunicação assíncrona entre componentes.

**Filtros:**

```rust
EventFilter::All                    // Todos os eventos
EventFilter::Layer(0)               // Camada específica
EventFilter::LayerRange(0, 4)       // Range de camadas (L0-L4)
EventFilter::StateChange            // Apenas mudanças de estado
EventFilter::Threshold              // Apenas thresholds
EventFilter::Error                  // Apenas erros
EventFilter::Source("camera".into()) // De um componente específico
```

**Exemplo:**

```rust
let bus = EventBus::with_history(1000);

// Inscrever handler
bus.subscribe(EventFilter::Layer(0), |event| {
    match event {
        SilEvent::StateChange { layer, old, new, .. } => {
            println!("L{}: {:?} → {:?}", layer, old, new);
        }
        _ => {}
    }
})?;

// Emitir evento
bus.emit(SilEvent::Ready { component: "sensor-0".into() })?;

// Histórico
let history = bus.history()?;
println!("Total events: {}", history.len());
```

### 5. ComponentRegistry

Registro de componentes com índices por tipo e camada.

**API:**

```rust
let mut registry = ComponentRegistry::new();

// Registrar
let id = registry.register(my_sensor, ComponentType::Sensor)?;

// Buscar
let wrapper = registry.get(&id).unwrap();
println!("Component: {} (layers: {:?})", wrapper.name, wrapper.layers);

// Listar por tipo
let sensors = registry.list_by_type(ComponentType::Sensor);

// Listar por camada
let layer0 = registry.list_by_layer(0);

// Remover
registry.unregister(&id)?;
```

## 📊 Configuração

```rust
pub struct OrchestratorConfig {
    /// Pipeline habilitado
    pub enable_pipeline: bool,

    /// Estágios do pipeline
    pub pipeline_stages: Vec<PipelineStage>,

    /// Sistema de eventos habilitado
    pub enable_events: bool,

    /// Tamanho do histórico de eventos
    pub event_history_size: usize,

    /// Timeout para execução de componentes (ms)
    pub component_timeout_ms: u64,

    /// Configuração do scheduler
    pub scheduler_config: SchedulerConfig,

    /// Modo debug (logs detalhados)
    pub debug: bool,
}
```

**Exemplo de configuração customizada:**

```rust
let config = OrchestratorConfig {
    enable_pipeline: true,
    pipeline_stages: vec![
        PipelineStage::Sense,
        PipelineStage::Process,
        PipelineStage::Actuate,
    ],
    enable_events: true,
    event_history_size: 5000,
    component_timeout_ms: 1000,
    scheduler_config: SchedulerConfig {
        target_rate_hz: 120.0,  // 120 Hz
        mode: SchedulerMode::FixedRate,
        allow_burst: false,
        max_burst_ticks: 5,
    },
    debug: true,
};

let orch = Orchestrator::with_config(config);
```

## 🎮 Exemplos Práticos

### 1. Sistema de Percepção (L0-L4)

```rust
use sil_orchestration::*;
use sil_photonic::CameraSensor;
use sil_acoustic::MicrophoneSensor;
use sil_haptic::PressureSensor;

let orch = Orchestrator::new();

// Registrar sensores
orch.register_sensor(CameraSensor::new())?;
orch.register_sensor(MicrophoneSensor::new())?;
orch.register_sensor(PressureSensor::new())?;

// Executar pipeline de percepção a 30 Hz
let mut config = OrchestratorConfig::default();
config.scheduler_config.target_rate_hz = 30.0;
config.pipeline_stages = vec![PipelineStage::Sense, PipelineStage::Process];

let orch = Orchestrator::with_config(config);
orch.run_cycles(1000)?; // 1000 frames a 30 Hz ≈ 33 segundos
```

### 2. Loop de Controle Motor

```rust
use sil_orchestration::*;
use sil_actuator::{ServoActuator, MotorActuator};

let orch = Orchestrator::new();

// Registrar atuadores
orch.register_actuator(ServoActuator::new())?;
orch.register_actuator(MotorActuator::new())?;

// Monitorar eventos de atuadores
orch.on(EventFilter::LayerRange(6, 6), |event| {
    println!("Actuator event: {:?}", event);
})?;

// Loop de controle a 100 Hz
let mut config = OrchestratorConfig::default();
config.scheduler_config.target_rate_hz = 100.0;
let orch = Orchestrator::with_config(config);

// Executar em thread separada
std::thread::spawn(move || {
    orch.run().unwrap();
});

// ... controlar externamente ...
```

### 3. Sistema Completo (L0-LF)

```rust
use sil_orchestration::*;

let mut config = OrchestratorConfig::default();
config.debug = true;
config.scheduler_config.target_rate_hz = 60.0;

let orch = Orchestrator::with_config(config);

// Registrar componentes de todas as camadas
// L0-L4: Sensores
orch.register_sensor(/* ... */)?;

// L5-L7: Processamento
orch.register_processor(/* ... */)?;

// L6: Atuadores
orch.register_actuator(/* ... */)?;

// L8-LA: Rede + Governança (futuro)
// LB-LF: Emergência + Meta (futuro)

// Executar sistema completo
orch.run()?; // Loop infinito
```

## 📈 Métricas e Monitoramento

```rust
let orch = Orchestrator::new();
// ... executar por algum tempo ...

let stats = orch.stats()?;

println!("Components: {}", stats.component_count);
println!("  Sensors: {}", stats.sensor_count);
println!("  Processors: {}", stats.processor_count);
println!("  Actuators: {}", stats.actuator_count);
println!("Pipeline cycles: {}", stats.pipeline_cycles);
println!("Events emitted: {}", stats.event_count);
println!("Uptime: {:?}", stats.uptime);
```

## 🧪 Testes

O módulo possui **41 testes unitários** cobrindo todos os subsistemas.

```bash
# Executar todos os testes
cargo test -p sil-orchestration

# Testes específicos
cargo test -p sil-orchestration --lib orchestrator::tests
cargo test -p sil-orchestration --lib scheduler::tests
cargo test -p sil-orchestration --lib events::tests
cargo test -p sil-orchestration --lib pipeline::tests
cargo test -p sil-orchestration --lib registry::tests

# Testes de integração
cargo test -p sil-orchestration --test integration
```

## 🎯 Status de Implementação

| Componente | Status | Testes | Descrição |
|:-----------|:------:|:------:|:----------|
| Orchestrator | ✅ | 3 | Core do sistema |
| Pipeline | ✅ | 13 | Execução de estágios |
| Scheduler | ✅ | 7 | Rate control |
| EventBus | ✅ | 10 | Pub/sub de eventos |
| Registry | ✅ | 8 | Registro de componentes |
| **Total** | **✅ 100%** | **41** | **Completo** |

### Funcionalidades Implementadas

- ✅ Registro de componentes (Sensor, Processor, Actuator)
- ✅ Pipeline de 7 estágios
- ✅ Execução real de componentes por estágio
- ✅ Scheduler com 3 modos (FixedRate, FixedDelay, BestEffort)
- ✅ Event bus com filtros avançados
- ✅ Histórico de eventos limitado
- ✅ Métricas e estatísticas
- ✅ Controle de taxa (Hz) preciso
- ✅ Debug mode com logs detalhados
- ✅ Métodos `run()` e `run_cycles()`
- ✅ Estado global compartilhado

### Próximas Melhorias (Futuras)

- ⏳ Async runtime com Tokio (execução não-bloqueante)
- ⏳ Suporte completo a NetworkNode, Governor, SwarmAgent
- ⏳ Metrics exporter (Prometheus, StatsD)
- ⏳ WebSocket para monitoring remoto
- ⏳ Pipeline stages dinâmicos
- ⏳ Component hot-reload
- ⏳ Distributed orchestration (multi-node)

## 🔗 Integração com Outros Módulos

O **sil-orchestration** integra-se com todos os módulos do ecossistema SIL:

```
sil-core ← sil-orchestration → sil-*
   ↑                               ↓
   ↑    ┌────────────────────────┐ ↓
   └────┤  Todos os Traits Base  ├─┘
        └────────────────────────┘

Módulos coordenados:
- sil-photonic (L0)
- sil-acoustic (L1)
- sil-olfactory (L2)
- sil-gustatory (L3)
- sil-haptic (L4)
- sil-electronic (L5)
- sil-actuator (L6)
- sil-environment (L7)
- sil-network (L8)
- sil-governance (L9-LA)
- sil-swarm (LB)
- sil-quantum (LC)
- sil-superposition (LD)
- sil-entanglement (LE)
- sil-collapse (LF)
```

## 📖 Referências

- [ARCHITECTURE_PLAN.md](../ARCHITECTURE_PLAN.md) — Plano completo da arquitetura SIL
- [sil-core README](../sil-core/README.md) — Traits fundamentais
- [IMPLEMENTATION_STATUS.md](../IMPLEMENTATION_STATUS.md) — Status global do projeto

## 📜 Licença

AGPL-3.0 — veja [LICENSE](../LICENSE) para detalhes.

---

**⧑** *Orquestração é liberdade. Coordenação é poder.*
