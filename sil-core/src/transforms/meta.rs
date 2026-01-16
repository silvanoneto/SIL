//! # 🎛️ Meta — Transformações Meta L(D-F)
//!
//! Camadas de controle de fluxo: Superposição, Entanglement, Colapso.
//!
//! ## Pattern: Meta-Controller
//!
//! Controladores decidem fork, sync e collapse do ciclo.

use crate::state::{ByteSil, SilState, layers};
use super::SilTransform;

/// Ações de meta-controle
#[derive(Debug, Clone, PartialEq)]
pub enum MetaAction {
    /// Próximo ciclo normal
    Continue,
    /// Criar branches (superposição)
    Fork(Vec<SilState>),
    /// Juntar branches (entanglement)
    Sync(Vec<SilState>),
    /// Terminar (colapso)
    Collapse,
}

/// Trait para controladores meta
pub trait MetaController: Send + Sync {
    /// Decide ação baseada no estado
    fn control(&self, state: &SilState) -> MetaAction;
    
    /// Nome do controlador
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

/// Transformação que aplica controle meta
pub struct MetaTransform<C: MetaController> {
    controller: C,
}

impl<C: MetaController> MetaTransform<C> {
    pub fn new(controller: C) -> Self {
        Self { controller }
    }
    
    /// Obtém ação de controle sem modificar estado
    pub fn get_action(&self, state: &SilState) -> MetaAction {
        self.controller.control(state)
    }
}

impl<C: MetaController> SilTransform for MetaTransform<C> {
    fn transform(&self, state: &SilState) -> SilState {
        let action = self.controller.control(state);
        
        // Codifica ação nas camadas meta
        let (ld, le, lf) = match action {
            MetaAction::Continue => (
                ByteSil::ONE,           // LD: superposição normal
                ByteSil::NULL,          // LE: sem entanglement
                ByteSil::ONE,           // LF: continua
            ),
            MetaAction::Fork(ref branches) => (
                ByteSil::new(branches.len() as i8, 0), // LD: número de branches
                ByteSil::NULL,                          // LE: sem entanglement
                ByteSil::new(1, 4),                     // LF: fase π/2 = fork
            ),
            MetaAction::Sync(ref branches) => (
                ByteSil::NULL,                          // LD: colapsa superposição
                ByteSil::new(branches.len() as i8, 0), // LE: número de fontes
                ByteSil::new(1, 8),                     // LF: fase π = sync
            ),
            MetaAction::Collapse => (
                ByteSil::NULL,          // LD: sem superposição
                ByteSil::NULL,          // LE: sem entanglement
                ByteSil::NULL,          // LF: colapso (null = terminar)
            ),
        };
        
        state
            .with_layer(layers::SUPERPOSITION, ld)
            .with_layer(layers::ENTANGLEMENT, le)
            .with_layer(layers::COLLAPSE, lf)
    }
    
    fn name(&self) -> &'static str {
        "MetaTransform"
    }
}

// =============================================================================
// Controladores Básicos
// =============================================================================

/// Controlador que sempre continua
#[derive(Debug, Clone, Copy, Default)]
pub struct AlwaysContinue;

impl MetaController for AlwaysContinue {
    fn control(&self, _state: &SilState) -> MetaAction {
        MetaAction::Continue
    }
    
    fn name(&self) -> &'static str {
        "AlwaysContinue"
    }
}

/// Controlador que colapsa quando LF é nulo
#[derive(Debug, Clone, Copy, Default)]
pub struct CollapseOnNull;

impl MetaController for CollapseOnNull {
    fn control(&self, state: &SilState) -> MetaAction {
        if state.layers[layers::COLLAPSE].is_null() {
            MetaAction::Collapse
        } else {
            MetaAction::Continue
        }
    }
    
    fn name(&self) -> &'static str {
        "CollapseOnNull"
    }
}

/// Controlador adaptativo baseado em thresholds
#[derive(Debug, Clone, Copy)]
pub struct AdaptiveController {
    /// Threshold para fork (LD.norm() > threshold)
    pub fork_threshold: u8,
    /// Threshold para sync (LE.norm() > threshold)
    pub sync_threshold: u8,
    /// Threshold para collapse (LF.norm() < threshold)
    pub collapse_threshold: u8,
}

impl AdaptiveController {
    pub fn new(fork: u8, sync: u8, collapse: u8) -> Self {
        Self {
            fork_threshold: fork,
            sync_threshold: sync,
            collapse_threshold: collapse,
        }
    }
}

impl Default for AdaptiveController {
    fn default() -> Self {
        Self {
            fork_threshold: 12,
            sync_threshold: 12,
            collapse_threshold: 2,
        }
    }
}

impl MetaController for AdaptiveController {
    fn control(&self, state: &SilState) -> MetaAction {
        let ld = state.layers[layers::SUPERPOSITION];
        let le = state.layers[layers::ENTANGLEMENT];
        let lf = state.layers[layers::COLLAPSE];
        
        // Prioridade: Collapse > Sync > Fork > Continue
        if lf.is_null() || lf.norm() < self.collapse_threshold {
            MetaAction::Collapse
        } else if le.norm() > self.sync_threshold {
            // Alto entanglement → precisa sync
            // (branches seriam buscados externamente)
            MetaAction::Sync(Vec::new())
        } else if ld.norm() > self.fork_threshold {
            // Alta superposição → fork
            // (gera branches com variações de fase)
            let branches = (0..4).map(|i| {
                let mut s = *state;
                s.layers[layers::SUPERPOSITION].theta = (ld.theta + i * 4) % 16;
                s
            }).collect();
            MetaAction::Fork(branches)
        } else {
            MetaAction::Continue
        }
    }
    
    fn name(&self) -> &'static str {
        "AdaptiveController"
    }
}

/// Controlador por contador de ciclos
#[derive(Debug)]
pub struct CycleCounter {
    pub max_cycles: usize,
    current: std::sync::atomic::AtomicUsize,
}

impl Clone for CycleCounter {
    fn clone(&self) -> Self {
        Self {
            max_cycles: self.max_cycles,
            current: std::sync::atomic::AtomicUsize::new(
                self.current.load(std::sync::atomic::Ordering::SeqCst)
            ),
        }
    }
}

impl CycleCounter {
    pub fn new(max_cycles: usize) -> Self {
        Self {
            max_cycles,
            current: std::sync::atomic::AtomicUsize::new(0),
        }
    }
    
    pub fn reset(&self) {
        self.current.store(0, std::sync::atomic::Ordering::SeqCst);
    }
    
    pub fn current(&self) -> usize {
        self.current.load(std::sync::atomic::Ordering::SeqCst)
    }
}

impl MetaController for CycleCounter {
    fn control(&self, _state: &SilState) -> MetaAction {
        let cycle = self.current.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if cycle >= self.max_cycles {
            MetaAction::Collapse
        } else {
            MetaAction::Continue
        }
    }
    
    fn name(&self) -> &'static str {
        "CycleCounter"
    }
}

// =============================================================================
// Transformações específicas de meta (sem controlador)
// =============================================================================

/// Prepara para colapso: LF = média de L0..LE
#[derive(Debug, Clone, Copy, Default)]
pub struct PrepareCollapse;

impl SilTransform for PrepareCollapse {
    fn transform(&self, state: &SilState) -> SilState {
        // LF = média de todas as camadas anteriores
        let sum_rho: i16 = state.layers[..15].iter()
            .map(|b| b.rho as i16)
            .sum();
        let sum_theta: u16 = state.layers[..15].iter()
            .map(|b| b.theta as u16)
            .sum();
        
        let collapse = ByteSil::new(
            (sum_rho / 15) as i8,
            (sum_theta / 15) as u8,
        );
        
        state.with_layer(layers::COLLAPSE, collapse)
    }
    
    fn name(&self) -> &'static str {
        "PrepareCollapse"
    }
}

/// Marca para superposição: LD = dado valor
#[derive(Debug, Clone, Copy)]
pub struct SetSuperposition(pub ByteSil);

impl SilTransform for SetSuperposition {
    fn transform(&self, state: &SilState) -> SilState {
        state.with_layer(layers::SUPERPOSITION, self.0)
    }
    
    fn name(&self) -> &'static str {
        "SetSuperposition"
    }
}

/// Marca entanglement: LE = dado valor
#[derive(Debug, Clone, Copy)]
pub struct SetEntanglement(pub ByteSil);

impl SilTransform for SetEntanglement {
    fn transform(&self, state: &SilState) -> SilState {
        state.with_layer(layers::ENTANGLEMENT, self.0)
    }
    
    fn name(&self) -> &'static str {
        "SetEntanglement"
    }
}

// =============================================================================
// Testes
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_always_continue() {
        let controller = AlwaysContinue;
        let state = SilState::vacuum();
        
        assert_eq!(controller.control(&state), MetaAction::Continue);
    }
    
    #[test]
    fn test_collapse_on_null() {
        let controller = CollapseOnNull;
        
        // Estado com LF nulo
        let state_null = SilState::vacuum();
        assert_eq!(controller.control(&state_null), MetaAction::Collapse);
        
        // Estado com LF não-nulo
        let state_active = SilState::neutral();
        assert_eq!(controller.control(&state_active), MetaAction::Continue);
    }
    
    #[test]
    fn test_adaptive_controller() {
        let controller = AdaptiveController::default();
        
        // Estado normal → Continue
        let normal = SilState::neutral();
        assert_eq!(controller.control(&normal), MetaAction::Continue);
        
        // Estado com LF nulo → Collapse
        let collapsed = SilState::neutral()
            .with_layer(layers::COLLAPSE, ByteSil::NULL);
        assert_eq!(controller.control(&collapsed), MetaAction::Collapse);
    }
    
    #[test]
    fn test_cycle_counter() {
        let controller = CycleCounter::new(3);
        let state = SilState::neutral();
        
        assert_eq!(controller.control(&state), MetaAction::Continue);
        assert_eq!(controller.control(&state), MetaAction::Continue);
        assert_eq!(controller.control(&state), MetaAction::Continue);
        assert_eq!(controller.control(&state), MetaAction::Collapse);
    }
    
    #[test]
    fn test_prepare_collapse() {
        let state = SilState::neutral();
        let result = PrepareCollapse.transform(&state);
        
        // Média de ONEs = ONE-ish
        assert!(!result.layers[layers::COLLAPSE].is_null());
    }
}
