//! # 🌊 Emergent Pattern — Auto-organização
//!
//! Padrões que emergem da interação entre componentes.
//!
//! ## Uso
//!
//! ```
//! use sil_core::patterns::emergent::*;
//! use sil_core::state::SilState;
//!
//! // Criar sistema emergente
//! let mut system = EmergentSystem::new(10);
//!
//! // Alimentar histórico
//! system.feed(SilState::neutral());
//! system.feed(SilState::maximum());
//!
//! // Detectar padrões
//! let patterns = system.detect();
//! ```

use crate::state::{ByteSil, SilState, layers};

/// **EmergenceDetector** — Trait para detecção de emergência L(B-C)
///
/// Segue Pattern 4 (Emergent) do SIL_CODE.md
///
/// # Exemplo
///
/// ```
/// use sil_core::patterns::emergent::EmergenceDetector;
/// use sil_core::state::{SilState, ByteSil};
///
/// struct TopologicalEmergence;
///
/// impl EmergenceDetector for TopologicalEmergence {
///     fn detect(&self, history: &[SilState]) -> [ByteSil; 2] {
///         // Calcular persistência topológica...
///         [
///             ByteSil::new(4, 8),  // LB: Sinérgico (complexidade)
///             ByteSil::new(2, 4),  // LC: Quântico (coerência)
///         ]
///     }
/// }
/// ```
pub trait EmergenceDetector: Send + Sync {
    /// Detecta padrões emergentes do histórico → L(B-C)
    fn detect(&self, history: &[SilState]) -> [ByteSil; 2];
}

/// Padrão emergente detectado
#[derive(Debug, Clone)]
pub struct EmergentPattern {
    /// Tipo do padrão
    pub kind: PatternKind,
    /// Força do padrão (0.0 a 1.0)
    pub strength: f64,
    /// Camadas afetadas
    pub affected_layers: Vec<usize>,
}

/// Tipos de padrões emergentes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternKind {
    /// Oscilação periódica
    Oscillation,
    /// Atrator estável
    Attractor,
    /// Transição de fase
    PhaseTransition,
    /// Sincronização
    Synchronization,
    /// Caos
    Chaos,
}

/// Sistema de detecção de emergência
pub struct EmergentSystem {
    history: Vec<SilState>,
    max_history: usize,
}

impl EmergentSystem {
    pub fn new(max_history: usize) -> Self {
        Self {
            history: Vec::with_capacity(max_history),
            max_history,
        }
    }
    
    /// Adiciona estado ao histórico
    pub fn feed(&mut self, state: SilState) {
        if self.history.len() >= self.max_history {
            self.history.remove(0);
        }
        self.history.push(state);
    }
    
    /// Limpa histórico
    pub fn clear(&mut self) {
        self.history.clear();
    }
    
    /// Tamanho do histórico
    pub fn len(&self) -> usize {
        self.history.len()
    }
    
    /// Histórico vazio?
    pub fn is_empty(&self) -> bool {
        self.history.is_empty()
    }
    
    /// Detecta padrões emergentes
    pub fn detect(&self) -> Vec<EmergentPattern> {
        let mut patterns = Vec::new();
        
        // Detecta sincronização (só precisa de 1 estado)
        if let Some(sync) = self.detect_synchronization() {
            patterns.push(sync);
        }
        
        // Detecções que precisam de histórico maior
        if self.history.len() < 2 {
            return patterns;
        }
        
        // Detecta oscilação (precisa de 4+ estados)
        if let Some(osc) = self.detect_oscillation() {
            patterns.push(osc);
        }
        
        // Detecta atrator (precisa de 2+ estados)
        if let Some(attr) = self.detect_attractor() {
            patterns.push(attr);
        }
        
        patterns
    }
    
    /// Detecta oscilação (alternância de valores)
    fn detect_oscillation(&self) -> Option<EmergentPattern> {
        if self.history.len() < 4 {
            return None;
        }
        
        // Verifica se há padrão A-B-A-B em alguma camada
        let mut oscillating_layers = Vec::new();
        
        for layer_idx in 0..16 {
            let values: Vec<_> = self.history.iter()
                .map(|s| s.layers[layer_idx].to_u8())
                .collect();
            
            // Verifica periodicidade
            let mut is_oscillating = true;
            for i in 2..values.len() {
                if values[i] != values[i - 2] {
                    is_oscillating = false;
                    break;
                }
            }
            
            if is_oscillating && values.len() >= 4 {
                oscillating_layers.push(layer_idx);
            }
        }
        
        if !oscillating_layers.is_empty() {
            Some(EmergentPattern {
                kind: PatternKind::Oscillation,
                strength: oscillating_layers.len() as f64 / 16.0,
                affected_layers: oscillating_layers,
            })
        } else {
            None
        }
    }
    
    /// Detecta atrator (convergência para valor estável)
    fn detect_attractor(&self) -> Option<EmergentPattern> {
        if self.history.len() < 3 {
            return None;
        }
        
        // Verifica se os últimos estados são iguais
        let last = self.history.last().unwrap();
        let second_last = &self.history[self.history.len() - 2];
        let third_last = &self.history[self.history.len() - 3];
        
        let mut stable_layers = Vec::new();
        
        for i in 0..16 {
            if last.layers[i] == second_last.layers[i] 
               && second_last.layers[i] == third_last.layers[i] {
                stable_layers.push(i);
            }
        }
        
        if stable_layers.len() >= 8 {
            Some(EmergentPattern {
                kind: PatternKind::Attractor,
                strength: stable_layers.len() as f64 / 16.0,
                affected_layers: stable_layers,
            })
        } else {
            None
        }
    }
    
    /// Detecta sincronização (camadas se alinhando)
    fn detect_synchronization(&self) -> Option<EmergentPattern> {
        if self.history.is_empty() {
            return None;
        }
        
        let last = self.history.last().unwrap();
        
        // Verifica se todas as fases (θ) são iguais
        let reference_theta = last.layers[0].theta;
        let synchronized: Vec<_> = (0..16)
            .filter(|&i| last.layers[i].theta == reference_theta)
            .collect();
        
        if synchronized.len() >= 12 {
            Some(EmergentPattern {
                kind: PatternKind::Synchronization,
                strength: synchronized.len() as f64 / 16.0,
                affected_layers: synchronized,
            })
        } else {
            None
        }
    }
    
    /// Calcula estado emergente (combinação do histórico)
    pub fn emergent_state(&self) -> SilState {
        if self.history.is_empty() {
            return SilState::vacuum();
        }
        
        // Média de todos os estados no histórico
        let mut sum_rho = [0i32; 16];
        let mut sum_theta = [0u32; 16];
        
        for state in &self.history {
            for i in 0..16 {
                sum_rho[i] += state.layers[i].rho as i32;
                sum_theta[i] += state.layers[i].theta as u32;
            }
        }
        
        let n = self.history.len() as i32;
        let mut layers = [ByteSil::NULL; 16];
        
        for i in 0..16 {
            layers[i] = ByteSil::new(
                (sum_rho[i] / n) as i8,
                (sum_theta[i] / n as u32) as u8,
            );
        }
        
        // Coloca resultado nas camadas de emergência
        let mut result = SilState::from_layers(layers);
        
        // LB: Sinérgico = XOR do histórico
        let synergy = self.history.iter()
            .map(|s| s.collapse(crate::state::CollapseStrategy::Xor))
            .fold(ByteSil::NULL, |a, b| a.xor(&b));
        result = result.with_layer(layers::SYNERGIC, synergy);
        
        // LC: Quântico = média das colapsadas
        let sum: num_complex::Complex<f64> = self.history.iter()
            .map(|s| s.collapse(crate::state::CollapseStrategy::Sum).to_complex())
            .sum();
        let quantum = ByteSil::from_complex(sum / self.history.len() as f64);
        result = result.with_layer(layers::QUANTUM, quantum);
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_emergent_system_new() {
        let system = EmergentSystem::new(10);
        assert!(system.is_empty());
    }
    
    #[test]
    fn test_emergent_system_feed() {
        let mut system = EmergentSystem::new(3);
        
        system.feed(SilState::neutral());
        system.feed(SilState::maximum());
        system.feed(SilState::vacuum());
        system.feed(SilState::neutral()); // Deve remover o primeiro
        
        assert_eq!(system.len(), 3);
    }
    
    #[test]
    fn test_detect_attractor() {
        let mut system = EmergentSystem::new(10);
        
        // Alimenta com estados iguais (atrator)
        let stable = SilState::neutral();
        for _ in 0..5 {
            system.feed(stable);
        }
        
        let patterns = system.detect();
        
        // Deve detectar atrator
        assert!(patterns.iter().any(|p| p.kind == PatternKind::Attractor));
    }
    
    #[test]
    fn test_detect_synchronization() {
        let mut system = EmergentSystem::new(10);
        
        // Estado com todas as fases iguais
        let synchronized = SilState::neutral(); // todos θ=0
        system.feed(synchronized);
        
        // Verificar o estado foi adicionado
        assert_eq!(system.len(), 1, "Sistema deve ter 1 estado");
        
        // Verificar que todas as fases são 0
        let last = system.history.last().unwrap();
        for i in 0..16 {
            assert_eq!(last.layers[i].theta, 0, "Layer {} deve ter theta=0", i);
        }
        
        let patterns = system.detect();
        
        // O threshold para sincronização é >= 12 camadas
        // neutral() tem todas 16 camadas com θ=0
        // Deve detectar sincronização
        assert!(
            patterns.iter().any(|p| p.kind == PatternKind::Synchronization),
            "Deveria detectar sincronização. Patterns encontrados: {:?}", patterns
        );
    }
    
    #[test]
    fn test_emergent_state() {
        let mut system = EmergentSystem::new(10);
        
        system.feed(SilState::vacuum());
        system.feed(SilState::maximum());
        
        let emergent = system.emergent_state();
        
        // Deve ser algo entre vacuum e maximum
        assert!(emergent.layers[0].rho > -8 && emergent.layers[0].rho < 7);
    }
}
