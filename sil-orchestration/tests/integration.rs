//! Testes de integração para sil-orchestration

use sil_orchestration::*;
use sil_core::prelude::*;

#[test]
fn test_orchestrator_creation() {
    let orch = Orchestrator::new();
    assert!(orch.component_count().is_ok());
}

#[test]
fn test_orchestrator_with_config() {
    let config = OrchestratorConfig {
        enable_pipeline: true,
        pipeline_stages: vec![PipelineStage::Sense, PipelineStage::Process],
        enable_events: true,
        event_history_size: 100,
        component_timeout_ms: 1000,
        scheduler_config: SchedulerConfig::default(),
        debug: false,
    };

    let orch = Orchestrator::with_config(config);
    assert!(orch.component_count().is_ok());
}

#[test]
fn test_orchestrator_start_stop() {
    let orch = Orchestrator::new();

    assert!(!orch.is_running().unwrap());

    orch.start().unwrap();
    assert!(orch.is_running().unwrap());

    orch.stop().unwrap();
    assert!(!orch.is_running().unwrap());
}

#[test]
fn test_orchestrator_event_emission() {
    let orch = Orchestrator::new();

    let event = SilEvent::StateChange {
        layer: 0,
        old: ByteSil::NULL,
        new: ByteSil::ONE,
        timestamp: 0,
    };

    orch.emit(event).unwrap();

    let history = orch.event_history();
    assert_eq!(history.len(), 1);
}

#[test]
fn test_orchestrator_stats() {
    let orch = Orchestrator::new();
    let stats = orch.stats().unwrap();

    assert_eq!(stats.component_count, 0);
    assert_eq!(stats.sensor_count, 0);
    assert_eq!(stats.processor_count, 0);
}

#[test]
fn test_pipeline_control() {
    let orch = Orchestrator::new();
    orch.start().unwrap();

    assert_eq!(orch.current_stage().unwrap(), Some(PipelineStage::Sense));

    orch.tick().unwrap();
    assert_eq!(orch.current_stage().unwrap(), Some(PipelineStage::Process));
}

#[test]
fn test_state_management() {
    let orch = Orchestrator::new();

    let initial_state = orch.state().unwrap();
    assert_eq!(initial_state, SilState::default());

    let mut new_state = SilState::default();
    new_state.set_layer(0, ByteSil::ONE);
    orch.update_state(new_state).unwrap();

    let updated_state = orch.state().unwrap();
    assert_eq!(updated_state.get(0), ByteSil::ONE);
}
