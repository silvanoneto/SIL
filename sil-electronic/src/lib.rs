//! # ⚡ sil-electronic — L5 Processamento
//!
//! Camada eletrônica implementando o trait `Processor` através de wrapper
//! do Virtual Sil Processor (VSP). Gerencia bytecode, pipeline, e execução
//! de instruções em múltiplas arquiteturas (CPU/GPU/NPU).
//!
//! ## Arquitetura
//!
//! ```text
//! ┌─────────────────────────────────┐
//! │     ElectronicProcessor         │ (Processor + SilComponent)
//! │  ┌───────────────────────────┐  │
//! │  │    VSP Instance           │  │
//! │  │  (16 registradores +      │  │
//! │  │   Stack + Heap)           │  │
//! │  └───────────────────────────┘  │
//! │  ┌───────────────────────────┐  │
//! │  │  ProcessorState           │  │
//! │  │  (bytecode, pc, cycles)   │  │
//! │  └───────────────────────────┘  │
//! └─────────────────────────────────┘
//! ```
//!
//! ## Exemplo
//!
//! ```ignore
//! use sil_electronic::ElectronicProcessor;
//! use sil_core::prelude::*;
//!
//! // Criar processador
//! let mut proc = ElectronicProcessor::new()?;
//!
//! // Carregar bytecode
//! proc.load_silc("program.silc")?;
//!
//! // Executar
//! let state = proc.process(&input_state)?;
//! ```

pub mod error;
pub mod processor;
pub mod state;

pub use error::{ElectronicError, ElectronicResult};
pub use processor::{ElectronicProcessor, ElectronicConfig};
pub use state::{ProcessorState, ExecutionInfo};

#[cfg(test)]
mod tests;
