//! # 🔗 Interaction — Transformações de Interação L(8-A)
//!
//! Camadas de comunicação: Cibernético, Geopolítico, Cosmopolítico.
//!
//! ## Pattern: Mediator
//!
//! Mediadores negociam entre estados locais e remotos.

use crate::state::{ByteSil, SilState, layers};
use super::SilTransform;

/// Trait para mediadores de interação
pub trait SilMediator: Send + Sync {
    /// Negocia entre estado local e remoto
    fn negotiate(&self, local: &SilState, remote: &SilState) -> [ByteSil; 3];
    
    /// Nome do mediador
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

/// Transformação que aplica mediação com estado remoto
pub struct MediatorTransform<M: SilMediator> {
    mediator: M,
    remote: SilState,
}

impl<M: SilMediator> MediatorTransform<M> {
    pub fn new(mediator: M, remote: SilState) -> Self {
        Self { mediator, remote }
    }
    
    pub fn with_remote(&self, remote: SilState) -> Self {
        Self {
            mediator: unsafe { std::ptr::read(&self.mediator) },
            remote,
        }
    }
}

impl<M: SilMediator + Clone> MediatorTransform<M> {
    pub fn update_remote(&self, remote: SilState) -> Self {
        Self {
            mediator: self.mediator.clone(),
            remote,
        }
    }
}

impl<M: SilMediator> SilTransform for MediatorTransform<M> {
    fn transform(&self, state: &SilState) -> SilState {
        let negotiated = self.mediator.negotiate(state, &self.remote);
        
        state
            .with_layer(layers::CYBERNETIC, negotiated[0])
            .with_layer(layers::GEOPOLITICAL, negotiated[1])
            .with_layer(layers::COSMOPOLITICAL, negotiated[2])
    }
    
    fn name(&self) -> &'static str {
        "MediatorTransform"
    }
}

// =============================================================================
// Mediadores Básicos
// =============================================================================

/// Mediador de consenso: calcula diferença e média
#[derive(Debug, Clone, Copy)]
pub struct ConsensusMediator {
    /// Peso do estado local (0.0 = remoto, 1.0 = local)
    pub local_weight: f64,
}

impl ConsensusMediator {
    pub fn new(local_weight: f64) -> Self {
        Self { local_weight: local_weight.clamp(0.0, 1.0) }
    }
    
    pub fn balanced() -> Self {
        Self { local_weight: 0.5 }
    }
}

impl Default for ConsensusMediator {
    fn default() -> Self {
        Self::balanced()
    }
}

impl SilMediator for ConsensusMediator {
    fn negotiate(&self, local: &SilState, remote: &SilState) -> [ByteSil; 3] {
        // L8: Feedback (diferença entre estados)
        let l8 = local.layers[layers::CYBERNETIC]
            .xor(&remote.layers[layers::CYBERNETIC]);
        
        // L9: Soberania (quem tem prioridade?)
        let local_strength = local.layers[layers::GEOPOLITICAL].norm();
        let remote_strength = remote.layers[layers::GEOPOLITICAL].norm();
        
        let l9 = if (local_strength as f64 * self.local_weight) 
                  >= (remote_strength as f64 * (1.0 - self.local_weight)) {
            local.layers[layers::GEOPOLITICAL]
        } else {
            remote.layers[layers::GEOPOLITICAL]
        };
        
        // LA: Ética (média ponderada)
        let local_cosmo = local.layers[layers::COSMOPOLITICAL];
        let remote_cosmo = remote.layers[layers::COSMOPOLITICAL];
        
        let w = self.local_weight;
        let rho = ((local_cosmo.rho as f64 * w) + (remote_cosmo.rho as f64 * (1.0 - w)))
            .round() as i8;
        let theta = ((local_cosmo.theta as f64 * w) + (remote_cosmo.theta as f64 * (1.0 - w)))
            .round() as u8;
        
        let la = ByteSil::new(rho, theta);
        
        [l8, l9, la]
    }
    
    fn name(&self) -> &'static str {
        "ConsensusMediator"
    }
}

/// Mediador local-first: sempre prioriza estado local
#[derive(Debug, Clone, Copy, Default)]
pub struct LocalFirstMediator;

impl SilMediator for LocalFirstMediator {
    fn negotiate(&self, local: &SilState, _remote: &SilState) -> [ByteSil; 3] {
        [
            local.layers[layers::CYBERNETIC],
            local.layers[layers::GEOPOLITICAL],
            local.layers[layers::COSMOPOLITICAL],
        ]
    }
    
    fn name(&self) -> &'static str {
        "LocalFirstMediator"
    }
}

/// Mediador remote-first: sempre prioriza estado remoto
#[derive(Debug, Clone, Copy, Default)]
pub struct RemoteFirstMediator;

impl SilMediator for RemoteFirstMediator {
    fn negotiate(&self, _local: &SilState, remote: &SilState) -> [ByteSil; 3] {
        [
            remote.layers[layers::CYBERNETIC],
            remote.layers[layers::GEOPOLITICAL],
            remote.layers[layers::COSMOPOLITICAL],
        ]
    }
    
    fn name(&self) -> &'static str {
        "RemoteFirstMediator"
    }
}

// =============================================================================
// Transformações específicas de interação (sem estado remoto)
// =============================================================================

/// Feedback interno: L8 = XOR(L5, L6, L7)
#[derive(Debug, Clone, Copy, Default)]
pub struct InternalFeedback;

impl SilTransform for InternalFeedback {
    fn transform(&self, state: &SilState) -> SilState {
        let processing = state.processing();
        let feedback = processing.iter()
            .fold(ByteSil::NULL, |a, b| a.xor(b));
        
        state.with_layer(layers::CYBERNETIC, feedback)
    }
    
    fn name(&self) -> &'static str {
        "InternalFeedback"
    }
}

/// Amplifica camadas de interação
#[derive(Debug, Clone, Copy)]
pub struct InteractionAmplify(pub i8);

impl SilTransform for InteractionAmplify {
    fn transform(&self, state: &SilState) -> SilState {
        let mut layers = state.layers;
        
        for i in 8..11 {
            let new_rho = (layers[i].rho as i16 + self.0 as i16)
                .clamp(-8, 7) as i8;
            layers[i].rho = new_rho;
        }
        
        SilState::from_layers(layers)
    }
    
    fn name(&self) -> &'static str {
        "InteractionAmplify"
    }
}

// =============================================================================
// Testes
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_consensus_mediator_balanced() {
        let local = SilState::neutral()
            .with_layer(layers::GEOPOLITICAL, ByteSil::new(5, 0));
        let remote = SilState::neutral()
            .with_layer(layers::GEOPOLITICAL, ByteSil::new(3, 0));
        
        let mediator = ConsensusMediator::balanced();
        let [_l8, l9, _la] = mediator.negotiate(&local, &remote);
        
        // Local tem mais força (5 > 3), mesmo com peso balanceado
        assert_eq!(l9.rho, 5);
    }
    
    #[test]
    fn test_local_first_mediator() {
        let local = SilState::neutral()
            .with_layer(layers::CYBERNETIC, ByteSil::new(4, 8));
        let remote = SilState::maximum();
        
        let mediator = LocalFirstMediator;
        let [l8, _l9, _la] = mediator.negotiate(&local, &remote);
        
        assert_eq!(l8, ByteSil::new(4, 8));
    }
    
    #[test]
    fn test_internal_feedback() {
        let state = SilState::vacuum()
            .with_layer(layers::ELECTRONIC, ByteSil::new(2, 4))
            .with_layer(layers::PSYCHOMOTOR, ByteSil::new(1, 2))
            .with_layer(layers::ENVIRONMENTAL, ByteSil::new(3, 6));
        
        let result = InternalFeedback.transform(&state);
        
        // L8 = fold(NULL, XOR(L5, L6, L7))
        // = NULL ^ L5 ^ L6 ^ L7
        // = (-8,0) ^ (2,4) ^ (1,2) ^ (3,6)
        let expected = ByteSil::NULL
            .xor(&ByteSil::new(2, 4))
            .xor(&ByteSil::new(1, 2))
            .xor(&ByteSil::new(3, 6));
        
        assert_eq!(result.layers[layers::CYBERNETIC], expected);
    }
}
