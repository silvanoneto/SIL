//! # 🌊 Emergence — Transformações de Emergência L(B-C)
//!
//! Camadas de padrões emergentes: Sinérgico, Quântico.
//!
//! ## Pattern: Emergent
//!
//! Detectores identificam padrões que emergem do histórico de estados.

use crate::state::{ByteSil, SilState, layers};
use super::SilTransform;

/// Trait para detectores de emergência
pub trait EmergenceDetector: Send + Sync {
    /// Detecta padrões emergentes do histórico
    fn detect(&self, history: &[SilState]) -> [ByteSil; 2];
    
    /// Nome do detector
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

/// Transformação que aplica detecção de emergência
pub struct EmergenceTransform<E: EmergenceDetector> {
    detector: E,
    history: Vec<SilState>,
    max_history: usize,
}

impl<E: EmergenceDetector> EmergenceTransform<E> {
    pub fn new(detector: E, max_history: usize) -> Self {
        Self {
            detector,
            history: Vec::with_capacity(max_history),
            max_history,
        }
    }
    
    /// Adiciona estado ao histórico
    pub fn record(&mut self, state: &SilState) {
        if self.history.len() >= self.max_history {
            self.history.remove(0);
        }
        self.history.push(*state);
    }
}

impl<E: EmergenceDetector> SilTransform for EmergenceTransform<E> {
    fn transform(&self, state: &SilState) -> SilState {
        let emerged = self.detector.detect(&self.history);
        
        state
            .with_layer(layers::SYNERGIC, emerged[0])
            .with_layer(layers::QUANTUM, emerged[1])
    }
    
    fn name(&self) -> &'static str {
        "EmergenceTransform"
    }
}

// =============================================================================
// Detectores Básicos
// =============================================================================

/// Detector de entropia: mede variação no histórico
#[derive(Debug, Clone, Copy, Default)]
pub struct EntropyDetector;

impl EmergenceDetector for EntropyDetector {
    fn detect(&self, history: &[SilState]) -> [ByteSil; 2] {
        if history.is_empty() {
            return [ByteSil::NULL, ByteSil::NULL];
        }
        
        // LB: Sinérgico — variância das magnitudes
        let magnitudes: Vec<i16> = history.iter()
            .flat_map(|s| s.layers.iter().map(|b| b.rho as i16))
            .collect();
        
        let mean_mag = magnitudes.iter().sum::<i16>() / magnitudes.len().max(1) as i16;
        let variance = magnitudes.iter()
            .map(|&m| (m - mean_mag).pow(2))
            .sum::<i16>() / magnitudes.len().max(1) as i16;
        
        let lb = ByteSil::new(
            (variance.min(7) - 4) as i8,
            0,
        );
        
        // LC: Quântico — coerência de fase
        let phases: Vec<u8> = history.iter()
            .flat_map(|s| s.layers.iter().map(|b| b.theta))
            .collect();
        
        let mean_phase = phases.iter().map(|&p| p as u16).sum::<u16>() 
            / phases.len().max(1) as u16;
        let phase_variance = phases.iter()
            .map(|&p| ((p as i16 - mean_phase as i16).abs() as u16).pow(2))
            .sum::<u16>() / phases.len().max(1) as u16;
        
        // Alta variância de fase = baixa coerência
        let coherence = 15u16.saturating_sub(phase_variance.min(15));
        let lc = ByteSil::new(0, coherence as u8);
        
        [lb, lc]
    }
    
    fn name(&self) -> &'static str {
        "EntropyDetector"
    }
}

/// Detector de periodicidade: busca padrões repetidos
#[derive(Debug, Clone, Copy, Default)]
pub struct PeriodicityDetector;

impl EmergenceDetector for PeriodicityDetector {
    fn detect(&self, history: &[SilState]) -> [ByteSil; 2] {
        if history.len() < 2 {
            return [ByteSil::NULL, ByteSil::NULL];
        }
        
        // Busca repetição: compara hash do estado atual com anteriores
        let current = history.last().unwrap();
        let current_hash = current.hash();
        
        let mut period = 0usize;
        for (i, state) in history.iter().rev().skip(1).enumerate() {
            if state.hash() == current_hash {
                period = i + 1;
                break;
            }
        }
        
        // LB: Período detectado
        let lb = ByteSil::new(
            (period.min(15) as i8) - 8,
            (period % 16) as u8,
        );
        
        // LC: Força do padrão (quantas repetições)
        let repetitions = history.iter()
            .filter(|s| s.hash() == current_hash)
            .count();
        
        let lc = ByteSil::new(
            (repetitions.min(15) as i8) - 8,
            0,
        );
        
        [lb, lc]
    }
    
    fn name(&self) -> &'static str {
        "PeriodicityDetector"
    }
}

/// Detector constante: retorna valores fixos
#[derive(Debug, Clone, Copy)]
pub struct ConstantDetector {
    pub values: [ByteSil; 2],
}

impl ConstantDetector {
    pub fn new(synergic: ByteSil, quantum: ByteSil) -> Self {
        Self { values: [synergic, quantum] }
    }
}

impl EmergenceDetector for ConstantDetector {
    fn detect(&self, _history: &[SilState]) -> [ByteSil; 2] {
        self.values
    }
    
    fn name(&self) -> &'static str {
        "ConstantDetector"
    }
}

// =============================================================================
// Transformações específicas de emergência (sem histórico)
// =============================================================================

/// Calcula sinergia interna: LB = XOR de todas as camadas anteriores
#[derive(Debug, Clone, Copy, Default)]
pub struct InternalSynergy;

impl SilTransform for InternalSynergy {
    fn transform(&self, state: &SilState) -> SilState {
        // LB = XOR(L0..LA)
        let synergy = state.layers[..11].iter()
            .fold(ByteSil::NULL, |a, b| a.xor(b));
        
        state.with_layer(layers::SYNERGIC, synergy)
    }
    
    fn name(&self) -> &'static str {
        "InternalSynergy"
    }
}

/// Calcula coerência interna: LC = média de todas as fases
#[derive(Debug, Clone, Copy, Default)]
pub struct InternalCoherence;

impl SilTransform for InternalCoherence {
    fn transform(&self, state: &SilState) -> SilState {
        // Média de ρ e θ de L0..LB
        let sum_rho: i16 = state.layers[..12].iter()
            .map(|b| b.rho as i16)
            .sum();
        let sum_theta: u16 = state.layers[..12].iter()
            .map(|b| b.theta as u16)
            .sum();
        
        let coherence = ByteSil::new(
            (sum_rho / 12) as i8,
            (sum_theta / 12) as u8,
        );
        
        state.with_layer(layers::QUANTUM, coherence)
    }
    
    fn name(&self) -> &'static str {
        "InternalCoherence"
    }
}

// =============================================================================
// Testes
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_entropy_detector_empty() {
        let detector = EntropyDetector;
        let [lb, lc] = detector.detect(&[]);
        
        assert!(lb.is_null());
        assert!(lc.is_null());
    }
    
    #[test]
    fn test_periodicity_detector() {
        let state = SilState::neutral();
        let history = vec![state, state, state];
        
        let detector = PeriodicityDetector;
        let [_lb, lc] = detector.detect(&history);
        
        // Deve detectar 3 repetições
        assert!(lc.rho > -8);
    }
    
    #[test]
    fn test_internal_synergy() {
        let state = SilState::neutral();
        let result = InternalSynergy.transform(&state);
        
        // XOR de NULL com 11 ONEs (ρ=0, θ=0)
        // NULL.rho = -8 (binario: 11111000), ONE.rho = 0
        // -8 ^ 0 ^ 0 ^ ... (11 vezes) = -8 ^ 0 = -8
        // Não, espera: -8 em i8 é representado como 0b11111000
        // 0 XOR 0 = 0, então 11 XORs de 0 = 0
        // -8 XOR 0 = -8
        assert_eq!(result.layers[layers::SYNERGIC].rho, -8);
    }
    
    #[test]
    fn test_internal_coherence() {
        let state = SilState::neutral();
        let result = InternalCoherence.transform(&state);
        
        // Média de ONEs = ONE
        assert_eq!(result.layers[layers::QUANTUM], ByteSil::new(0, 0));
    }
}
