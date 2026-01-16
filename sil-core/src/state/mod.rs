//! # 🌀 State — Estado SIL
//!
//! Módulo central do padrão SIL: estado como vetor de 16 camadas.
//!
//! ## Estrutura
//!
//! - [`BitDeSil`]: Unidade multidimensional (7 faces) — bit reinterpretado
//! - [`ByteSil`]: Unidade fundamental (ρ, θ) — 8 bits
//! - [`SilState`]: Estado completo — 16 camadas × 8 bits = 128 bits
//!
//! ## Princípio
//!
//! > *"Estado é sagrado — nunca modifique in-place, sempre crie novo."*

mod bit;
mod byte_sil;
mod sil_state;
pub mod simd;

pub use bit::{BitDeSil, PHI, PHI_INV};
pub use byte_sil::ByteSil;
pub use sil_state::{CollapseStrategy, SilState};

/// Número de camadas do sistema SIL
pub const NUM_LAYERS: usize = 16;

/// Índices das camadas por grupo funcional
pub mod layers {
    //! Índices nomeados das 16 camadas SIL
    
    // ═══════════════════════════════════════════════════════════════════════
    // PERCEPÇÃO (L0-L4) — Sensores
    // ═══════════════════════════════════════════════════════════════════════
    
    /// L(0) Fotônico — Luz, visão
    pub const PHOTONIC: usize = 0x0;
    /// L(1) Acústico — Som, audição
    pub const ACOUSTIC: usize = 0x1;
    /// L(2) Olfativo — Cheiro
    pub const OLFACTORY: usize = 0x2;
    /// L(3) Gustativo — Sabor
    pub const GUSTATORY: usize = 0x3;
    /// L(4) Dérmico — Toque, temperatura
    pub const DERMIC: usize = 0x4;
    
    // ═══════════════════════════════════════════════════════════════════════
    // PROCESSAMENTO (L5-L7) — Computação local
    // ═══════════════════════════════════════════════════════════════════════
    
    /// L(5) Eletrônico — Hardware, circuitos
    pub const ELECTRONIC: usize = 0x5;
    /// L(6) Psicomotor — Movimento, ação
    pub const PSYCHOMOTOR: usize = 0x6;
    /// L(7) Ambiental — Contexto, ambiente
    pub const ENVIRONMENTAL: usize = 0x7;
    
    // ═══════════════════════════════════════════════════════════════════════
    // INTERAÇÃO (L8-LA) — Comunicação
    // ═══════════════════════════════════════════════════════════════════════
    
    /// L(8) Cibernético — Feedback, controle
    pub const CYBERNETIC: usize = 0x8;
    /// L(9) Geopolítico — Soberania, território
    pub const GEOPOLITICAL: usize = 0x9;
    /// L(A) Cosmopolítico — Ética, valores universais
    pub const COSMOPOLITICAL: usize = 0xA;
    
    // ═══════════════════════════════════════════════════════════════════════
    // EMERGÊNCIA (LB-LC) — Padrões
    // ═══════════════════════════════════════════════════════════════════════
    
    /// L(B) Sinérgico — Complexidade emergente
    pub const SYNERGIC: usize = 0xB;
    /// L(C) Quântico — Coerência
    pub const QUANTUM: usize = 0xC;
    
    // ═══════════════════════════════════════════════════════════════════════
    // META (LD-LF) — Controle de fluxo
    // ═══════════════════════════════════════════════════════════════════════
    
    /// L(D) Superposição — Estados paralelos
    pub const SUPERPOSITION: usize = 0xD;
    /// L(E) Entanglement — Correlação não-local
    pub const ENTANGLEMENT: usize = 0xE;
    /// L(F) Colapso — Decisão, medição
    pub const COLLAPSE: usize = 0xF;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_layer_count() {
        assert_eq!(NUM_LAYERS, 16);
        assert_eq!(NUM_LAYERS, 1 << 4); // 2⁴
    }
    
    #[test]
    fn test_phi_properties() {
        // φ² = φ + 1
        let phi_sq = PHI * PHI;
        let phi_plus_one = PHI + 1.0;
        assert!((phi_sq - phi_plus_one).abs() < 1e-10);
    }
}
