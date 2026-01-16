//! # 🔗 Mediator Pattern — Interação
//!
//! Mediadores coordenam comunicação entre múltiplos nós.
//!
//! ## Uso
//!
//! ```
//! use sil_core::patterns::mediator::*;
//! use sil_core::state::SilState;
//!
//! // Criar mediador de consenso
//! let mediator = ConsensusMediatorPattern::new(0.5);
//!
//! let local = SilState::neutral();
//! let remote = SilState::maximum();
//! let result = mediator.mediate(&local, &remote);
//! ```

use crate::state::{ByteSil, SilState};

/// **SilMediator** — Trait para mediação L(8-A)
///
/// Segue Pattern 3 (Mediator) do SIL_CODE.md
///
/// # Exemplo
///
/// ```
/// use sil_core::patterns::mediator::SilMediator;
/// use sil_core::state::{SilState, ByteSil};
///
/// struct ConsensusMediator { threshold: f64 }
///
/// impl SilMediator for ConsensusMediator {
///     fn negotiate(&self, local: &SilState, remote: &SilState) -> [ByteSil; 3] {
///         [
///             local.layers[8].xor(&remote.layers[8]),  // L8: Feedback
///             local.layers[9],                           // L9: Soberania (local)
///             ByteSil::from_complex(                     // LA: Ética (média)
///                 (local.layers[0xA].to_complex() + remote.layers[0xA].to_complex()) / 2.0
///             ),
///         ]
///     }
/// }
/// ```
pub trait SilMediator: Send + Sync {
    /// Negocia entre estados local e remoto → L(8-A)
    fn negotiate(&self, local: &SilState, remote: &SilState) -> [ByteSil; 3];
}

/// Trait para mediadores genéricos — compatibilidade com código existente
pub trait Mediator: Send + Sync {
    /// Media entre estado local e remoto
    fn mediate(&self, local: &SilState, remote: &SilState) -> SilState;
    
    /// Nome do mediador
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

/// Mediador de consenso com peso
#[derive(Debug, Clone, Copy)]
pub struct ConsensusMediatorPattern {
    /// Peso do local (0.0 = remoto, 1.0 = local)
    local_weight: f64,
}

impl ConsensusMediatorPattern {
    pub fn new(local_weight: f64) -> Self {
        Self { local_weight: local_weight.clamp(0.0, 1.0) }
    }
    
    pub fn local_only() -> Self {
        Self { local_weight: 1.0 }
    }
    
    pub fn remote_only() -> Self {
        Self { local_weight: 0.0 }
    }
    
    pub fn balanced() -> Self {
        Self { local_weight: 0.5 }
    }
}

impl Default for ConsensusMediatorPattern {
    fn default() -> Self {
        Self::balanced()
    }
}

impl Mediator for ConsensusMediatorPattern {
    fn mediate(&self, local: &SilState, remote: &SilState) -> SilState {
        let w = self.local_weight;
        
        // Interpola todas as camadas
        let mut layers = [ByteSil::NULL; 16];
        
        for i in 0..16 {
            let l = &local.layers[i];
            let r = &remote.layers[i];
            
            let rho = (l.rho as f64 * w + r.rho as f64 * (1.0 - w))
                .round() as i8;
            let theta = (l.theta as f64 * w + r.theta as f64 * (1.0 - w))
                .round() as u8;
            
            layers[i] = ByteSil::new(rho, theta);
        }
        
        SilState::from_layers(layers)
    }
    
    fn name(&self) -> &'static str {
        "ConsensusMediatorPattern"
    }
}

/// Mediador de votação: cada camada vence quem tem maior norma
#[derive(Debug, Clone, Copy, Default)]
pub struct VotingMediator;

impl Mediator for VotingMediator {
    fn mediate(&self, local: &SilState, remote: &SilState) -> SilState {
        let mut layers = [ByteSil::NULL; 16];
        
        for i in 0..16 {
            layers[i] = if local.layers[i].norm() >= remote.layers[i].norm() {
                local.layers[i]
            } else {
                remote.layers[i]
            };
        }
        
        SilState::from_layers(layers)
    }
    
    fn name(&self) -> &'static str {
        "VotingMediator"
    }
}

/// Mediador de XOR: combina via XOR (detecta diferenças)
#[derive(Debug, Clone, Copy, Default)]
pub struct XorMediator;

impl Mediator for XorMediator {
    fn mediate(&self, local: &SilState, remote: &SilState) -> SilState {
        local.xor(remote)
    }
    
    fn name(&self) -> &'static str {
        "XorMediator"
    }
}

/// Hub central: coordena múltiplos participantes
pub struct MediatorHub {
    participants: Vec<SilState>,
    mediator: Box<dyn Mediator>,
}

impl MediatorHub {
    pub fn new(mediator: Box<dyn Mediator>) -> Self {
        Self {
            participants: Vec::new(),
            mediator,
        }
    }
    
    pub fn add_participant(&mut self, state: SilState) {
        self.participants.push(state);
    }
    
    pub fn clear(&mut self) {
        self.participants.clear();
    }
    
    /// Resolve: combina todos os participantes
    pub fn resolve(&self) -> SilState {
        if self.participants.is_empty() {
            return SilState::vacuum();
        }
        
        self.participants.iter()
            .skip(1)
            .fold(self.participants[0], |acc, p| {
                self.mediator.mediate(&acc, p)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_consensus_mediator_balanced() {
        let local = SilState::vacuum();
        let remote = SilState::maximum();
        
        let mediator = ConsensusMediatorPattern::balanced();
        let result = mediator.mediate(&local, &remote);
        
        // Deve ser aproximadamente meio termo
        // local.rho = -8, remote.rho = 7, média ≈ -0.5 → 0
        assert!(result.layers[0].rho > -5 && result.layers[0].rho < 5);
    }
    
    #[test]
    fn test_voting_mediator() {
        let local = SilState::vacuum()
            .with_layer(0, ByteSil::new(5, 0));
        let remote = SilState::vacuum()
            .with_layer(0, ByteSil::new(3, 0));
        
        let mediator = VotingMediator;
        let result = mediator.mediate(&local, &remote);
        
        // Local ganha em L0 (5 > 3)
        assert_eq!(result.layers[0].rho, 5);
    }
    
    #[test]
    fn test_mediator_hub() {
        let mut hub = MediatorHub::new(Box::new(ConsensusMediatorPattern::balanced()));
        
        hub.add_participant(SilState::neutral());
        hub.add_participant(SilState::maximum());
        hub.add_participant(SilState::vacuum());
        
        let result = hub.resolve();
        
        // Deve ter combinado todos
        assert!(!result.layers[0].is_null() || result.layers[0].rho > -8);
    }
}
