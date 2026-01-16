//! Testes integrados para sil-electronic

use crate::*;
use sil_core::prelude::*;

#[test]
fn test_electronic_processor_creation() {
    let processor = ElectronicProcessor::new();
    assert!(processor.is_ok(), "Failed to create ElectronicProcessor");
}

#[test]
fn test_electronic_processor_with_custom_config() {
    let config = ElectronicConfig {
        heap_size: 16384,
        stack_size: 512,
        enable_gpu: false,
        enable_npu: false,
        max_cycles: 100000,
        debug: false,
    };
    let processor = ElectronicProcessor::with_config(config);
    assert!(processor.is_ok());
}

#[test]
fn test_processor_load_and_check_state() {
    let mut processor = ElectronicProcessor::new().unwrap();
    let code = vec![0xFF; 100]; // dummy bytecode
    let data = vec![0x00; 50];

    let result = processor.load_bytes(&code, &data);
    assert!(result.is_ok());

    let state = processor.state().unwrap();
    assert_eq!(state.bytecode.len(), 100);
    assert_eq!(state.data.len(), 50);
    assert_eq!(state.pc, 0);
    assert_eq!(state.cycles, 0);
    assert!(!state.halted);
}

#[test]
fn test_processor_ready_check() {
    let mut processor = ElectronicProcessor::new().unwrap();

    // Sem bytecode, estado não está ready
    let state = processor.state().unwrap();
    assert!(!state.is_ready());

    // Com bytecode, estado está ready
    processor.load_bytes(&[0x00, 0x01], &[]).unwrap();
    let state = processor.state().unwrap();
    assert!(state.is_ready());

    // Após reset, sem bytecode novamente
    processor.reset().unwrap();
    let state = processor.state().unwrap();
    assert!(!state.is_ready());
}

#[test]
fn test_processor_reset_clears_state() {
    let mut processor = ElectronicProcessor::new().unwrap();
    processor.load_bytes(&[0x01, 0x02, 0x03], &[0x04]).unwrap();

    let state_before = processor.state().unwrap();
    assert_eq!(state_before.bytecode.len(), 3);

    processor.reset().unwrap();

    let state_after = processor.state().unwrap();
    assert_eq!(state_after.pc, 0);
    assert_eq!(state_after.cycles, 0);
    assert!(!state_after.halted);
    assert!(state_after.error.is_none());
}

#[test]
fn test_processor_status_transitions() {
    let mut processor = ElectronicProcessor::new().unwrap();

    // Sem bytecode
    let state = processor.state().unwrap();
    assert!(!state.is_ready());

    // Com bytecode
    processor.load_bytes(&[0x00], &[]).unwrap();
    let state = processor.state().unwrap();
    assert!(state.is_ready());
}

#[test]
fn test_processor_execution_info() {
    let mut processor = ElectronicProcessor::new().unwrap();
    processor.load_bytes(&[0xAA; 256], &[0xBB; 128]).unwrap();

    let info = processor.execution_info().unwrap();
    assert_eq!(info.bytecode_size, 256);
    assert_eq!(info.data_size, 128);
    assert!(!info.halted);
    assert!(!info.has_error);
}

#[test]
fn test_electronic_processor_implements_processor_trait() {
    let processor = ElectronicProcessor::new().unwrap();
    assert_eq!(processor.layers(), &[5]);
    assert_eq!(processor.version(), "2026.1.11");
}

#[test]
fn test_electronic_processor_version() {
    let processor = ElectronicProcessor::new().unwrap();
    assert_eq!(processor.version(), "2026.1.11");
}

#[test]
fn test_processor_configuration_default() {
    let config = ElectronicConfig::default();
    assert_eq!(config.heap_size, 65536);
    assert_eq!(config.stack_size, 1024);
    assert!(config.enable_gpu);
    assert!(config.enable_npu);
    assert_eq!(config.max_cycles, 1_000_000);
}

#[test]
fn test_processor_state_ready() {
    let mut processor = ElectronicProcessor::new().unwrap();
    
    let state = processor.state().unwrap();
    assert!(!state.is_ready());

    processor.load_bytes(&[0xFF; 10], &[]).unwrap();
    let state = processor.state().unwrap();
    assert!(state.is_ready());

    processor.reset().unwrap();
    let state = processor.state().unwrap();
    assert!(!state.is_ready());
}
