//! # 🦾 sil-actuator — L6 Atuador/Motor
//!
//! Camada de atuação implementando o trait `Actuator` do SIL core.
//! Gerencia servos, motores DC e outros atuadores físicos, convertendo
//! comandos de alto nível em controle de hardware.
//!
//! ## Arquitetura
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │         Actuator Layer (L6)             │
//! │  ┌──────────────┐  ┌─────────────────┐  │
//! │  │ ServoActuator│  │  MotorActuator  │  │
//! │  │  (0-180°)    │  │   (-100..100%)  │  │
//! │  └──────────────┘  └─────────────────┘  │
//! │         ↓                   ↓            │
//! │  ┌──────────────────────────────────┐   │
//! │  │    Actuator Trait (L6)           │   │
//! │  │  send(), status(), emergency()   │   │
//! │  └──────────────────────────────────┘   │
//! └─────────────────────────────────────────┘
//!                   ↓
//!          Hardware (PWM, GPIO)
//! ```
//!
//! ## Componentes
//!
//! ### ServoActuator
//!
//! Controla servomotores (0-180°) com:
//! - Controle de posição angular
//! - Limites configuráveis
//! - Conversão para largura de pulso PWM
//! - Contador de movimentos
//!
//! ### MotorActuator
//!
//! Controla motores DC com:
//! - Controle bidirecional (-100% a +100%)
//! - Simulação de corrente
//! - Limite de corrente configurável
//! - Inversão de direção
//!
//! ## Exemplo de Uso
//!
//! ```rust
//! use sil_actuator::{ServoActuator, MotorActuator};
//! use sil_actuator::types::{ServoPosition, MotorSpeed};
//! use sil_core::traits::Actuator;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Criar servo
//! let mut servo = ServoActuator::named("gripper", 1)?;
//! let position = ServoPosition::new(90.0)?;
//! servo.send(position)?;
//!
//! // Criar motor
//! let mut motor = MotorActuator::named("wheel-left", 2)?;
//! let speed = MotorSpeed::new(75.0)?;
//! motor.send(speed)?;
//!
//! // Parada de emergência
//! motor.emergency_stop()?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Traits Implementados
//!
//! Ambos `ServoActuator` e `MotorActuator` implementam:
//!
//! - [`Actuator`](sil_core::traits::Actuator) - Interface de atuador
//! - [`SilComponent`](sil_core::traits::SilComponent) - Componente SIL base
//! - `Clone`, `Debug`, `Send`, `Sync`
//!
//! ## Características
//!
//! - **Thread-safe**: Usa `Arc<Mutex<_>>` internamente
//! - **Mock hardware**: Simulação para testes sem hardware real
//! - **Validação**: Limites de range e segurança
//! - **Telemetria**: Contadores de movimentos, tempo de operação, corrente
//! - **Calibração**: Suporte a procedimentos de calibração
//! - **Emergency stop**: Parada imediata de segurança
//!
//! ## Segurança
//!
//! - Validação de comandos antes de execução
//! - Limites configuráveis por atuador
//! - Estados de fault detectáveis
//! - Emergency stop sempre disponível
//! - Simulação de corrente e proteção contra sobrecarga

pub mod error;
pub mod types;
pub mod servo;
pub mod motor;

pub use error::{ActuatorError, ActuatorResult};
pub use types::{ServoPosition, MotorSpeed, MotorDirection, ActuatorCommand};
pub use servo::{ServoActuator, ServoConfig, ServoState};
pub use motor::{MotorActuator, MotorConfig, MotorState};

#[cfg(test)]
mod tests;
