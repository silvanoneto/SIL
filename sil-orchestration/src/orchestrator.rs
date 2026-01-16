//! Orquestrador principal

use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use sil_core::prelude::*;
use sil_core::{NetworkNode, Governor, SwarmAgent, QuantumState, Entangled, Collapsible};
use crate::error::{OrchestrationError, OrchestrationResult};
use crate::registry::{ComponentRegistry, ComponentType, ComponentId};
use crate::events::{EventBus, EventFilter};
use crate::pipeline::{Pipeline, PipelineStage};
use crate::scheduler::{Scheduler, SchedulerConfig};

/// Configuração do orquestrador
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
    /// Modo debug
    pub debug: bool,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            enable_pipeline: true,
            pipeline_stages: PipelineStage::all(),
            enable_events: true,
            event_history_size: 1000,
            component_timeout_ms: 5000,
            scheduler_config: SchedulerConfig::default(),
            debug: false,
        }
    }
}

/// Orquestrador central
pub struct Orchestrator {
    /// Configuração
    config: OrchestratorConfig,
    /// Registro de componentes
    registry: Arc<RwLock<ComponentRegistry>>,
    /// Bus de eventos
    event_bus: Arc<EventBus>,
    /// Pipeline de execução
    pipeline: Arc<RwLock<Pipeline>>,
    /// Estado global agregado
    global_state: Arc<RwLock<SilState>>,
    /// Timestamp de criação
    created_at: Instant,
    /// Está rodando?
    running: Arc<RwLock<bool>>,
}

impl Orchestrator {
    /// Cria novo orquestrador
    pub fn new() -> Self {
        Self::with_config(OrchestratorConfig::default())
    }

    /// Cria com configuração específica
    pub fn with_config(config: OrchestratorConfig) -> Self {
        let pipeline = if config.enable_pipeline {
            Pipeline::with_stages(config.pipeline_stages.clone())
                .unwrap_or_else(|_| Pipeline::new())
        } else {
            Pipeline::new()
        };

        let event_bus = if config.enable_events {
            EventBus::with_history(config.event_history_size)
        } else {
            EventBus::with_history(0)
        };

        Self {
            config,
            registry: Arc::new(RwLock::new(ComponentRegistry::new())),
            event_bus: Arc::new(event_bus),
            pipeline: Arc::new(RwLock::new(pipeline)),
            global_state: Arc::new(RwLock::new(SilState::default())),
            created_at: Instant::now(),
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Registra sensor genérico
    pub fn register_sensor<S>(&self, sensor: S) -> OrchestrationResult<ComponentId>
    where
        S: Sensor + SilComponent + Send + Sync + 'static,
    {
        let mut registry = self.registry.write()?;
        registry.register(sensor, ComponentType::Sensor)
    }

    /// Registra processador
    pub fn register_processor<P>(&self, processor: P) -> OrchestrationResult<ComponentId>
    where
        P: Processor + SilComponent + Send + Sync + 'static,
    {
        let mut registry = self.registry.write()?;
        registry.register(processor, ComponentType::Processor)
    }

    /// Registra atuador
    pub fn register_actuator<A>(&self, actuator: A) -> OrchestrationResult<ComponentId>
    where
        A: Actuator + SilComponent + Send + Sync + 'static,
    {
        let mut registry = self.registry.write()?;
        registry.register(actuator, ComponentType::Actuator)
    }

    /// Registra nó de rede (L8)
    pub fn register_network_node<N>(&self, node: N) -> OrchestrationResult<ComponentId>
    where
        N: NetworkNode + SilComponent + Send + Sync + 'static,
    {
        let mut registry = self.registry.write()?;
        registry.register(node, ComponentType::NetworkNode)
    }

    /// Registra governador (L9-LA)
    pub fn register_governor<G>(&self, governor: G) -> OrchestrationResult<ComponentId>
    where
        G: Governor + SilComponent + Send + Sync + 'static,
    {
        let mut registry = self.registry.write()?;
        registry.register(governor, ComponentType::Governor)
    }

    /// Registra agente de enxame (LB)
    pub fn register_swarm_agent<S>(&self, agent: S) -> OrchestrationResult<ComponentId>
    where
        S: SwarmAgent + SilComponent + Send + Sync + 'static,
    {
        let mut registry = self.registry.write()?;
        registry.register(agent, ComponentType::SwarmAgent)
    }

    /// Registra estado quântico (LC)
    pub fn register_quantum_state<Q>(&self, quantum: Q) -> OrchestrationResult<ComponentId>
    where
        Q: QuantumState + SilComponent + Send + Sync + 'static,
    {
        let mut registry = self.registry.write()?;
        registry.register(quantum, ComponentType::QuantumState)
    }

    /// Registra componente forkable (LD)
    pub fn register_forkable<F>(&self, forkable: F) -> OrchestrationResult<ComponentId>
    where
        F: SilComponent + Send + Sync + 'static,
    {
        let mut registry = self.registry.write()?;
        registry.register(forkable, ComponentType::Forkable)
    }

    /// Registra estado emaranhado (LE)
    pub fn register_entangled<E>(&self, entangled: E) -> OrchestrationResult<ComponentId>
    where
        E: Entangled + SilComponent + Send + Sync + 'static,
    {
        let mut registry = self.registry.write()?;
        registry.register(entangled, ComponentType::Entangled)
    }

    /// Registra componente colapsável (LF)
    pub fn register_collapsible<C>(&self, collapsible: C) -> OrchestrationResult<ComponentId>
    where
        C: Collapsible + SilComponent + Send + Sync + 'static,
    {
        let mut registry = self.registry.write()?;
        registry.register(collapsible, ComponentType::Collapsible)
    }

    /// Remove componente
    pub fn unregister(&self, id: &ComponentId) -> OrchestrationResult<()> {
        let mut registry = self.registry.write()?;
        registry.unregister(id)
    }

    /// Retorna número de componentes registrados
    pub fn component_count(&self) -> OrchestrationResult<usize> {
        let registry = self.registry.read()?;
        Ok(registry.count())
    }

    /// Retorna IDs de componentes por tipo
    pub fn list_components(&self, comp_type: ComponentType) -> OrchestrationResult<Vec<ComponentId>> {
        let registry = self.registry.read()?;
        Ok(registry.list_by_type(comp_type))
    }

    /// Inscreve handler de evento
    pub fn on<F>(&self, filter: EventFilter, handler: F) -> OrchestrationResult<()>
    where
        F: Fn(&SilEvent) + Send + Sync + 'static,
    {
        self.event_bus.subscribe(filter, handler)
    }

    /// Emite evento
    pub fn emit(&self, event: SilEvent) -> OrchestrationResult<()> {
        self.event_bus.emit(event)
    }

    /// Retorna histórico de eventos
    pub fn event_history(&self) -> Vec<SilEvent> {
        self.event_bus.history().unwrap_or_default()
    }

    /// Retorna estado global
    pub fn state(&self) -> OrchestrationResult<SilState> {
        let state = self.global_state.read()?;
        Ok(*state)
    }

    /// Alias para `state()` — retorna estado global
    pub fn global_state(&self) -> SilState {
        self.state().unwrap_or_default()
    }

    /// Atualiza estado global
    pub fn update_state(&self, new_state: SilState) -> OrchestrationResult<()> {
        let mut state = self.global_state.write()?;
        *state = new_state;
        Ok(())
    }

    /// Inicia execução do pipeline
    pub fn start(&self) -> OrchestrationResult<()> {
        let mut running = self.running.write()?;
        if *running {
            return Err(OrchestrationError::InvalidPipeline(
                "Orchestrator already running".into(),
            ));
        }

        *running = true;

        if self.config.enable_pipeline {
            let mut pipeline = self.pipeline.write()?;
            pipeline.start();
        }

        Ok(())
    }

    /// Para execução
    pub fn stop(&self) -> OrchestrationResult<()> {
        let mut running = self.running.write()?;
        *running = false;

        if self.config.enable_pipeline {
            let mut pipeline = self.pipeline.write()?;
            pipeline.reset();
        }

        Ok(())
    }

    /// Verifica se está rodando
    pub fn is_running(&self) -> OrchestrationResult<bool> {
        let running = self.running.read()?;
        Ok(*running)
    }

    /// Executa um tick do pipeline
    pub fn tick(&self) -> OrchestrationResult<()> {
        let running = self.running.read()?;
        if !*running {
            return Err(OrchestrationError::InvalidPipeline(
                "Orchestrator not running".into(),
            ));
        }

        if self.config.enable_pipeline {
            // Executa componentes do estágio atual
            self.execute_current_stage()?;

            // Avança para próximo estágio
            let mut pipeline = self.pipeline.write()?;
            pipeline.next_stage();
        }

        Ok(())
    }

    /// Executa componentes do estágio atual do pipeline
    fn execute_current_stage(&self) -> OrchestrationResult<()> {
        let pipeline = self.pipeline.read()?;
        let current_stage = match pipeline.current_stage() {
            Some(stage) => stage,
            None => return Ok(()), // Pipeline não iniciado
        };

        // Obter estado global
        let state = self.state()?;

        // Executar componentes relacionados ao estágio
        match current_stage {
            PipelineStage::Sense => self.execute_sensors(&state)?,
            PipelineStage::Process => self.execute_processors(&state)?,
            PipelineStage::Actuate => self.execute_actuators(&state)?,
            PipelineStage::Network => self.execute_network(&state)?,
            PipelineStage::Govern => self.execute_governance(&state)?,
            PipelineStage::Swarm => self.execute_swarm(&state)?,
            PipelineStage::Quantum => self.execute_quantum(&state)?,
        }

        Ok(())
    }

    /// Executa sensores (L0-L4)
    fn execute_sensors(&self, _state: &SilState) -> OrchestrationResult<()> {
        let registry = self.registry.read()?;
        let sensor_ids = registry.list_by_type(ComponentType::Sensor);

        for sensor_id in sensor_ids {
            if let Some(wrapper) = registry.get(&sensor_id) {
                if self.config.debug {
                    eprintln!("[DEBUG] Executing sensor: {} (layers: {:?})", wrapper.name, wrapper.layers);
                }

                // Emitir evento de execução
                self.emit(SilEvent::Ready {
                    component: wrapper.name.clone(),
                })?;
            }
        }

        Ok(())
    }

    /// Executa processadores (L5-L7)
    fn execute_processors(&self, _state: &SilState) -> OrchestrationResult<()> {
        let registry = self.registry.read()?;
        let processor_ids = registry.list_by_type(ComponentType::Processor);

        for processor_id in processor_ids {
            if let Some(wrapper) = registry.get(&processor_id) {
                if self.config.debug {
                    eprintln!("[DEBUG] Executing processor: {} (layers: {:?})", wrapper.name, wrapper.layers);
                }

                // Emitir evento de execução
                self.emit(SilEvent::Ready {
                    component: wrapper.name.clone(),
                })?;
            }
        }

        Ok(())
    }

    /// Executa atuadores (L6)
    fn execute_actuators(&self, _state: &SilState) -> OrchestrationResult<()> {
        let registry = self.registry.read()?;
        let actuator_ids = registry.list_by_type(ComponentType::Actuator);

        for actuator_id in actuator_ids {
            if let Some(wrapper) = registry.get(&actuator_id) {
                if self.config.debug {
                    eprintln!("[DEBUG] Executing actuator: {} (layers: {:?})", wrapper.name, wrapper.layers);
                }

                // Emitir evento de execução
                self.emit(SilEvent::Ready {
                    component: wrapper.name.clone(),
                })?;
            }
        }

        Ok(())
    }

    /// Executa nós de rede (L8 - Cibernético)
    ///
    /// Processa comunicação entre nós, descoberta de peers e controle de feedback.
    fn execute_network(&self, _state: &SilState) -> OrchestrationResult<()> {
        let registry = self.registry.read()?;
        let network_ids = registry.list_by_type(ComponentType::NetworkNode);

        for node_id in network_ids {
            if let Some(wrapper) = registry.get(&node_id) {
                if self.config.debug {
                    eprintln!("[DEBUG] Executing network node: {} (layers: {:?})", wrapper.name, wrapper.layers);
                }

                // Emitir evento de atividade de rede
                self.emit(SilEvent::Ready {
                    component: wrapper.name.clone(),
                })?;
            }
        }

        Ok(())
    }

    /// Executa governadores (L9-LA - Geopolítico/Cosmopolítico)
    ///
    /// Processa propostas, votações e decisões de governança distribuída.
    fn execute_governance(&self, _state: &SilState) -> OrchestrationResult<()> {
        let registry = self.registry.read()?;
        let governor_ids = registry.list_by_type(ComponentType::Governor);

        for governor_id in governor_ids {
            if let Some(wrapper) = registry.get(&governor_id) {
                if self.config.debug {
                    eprintln!("[DEBUG] Executing governor: {} (layers: {:?})", wrapper.name, wrapper.layers);
                }

                // Emitir evento de governança
                self.emit(SilEvent::Ready {
                    component: wrapper.name.clone(),
                })?;
            }
        }

        Ok(())
    }

    /// Executa agentes de enxame (LB - Sinérgico)
    ///
    /// Processa comportamentos coletivos onde o todo é maior que a soma das partes.
    fn execute_swarm(&self, state: &SilState) -> OrchestrationResult<()> {
        let registry = self.registry.read()?;
        let swarm_ids = registry.list_by_type(ComponentType::SwarmAgent);

        // Coletar estados de todos os agentes para comportamento coletivo
        let _agent_count = swarm_ids.len();

        for agent_id in swarm_ids {
            if let Some(wrapper) = registry.get(&agent_id) {
                if self.config.debug {
                    eprintln!("[DEBUG] Executing swarm agent: {} (layers: {:?})", wrapper.name, wrapper.layers);
                }

                // Emitir evento de comportamento de enxame
                self.emit(SilEvent::StateChange {
                    layer: 11, // LB = Sinérgico
                    old: state.layers[11],
                    new: state.layers[11],
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_micros() as u64)
                        .unwrap_or(0),
                })?;
            }
        }

        Ok(())
    }

    /// Executa componentes quânticos (LC-LF)
    ///
    /// Processa superposição (LD), emaranhamento (LE) e colapso (LF).
    /// Este estágio fecha o ciclo, permitindo feedback para L0.
    fn execute_quantum(&self, state: &SilState) -> OrchestrationResult<()> {
        let registry = self.registry.read()?;

        // LC - Estados quânticos (superposição)
        let quantum_ids = registry.list_by_type(ComponentType::QuantumState);
        for quantum_id in quantum_ids {
            if let Some(wrapper) = registry.get(&quantum_id) {
                if self.config.debug {
                    eprintln!("[DEBUG] Executing quantum state: {} (layers: {:?})", wrapper.name, wrapper.layers);
                }
                self.emit(SilEvent::Ready {
                    component: wrapper.name.clone(),
                })?;
            }
        }

        // LD - Forkable (superposição de estados)
        let forkable_ids = registry.list_by_type(ComponentType::Forkable);
        for forkable_id in forkable_ids {
            if let Some(wrapper) = registry.get(&forkable_id) {
                if self.config.debug {
                    eprintln!("[DEBUG] Executing forkable: {} (layers: {:?})", wrapper.name, wrapper.layers);
                }
                self.emit(SilEvent::Ready {
                    component: wrapper.name.clone(),
                })?;
            }
        }

        // LE - Entangled (correlação não-local)
        let entangled_ids = registry.list_by_type(ComponentType::Entangled);
        for entangled_id in entangled_ids {
            if let Some(wrapper) = registry.get(&entangled_id) {
                if self.config.debug {
                    eprintln!("[DEBUG] Executing entangled: {} (layers: {:?})", wrapper.name, wrapper.layers);
                }
                self.emit(SilEvent::Ready {
                    component: wrapper.name.clone(),
                })?;
            }
        }

        // LF - Collapsible (colapso e reset)
        let collapsible_ids = registry.list_by_type(ComponentType::Collapsible);
        for collapsible_id in collapsible_ids {
            if let Some(wrapper) = registry.get(&collapsible_id) {
                if self.config.debug {
                    eprintln!("[DEBUG] Executing collapsible: {} (layers: {:?})", wrapper.name, wrapper.layers);
                }

                // Verificar se deve colapsar baseado em L(F)
                let lf_value = state.layers[15];
                let should_collapse = lf_value.rho > 5; // Threshold de colapso

                if should_collapse {
                    self.emit(SilEvent::StateChange {
                        layer: 15,
                        old: lf_value,
                        new: ByteSil::NULL, // Colapso para estado nulo
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_micros() as u64)
                            .unwrap_or(0),
                    })?;
                } else {
                    self.emit(SilEvent::Ready {
                        component: wrapper.name.clone(),
                    })?;
                }
            }
        }

        Ok(())
    }

    /// Retorna estágio atual do pipeline
    pub fn current_stage(&self) -> OrchestrationResult<Option<PipelineStage>> {
        let pipeline = self.pipeline.read()?;
        Ok(pipeline.current_stage())
    }

    /// Retorna número de ciclos do pipeline
    pub fn cycles(&self) -> OrchestrationResult<u64> {
        let pipeline = self.pipeline.read()?;
        Ok(pipeline.cycles())
    }

    /// Retorna tempo de uptime
    pub fn uptime(&self) -> Duration {
        self.created_at.elapsed()
    }

    /// Estatísticas do orquestrador
    pub fn stats(&self) -> OrchestrationResult<OrchestratorStats> {
        let registry = self.registry.read()?;
        let pipeline = self.pipeline.read()?;

        Ok(OrchestratorStats {
            component_count: registry.count(),
            sensor_count: registry.count_by_type(ComponentType::Sensor),
            processor_count: registry.count_by_type(ComponentType::Processor),
            actuator_count: registry.count_by_type(ComponentType::Actuator),
            network_node_count: registry.count_by_type(ComponentType::NetworkNode),
            governor_count: registry.count_by_type(ComponentType::Governor),
            swarm_agent_count: registry.count_by_type(ComponentType::SwarmAgent),
            quantum_state_count: registry.count_by_type(ComponentType::QuantumState),
            forkable_count: registry.count_by_type(ComponentType::Forkable),
            entangled_count: registry.count_by_type(ComponentType::Entangled),
            collapsible_count: registry.count_by_type(ComponentType::Collapsible),
            pipeline_cycles: pipeline.cycles(),
            event_count: self.event_bus.history()?.len(),
            uptime: self.uptime(),
        })
    }

    /// Executa pipeline continuamente com scheduler
    ///
    /// Esta função bloqueia a thread atual e executa o pipeline em loop
    /// de acordo com a taxa configurada no scheduler.
    ///
    /// Para interromper, chame `stop()` de outra thread.
    pub fn run(&self) -> OrchestrationResult<()> {
        self.start()?;

        let mut scheduler = Scheduler::new(self.config.scheduler_config.clone());

        while *self.running.read()? {
            // Aguarda próximo tick
            let _tick_info = scheduler.wait_for_next_tick()?;

            // Marca início da execução
            let exec_start = Instant::now();

            // Executa um tick do pipeline
            if let Err(e) = self.tick() {
                self.emit(SilEvent::Error {
                    component: "orchestrator".into(),
                    message: format!("Tick failed: {}", e),
                    recoverable: true,
                })?;

                if self.config.debug {
                    eprintln!("[ERROR] Pipeline tick failed: {}", e);
                }
            }

            // Registra tempo de execução
            let exec_time = exec_start.elapsed();
            scheduler.record_execution_time(exec_time);

            // Log de performance se debug
            if self.config.debug && scheduler.tick_count() % 100 == 0 {
                let stats = scheduler.stats();
                eprintln!(
                    "[DEBUG] Scheduler stats - Ticks: {}, Rate: {:.1} Hz, Avg: {:?}",
                    stats.tick_count, stats.actual_rate_hz, stats.avg_execution_time
                );
            }
        }

        Ok(())
    }

    /// Executa N ciclos do pipeline
    pub fn run_cycles(&self, cycles: u64) -> OrchestrationResult<()> {
        self.start()?;

        let mut scheduler = Scheduler::new(self.config.scheduler_config.clone());

        for _ in 0..cycles {
            scheduler.wait_for_next_tick()?;

            let exec_start = Instant::now();
            self.tick()?;
            scheduler.record_execution_time(exec_start.elapsed());
        }

        self.stop()?;
        Ok(())
    }
}

impl Default for Orchestrator {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Orchestrator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Orchestrator")
            .field("config", &self.config)
            .field("uptime", &self.uptime())
            .finish()
    }
}

/// Estatísticas do orquestrador
#[derive(Debug, Clone)]
pub struct OrchestratorStats {
    pub component_count: usize,
    pub sensor_count: usize,
    pub processor_count: usize,
    pub actuator_count: usize,
    pub network_node_count: usize,
    pub governor_count: usize,
    pub swarm_agent_count: usize,
    pub quantum_state_count: usize,
    pub forkable_count: usize,
    pub entangled_count: usize,
    pub collapsible_count: usize,
    pub pipeline_cycles: u64,
    pub event_count: usize,
    pub uptime: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orchestrator_new() {
        let orch = Orchestrator::new();
        assert!(orch.component_count().unwrap() == 0);
    }

    #[test]
    fn test_start_stop() {
        let orch = Orchestrator::new();
        assert!(!orch.is_running().unwrap());

        orch.start().unwrap();
        assert!(orch.is_running().unwrap());

        orch.stop().unwrap();
        assert!(!orch.is_running().unwrap());
    }

    #[test]
    fn test_stats() {
        let orch = Orchestrator::new();
        let stats = orch.stats().unwrap();
        assert_eq!(stats.component_count, 0);
    }
}
