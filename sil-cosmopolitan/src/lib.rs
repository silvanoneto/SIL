//! # sil-cosmopolitan — L(A) Cosmopolitico
//!
//! Camada de etica, especies e hospitalidade do SIL.
//!
//! ## Conceito
//!
//! L(A) e a camada de valores universais, que define:
//! - **EthicalMode**: 8 frameworks eticos (Deontologico a Cosmico)
//! - **AgentSpecies**: 6 tipos de agentes (Humano a Desconhecido)
//! - **HospitalityProtocol**: Protocolo de acolhimento do outro
//! - **MoralCircle**: Circulos de consideracao moral
//!
//! ## Filosofia
//!
//! > *"Sou porque somos"* — Ubuntu
//!
//! A camada cosmopolitica reconhece que a etica nao e universal,
//! mas contextual. Cada framework etico tem seu dominio de aplicacao:
//!
//! - **Deontologico**: Quando regras claras existem
//! - **Consequencialista**: Quando resultados importam
//! - **Virtue**: Quando carater e formacao importam
//! - **Care**: Quando relacoes sao prioritarias
//! - **Ubuntu**: Quando comunidade e central
//! - **Gaian**: Quando ecosistemas estao em jogo
//! - **Cosmic**: Quando escala interplanetaria

pub mod ethics;
pub mod species;
pub mod rights;
pub mod hospitality;
pub mod circles;
pub mod state;

pub use ethics::{EthicalMode, EthicalFramework};
pub use species::{AgentSpecies, AgentCapabilities};
pub use rights::{Right, Duty, RightsRegistry};
pub use hospitality::{HospitalityProtocol, HospitalityPhase, HospitalityResult, HospitalitySession};
pub use circles::{MoralCircle, CircleExpansion};
pub use state::CosmopoliticalState;
