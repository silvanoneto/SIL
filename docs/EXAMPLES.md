# 🎯 Exemplos Práticos — 

Este documento apresenta casos de uso completos e práticos do ecossistema /SIL.

---

## 📑 Índice

1. [Robótica Autônoma](#1-robótica-autônoma)
2. [Rede de Sensores Distribuída](#2-rede-de-sensores-distribuída)
3. [Sistema de Monitoramento Ambiental](#3-sistema-de-monitoramento-ambiental)
4. [Swarm Intelligence](#4-swarm-intelligence)
5. [Computação Quântica Simulada](#5-computação-quântica-simulada)
6. [Edge AI com NPU](#6-edge-ai-com-npu)
7. [Drone Control Loop](#7-drone-control-loop)
8. [Mesh Network P2P](#8-mesh-network-p2p)

---

## 1. Robótica Autônoma

Sistema completo de robô autônomo com percepção, planejamento e atuação.

### Arquitetura

```
Camera (L0) ────┐
Lidar (L0)  ────┤
IMU (L4)    ────┼──> Object Detection (L5)
GPS (L7)    ────┘          ↓
                    Path Planning (L5)
                           ↓
                    Motor Control (L6)
                           ↓
                    Wheels + Servos
```

### Código Completo

```rust
use sil_orchestration::*;
use sil_photonic::CameraSensor;
use sil_haptic::IMUSensor;
use sil_environment::GPSSensor;
use sil_electronic::ElectronicProcessor;
use sil_actuator::{MotorActuator, ServoActuator};

struct AutonomousRobot {
    orchestrator: Orchestrator,
}

impl AutonomousRobot {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Configurar para 50 Hz (20ms loop)
        let config = OrchestratorConfig {
            scheduler_config: SchedulerConfig {
                target_rate_hz: 50.0,
                mode: SchedulerMode::FixedRate,
                ..Default::default()
            },
            debug: false,
            ..Default::default()
        };

        let orch = Orchestrator::with_config(config);

        // === PERCEPÇÃO (L0-L4) ===
        orch.register_sensor(CameraSensor::new())?;  // L0: Visão
        orch.register_sensor(IMUSensor::new())?;      // L4: Acelerômetro/Giroscópio
        orch.register_sensor(GPSSensor::new())?;      // L7: Posição global

        // === PROCESSAMENTO (L5) ===
        let processor = ElectronicProcessor::new()?;
        orch.register_processor(processor)?;

        // === ATUAÇÃO (L6) ===
        orch.register_actuator(MotorActuator::left())?;   // Motor esquerdo
        orch.register_actuator(MotorActuator::right())?;  // Motor direito
        orch.register_actuator(ServoActuator::camera())?; // Servo da câmera

        // === EVENTOS ===
        orch.on(EventFilter::Layer(0), |event| {
            println!("📸 Camera update: {:?}", event);
        })?;

        orch.on(EventFilter::Layer(6), |event| {
            println!("⚙️  Motor action: {:?}", event);
        })?;

        orch.on(EventFilter::Error, |event| {
            eprintln!("❌ Error: {:?}", event);
        })?;

        Ok(Self { orchestrator: orch })
    }

    fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🤖 Iniciando robô autônomo...");

        self.orchestrator.start()?;

        // Loop principal
        self.orchestrator.run()?;

        Ok(())
    }

    fn run_mission(&self, duration_secs: u64) -> Result<(), Box<dyn std::error::Error>> {
        let cycles = (duration_secs * 50) as u64;  // 50 Hz

        println!("🎯 Executando missão por {} segundos ({} ciclos)",
                 duration_secs, cycles);

        self.orchestrator.start()?;
        self.orchestrator.run_cycles(cycles)?;
        self.orchestrator.stop()?;

        // Estatísticas
        let stats = self.orchestrator.stats()?;
        println!("\n📊 Missão completa:");
        println!("  • Ciclos executados: {}", stats.pipeline_cycles);
        println!("  • Eventos processados: {}", stats.event_count);
        println!("  • Uptime: {:?}", stats.uptime);

        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let robot = AutonomousRobot::new()?;

    // Executar missão de 60 segundos
    robot.run_mission(60)?;

    Ok(())
}
```

### Saída Esperada

```
🤖 Iniciando robô autônomo...
📸 Camera update: StateChange { layer: 0, ... }
⚙️  Motor action: ActuatorCommand { ... }
📸 Camera update: StateChange { layer: 0, ... }
...
📊 Missão completa:
  • Ciclos executados: 3000
  • Eventos processados: 6000
  • Uptime: 60.012s
```

---

## 2. Rede de Sensores Distribuída

Sistema P2P de sensores ambientais com consenso distribuído.

### Arquitetura

```
Nó A (Sensor de Temperatura) ──┐
Nó B (Sensor de Humidade)    ──┼──> Mesh Network (L8)
Nó C (Sensor de Pressão)     ──┘          ↓
                                   Governance (L9-LA)
                                           ↓
                                   Consenso: Alerta Global?
```

### Código Completo

```rust
use sil_network::{SilNode, SilMessage, NetworkConfig};
use sil_governance::{Governance, Proposal, Vote, ProposalData};
use sil_environment::ClimateSensor;
use sil_core::prelude::*;

struct SensorNode {
    node_id: String,
    network: SilNode,
    governance: Governance,
    sensor: ClimateSensor,
}

impl SensorNode {
    fn new(node_id: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config = NetworkConfig {
            node_id: node_id.to_string(),
            multicast_addr: "239.255.0.1:5678".parse()?,
            ..Default::default()
        };

        Ok(Self {
            node_id: node_id.to_string(),
            network: SilNode::new(config)?,
            governance: Governance::new()?,
            sensor: ClimateSensor::new()?,
        })
    }

    fn join_mesh(&mut self, mesh_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("[{}] Entrando na rede mesh '{}'", self.node_id, mesh_id);
        self.network.join_mesh(mesh_id)?;
        Ok(())
    }

    fn sense_and_broadcast(&mut self) -> Result<SilState, Box<dyn std::error::Error>> {
        // Ler sensor local
        let update = self.sensor.sense()?;
        let mut state = SilState::neutral();
        state.set_layer(update.layer, update.value)?;

        // Broadcast para a rede
        self.network.broadcast(&state)?;

        println!("[{}] 📡 Broadcasting: L{}={:?}",
                 self.node_id, update.layer, update.value);

        Ok(state)
    }

    fn receive_peer_states(&mut self) -> Result<Vec<SilState>, Box<dyn std::error::Error>> {
        let mut states = Vec::new();

        while let Some(msg) = self.network.receive()? {
            if let SilMessage::StateSync { state, .. } = msg {
                states.push(state);
            }
        }

        println!("[{}] 📥 Recebeu {} estados de peers", self.node_id, states.len());
        Ok(states)
    }

    fn propose_alert(&mut self, reason: &str) -> Result<String, Box<dyn std::error::Error>> {
        let proposal = Proposal {
            data: ProposalData::Custom(format!("ALERTA: {}", reason)),
            proposer: self.node_id.clone(),
            ..Default::default()
        };

        let id = self.governance.propose(proposal)?;
        println!("[{}] 🗳️  Proposta criada: {}", self.node_id, id);

        Ok(id)
    }

    fn vote(&mut self, proposal_id: &str, vote: Vote) -> Result<(), Box<dyn std::error::Error>> {
        self.governance.vote(proposal_id, vote)?;
        println!("[{}] ✅ Votou {:?} na proposta {}", self.node_id, vote, proposal_id);
        Ok(())
    }

    fn check_consensus(&self, proposal_id: &str) -> bool {
        use sil_governance::ProposalStatus;
        matches!(self.governance.status(proposal_id), ProposalStatus::Accepted)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 Rede de Sensores Distribuída\n");

    // Criar 3 nós
    let mut node_a = SensorNode::new("node-a")?;
    let mut node_b = SensorNode::new("node-b")?;
    let mut node_c = SensorNode::new("node-c")?;

    // Entrar na mesma rede
    let mesh_id = "environmental-sensors";
    node_a.join_mesh(mesh_id)?;
    node_b.join_mesh(mesh_id)?;
    node_c.join_mesh(mesh_id)?;

    println!("\n🔄 Loop de sensoriamento e consenso...\n");

    for cycle in 0..10 {
        println!("--- Ciclo {} ---", cycle);

        // Cada nó lê seu sensor e broadcast
        let state_a = node_a.sense_and_broadcast()?;
        let state_b = node_b.sense_and_broadcast()?;
        let state_c = node_c.sense_and_broadcast()?;

        // Receber estados dos peers
        let peers_a = node_a.receive_peer_states()?;
        let peers_b = node_b.receive_peer_states()?;
        let peers_c = node_c.receive_peer_states()?;

        // Simular detecção de anomalia
        if cycle == 5 {
            println!("\n⚠️  ANOMALIA DETECTADA!\n");

            // Nó A propõe alerta
            let proposal_id = node_a.propose_alert("Temperatura acima do limite")?;

            // Todos votam
            node_a.vote(&proposal_id, Vote::Yes)?;
            node_b.vote(&proposal_id, Vote::Yes)?;
            node_c.vote(&proposal_id, Vote::Yes)?;

            // Verificar consenso
            if node_a.check_consensus(&proposal_id) {
                println!("\n✅ CONSENSO ALCANÇADO: Alerta aceito!");
                println!("📢 Executando ação de emergência...\n");
            }
        }

        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    println!("✅ Simulação completa!");
    Ok(())
}
```

---

## 3. Sistema de Monitoramento Ambiental

Fusão de múltiplos sensores com alertas inteligentes.

### Código Completo

```rust
use sil_orchestration::*;
use sil_environment::{ClimateSensor, SensorFusion};
use sil_photonic::LightSensor;
use sil_acoustic::MicrophoneSensor;
use sil_core::prelude::*;

struct EnvironmentalMonitor {
    orch: Orchestrator,
    alert_threshold: f32,
}

impl EnvironmentalMonitor {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = OrchestratorConfig {
            scheduler_config: SchedulerConfig {
                target_rate_hz: 1.0,  // 1 Hz (1 leitura por segundo)
                mode: SchedulerMode::FixedRate,
                ..Default::default()
            },
            ..Default::default()
        };

        let orch = Orchestrator::with_config(config);

        // Sensores ambientais
        orch.register_sensor(ClimateSensor::new())?;      // L7: Temp/Humidade/Pressão
        orch.register_sensor(LightSensor::new())?;         // L0: Luminosidade
        orch.register_sensor(MicrophoneSensor::new())?;    // L1: Nível de ruído

        // Processador de fusão
        orch.register_processor(SensorFusion::new())?;     // L7: Fusão de dados

        // Handlers de eventos
        orch.on(EventFilter::Threshold, |event| {
            println!("⚠️  ALERTA: {:?}", event);
        })?;

        orch.on(EventFilter::StateChange, |event| {
            if let SilEvent::StateChange { layer, new, .. } = event {
                match layer {
                    0 => println!("💡 Luminosidade: {:?}", new),
                    1 => println!("🔊 Ruído: {:?}", new),
                    7 => println!("🌡️  Ambiente: {:?}", new),
                    _ => {}
                }
            }
        })?;

        Ok(Self {
            orch,
            alert_threshold: 0.8,
        })
    }

    fn monitor(&self, duration_secs: u64) -> Result<(), Box<dyn std::error::Error>> {
        println!("🌍 Iniciando monitoramento ambiental...");
        println!("⏱️  Duração: {} segundos\n", duration_secs);

        self.orch.start()?;
        self.orch.run_cycles(duration_secs)?;  // 1 Hz
        self.orch.stop()?;

        let stats = self.orch.stats()?;
        println!("\n📊 Estatísticas:");
        println!("  • Leituras: {}", stats.pipeline_cycles);
        println!("  • Eventos: {}", stats.event_count);
        println!("  • Sensores: {}", stats.sensor_count);
        println!("  • Processadores: {}", stats.processor_count);

        Ok(())
    }

    fn get_current_state(&self) -> Result<SilState, Box<dyn std::error::Error>> {
        Ok(self.orch.state()?)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let monitor = EnvironmentalMonitor::new()?;

    // Monitorar por 60 segundos (60 leituras)
    monitor.monitor(60)?;

    // Obter estado final
    let final_state = monitor.get_current_state()?;
    println!("\n🎯 Estado final do ambiente:");
    println!("{:#?}", final_state);

    Ok(())
}
```

---

## 4. Swarm Intelligence

Simulação de enxame com comportamento emergente.

### Código Completo

```rust
use sil_swarm::{SwarmNode, SwarmBehavior, SwarmConfig};
use sil_core::prelude::*;
use std::collections::HashMap;

struct SwarmSimulation {
    agents: HashMap<String, SwarmNode>,
    global_state: SilState,
}

impl SwarmSimulation {
    fn new(num_agents: usize) -> Self {
        let mut agents = HashMap::new();

        for i in 0..num_agents {
            let id = format!("agent-{}", i);
            let mut agent = SwarmNode::new(id.clone());
            agent.set_behavior(SwarmBehavior::Flocking);
            agents.insert(id, agent);
        }

        Self {
            agents,
            global_state: SilState::neutral(),
        }
    }

    fn connect_neighbors(&mut self) {
        let ids: Vec<String> = self.agents.keys().cloned().collect();

        for (i, id) in ids.iter().enumerate() {
            let agent = self.agents.get_mut(id).unwrap();

            // Conectar aos 5 vizinhos mais próximos (circular)
            for offset in 1..=5 {
                let neighbor_idx = (i + offset) % ids.len();
                let neighbor_id = &ids[neighbor_idx];
                agent.add_neighbor(neighbor_id.clone()).ok();
            }
        }
    }

    fn step(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Coletar estados de todos os agentes
        let mut agent_states: HashMap<String, SilState> = HashMap::new();

        for (id, agent) in &self.agents {
            let state = agent.state()?;
            agent_states.insert(id.clone(), state);
        }

        // Atualizar cada agente baseado nos vizinhos
        for (id, agent) in &mut self.agents {
            let local_state = agent_states[id].clone();

            // Obter estados dos vizinhos
            let neighbors = agent.neighbors();
            let neighbor_states: Vec<SilState> = neighbors
                .iter()
                .filter_map(|n| agent_states.get(n).cloned())
                .collect();

            // Calcular novo estado (flocking)
            let new_state = agent.behavior(&local_state, &neighbor_states);

            // Aplicar
            agent.update_state(new_state)?;
        }

        Ok(())
    }

    fn run(&mut self, steps: usize) -> Result<(), Box<dyn std::error::Error>> {
        println!("🐝 Simulação de Enxame");
        println!("  • Agentes: {}", self.agents.len());
        println!("  • Comportamento: Flocking");
        println!("  • Passos: {}\n", steps);

        for step in 0..steps {
            self.step()?;

            if step % 10 == 0 {
                println!("⏱️  Passo {}/{}", step, steps);
                self.print_statistics();
            }
        }

        println!("\n✅ Simulação completa!");
        Ok(())
    }

    fn print_statistics(&self) {
        // Calcular centro de massa do enxame
        let mut sum = SilState::neutral();
        for agent in self.agents.values() {
            if let Ok(state) = agent.state() {
                // XOR para combinar (simplificado)
                for layer in 0..16 {
                    if let (Ok(a), Ok(b)) = (sum.get_layer(layer), state.get_layer(layer)) {
                        let combined = a.xor(&b);
                        sum.set_layer(layer, combined).ok();
                    }
                }
            }
        }
        println!("  📍 Centro de massa: {:?}", sum.get_layer(0));
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Criar enxame com 50 agentes
    let mut swarm = SwarmSimulation::new(50);

    // Conectar vizinhança
    swarm.connect_neighbors();

    // Executar 100 passos
    swarm.run(100)?;

    Ok(())
}
```

---

## 5. Computação Quântica Simulada

Fork/merge de estados com superposição.

### Código Completo

```rust
use sil_quantum::QuantumProcessor;
use sil_superposition::{StateManager, MergeStrategy};
use sil_core::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌌 Computação Quântica Simulada\n");

    // === PARTE 1: Superposição ===
    println!("📊 Criando superposição de 3 estados...");

    let state1 = SilState::neutral();
    let mut state2 = SilState::neutral();
    state2.set_layer(0, ByteSil { rho: 10, theta: 0 })?;
    let mut state3 = SilState::neutral();
    state3.set_layer(0, ByteSil { rho: 5, theta: 128 })?;

    let states = vec![state1, state2, state3];
    let weights = vec![0.5, 0.3, 0.2];

    let mut qp = QuantumProcessor::new();
    let superposed = qp.superpose(&states, &weights);

    println!("  • Coerência: {:.2}", qp.coherence());
    println!("  • Estado superposto: {:?}\n", superposed.get_layer(0)?);

    // === PARTE 2: Fork/Merge ===
    println!("🔀 Explorando múltiplos caminhos...");

    let mut manager = StateManager::new(superposed.clone());
    manager.set_default_strategy(MergeStrategy::Max);

    // Fork 1: Processar com estratégia A
    println!("  Criando fork 1...");
    let mut fork1 = manager.fork();
    for layer in 0..4 {
        let val = fork1.get_layer(layer)?;
        let scaled = ByteSil {
            rho: val.rho().saturating_add(2),
            theta: val.theta(),
        };
        fork1.set_layer(layer, scaled)?;
    }

    // Fork 2: Processar com estratégia B
    println!("  Criando fork 2...");
    let mut fork2 = manager.fork();
    for layer in 0..4 {
        let val = fork2.get_layer(layer)?;
        let rotated = ByteSil {
            rho: val.rho(),
            theta: val.theta().wrapping_add(64),
        };
        fork2.set_layer(layer, rotated)?;
    }

    // Merge com estratégia Max (pega maior magnitude)
    println!("  Fazendo merge com estratégia Max...");
    manager.merge_with_strategy(&fork1, MergeStrategy::Max)?;
    manager.merge_with_strategy(&fork2, MergeStrategy::Max)?;

    let final_state = manager.current();
    println!("  • Estado final: {:?}\n", final_state.get_layer(0)?);

    // === PARTE 3: Colapso ===
    println!("💥 Colapsando superposição...");

    let seed = 42;
    let collapsed = qp.collapse(seed);

    println!("  • Seed: {}", seed);
    println!("  • Estado colapsado: {:?}", collapsed.get_layer(0)?);
    println!("  • Coerência pós-colapso: {:.2}\n", qp.coherence());

    println!("✅ Simulação quântica completa!");
    Ok(())
}
```

---

## 6. Edge AI com NPU

Inferência em hardware neural (futuro).

### Código (Pseudo-implementation)

```rust
use sil_electronic::{ElectronicProcessor, ProcessorConfig, Backend};
use sil_core::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧠 Edge AI com NPU\n");

    // Configurar para usar NPU
    let config = ProcessorConfig {
        backend: Backend::NPU,  // Requer feature "npu"
        ..Default::default()
    };

    let mut processor = ElectronicProcessor::with_config(config)?;

    // Carregar modelo neural (bytecode VSP)
    let model_bytes = std::fs::read("models/classifier.silc")?;
    processor.load_bytecode(&model_bytes)?;

    println!("✅ Modelo carregado ({} bytes)", model_bytes.len());
    println!("⚡ Backend: NPU (Neural Engine)\n");

    // Loop de inferência
    for i in 0..10 {
        // Simular entrada sensorial
        let mut state = SilState::neutral();
        state.set_layer(0, ByteSil { rho: 10, theta: (i * 25) as u8 })?;

        // Inferência no NPU
        let start = std::time::Instant::now();
        let result = processor.execute(&state)?;
        let duration = start.elapsed();

        println!("Frame {}: {:?} (latency: {:?})",
                 i, result.get_layer(5)?, duration);

        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    println!("\n✅ Inferência completa!");
    Ok(())
}
```

---

## 7. Drone Control Loop

Loop de controle para quadricóptero.

### Código Completo

```rust
use sil_orchestration::*;
use sil_haptic::IMUSensor;
use sil_environment::BarometerSensor;
use sil_actuator::{MotorActuator, ActuatorCommand};
use sil_core::prelude::*;

struct DroneController {
    orch: Orchestrator,
    target_altitude: f32,
    kp: f32,  // Proporcional
    ki: f32,  // Integral
    kd: f32,  // Derivativo
}

impl DroneController {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Loop de controle a 400 Hz (2.5ms)
        let config = OrchestratorConfig {
            scheduler_config: SchedulerConfig {
                target_rate_hz: 400.0,
                mode: SchedulerMode::FixedRate,
                ..Default::default()
            },
            ..Default::default()
        };

        let orch = Orchestrator::with_config(config);

        // Sensores
        orch.register_sensor(IMUSensor::new())?;           // L4: Aceleração/rotação
        orch.register_sensor(BarometerSensor::new())?;     // L7: Altitude

        // Motores (4 ESCs)
        orch.register_actuator(MotorActuator::motor(0))?;  // Motor 1
        orch.register_actuator(MotorActuator::motor(1))?;  // Motor 2
        orch.register_actuator(MotorActuator::motor(2))?;  // Motor 3
        orch.register_actuator(MotorActuator::motor(3))?;  // Motor 4

        Ok(Self {
            orch,
            target_altitude: 10.0,  // 10 metros
            kp: 1.0,
            ki: 0.1,
            kd: 0.5,
        })
    }

    fn fly(&self, duration_secs: u64) -> Result<(), Box<dyn std::error::Error>> {
        println!("🚁 Drone Controller - Loop de 400 Hz");
        println!("🎯 Altitude alvo: {:.1}m\n", self.target_altitude);

        let cycles = duration_secs * 400;  // 400 Hz

        self.orch.start()?;
        self.orch.run_cycles(cycles)?;
        self.orch.stop()?;

        let stats = self.orch.stats()?;
        println!("\n📊 Estatísticas de voo:");
        println!("  • Ciclos: {}", stats.pipeline_cycles);
        println!("  • Taxa média: {:.1} Hz",
                 stats.pipeline_cycles as f64 / stats.uptime.as_secs_f64());

        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let drone = DroneController::new()?;

    // Voar por 10 segundos
    drone.fly(10)?;

    Ok(())
}
```

---

## 8. Mesh Network P2P

Rede mesh auto-organizável.

### Código Completo

```rust
use sil_network::{SilNode, NetworkConfig, MeshNode};
use sil_core::prelude::*;
use std::collections::HashMap;

struct MeshNetwork {
    nodes: HashMap<String, MeshNode>,
}

impl MeshNetwork {
    fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    fn add_node(&mut self, node_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let config = NetworkConfig {
            node_id: node_id.to_string(),
            multicast_addr: "239.255.0.1:5678".parse()?,
            ..Default::default()
        };

        let node = MeshNode::new(config)?;
        self.nodes.insert(node_id.to_string(), node);

        println!("➕ Nó '{}' adicionado", node_id);
        Ok(())
    }

    fn connect(&mut self, node_a: &str, node_b: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let (Some(a), Some(b)) = (self.nodes.get_mut(node_a), self.nodes.get(node_b)) {
            a.add_peer(node_b)?;
            println!("🔗 Conectado: {} <-> {}", node_a, node_b);
        }
        Ok(())
    }

    fn broadcast_from(&mut self, node_id: &str, state: &SilState) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(node) = self.nodes.get_mut(node_id) {
            node.broadcast(state)?;
            println!("📡 '{}' broadcasting...", node_id);
        }
        Ok(())
    }

    fn print_topology(&self) {
        println!("\n🌐 Topologia da Rede:");
        for (id, node) in &self.nodes {
            let peers = node.peer_count();
            println!("  • {}: {} peers", id, peers);
        }
        println!();
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🕸️  Mesh Network P2P\n");

    let mut mesh = MeshNetwork::new();

    // Criar 5 nós
    mesh.add_node("node-1")?;
    mesh.add_node("node-2")?;
    mesh.add_node("node-3")?;
    mesh.add_node("node-4")?;
    mesh.add_node("node-5")?;

    // Conectar em topologia de anel
    mesh.connect("node-1", "node-2")?;
    mesh.connect("node-2", "node-3")?;
    mesh.connect("node-3", "node-4")?;
    mesh.connect("node-4", "node-5")?;
    mesh.connect("node-5", "node-1")?;

    // Adicionar atalhos (mesh)
    mesh.connect("node-1", "node-3")?;
    mesh.connect("node-2", "node-4")?;

    mesh.print_topology();

    // Broadcast de estado
    let state = SilState::neutral();
    mesh.broadcast_from("node-1", &state)?;

    println!("✅ Rede estabelecida!");
    Ok(())
}
```

---

## 📚 Recursos Adicionais

### Documentação

- [ARCHITECTURE.md](ARCHITECTURE.md) — Arquitetura completa
- [DIAGRAMS.md](DIAGRAMS.md) — Diagramas visuais (Mermaid)
- [GETTING_STARTED.md](GETTING_STARTED.md) — Tutorial passo a passo
- [PERFORMANCE.md](PERFORMANCE.md) — Benchmarks e otimizações

### Módulos

- [sil-core README](sil-core/README.md)
- [sil-orchestration README](sil-orchestration/README.md)
- [sil-network README](sil-network/README.md)
- [sil-swarm README](sil-swarm/README.md)

---

**⧑** *Estes exemplos são apenas o começo. O que você vai construir?*
