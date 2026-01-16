//! # 👁️ Observer Pattern — Percepção
//!
//! Sensores que observam o ambiente e populam camadas de percepção.
//!
//! ## Uso
//!
//! ```
//! use sil_core::patterns::observer::*;
//! use sil_core::state::SilState;
//!
//! // Criar observador composto
//! let observer = CompositeObserver::new(vec![
//!     Box::new(LightObserver::new(0.8)),
//!     Box::new(SoundObserver::new(0.5)),
//! ]);
//!
//! let state = observer.observe(&SilState::neutral());
//! ```

use crate::state::{ByteSil, SilState, layers};
use crate::transforms::SilTransform;

/// **SilSensor** — Trait para sensores que populam L(0-4)
///
/// Segue Pattern 1 (Observer) do SIL_CODE.md
///
/// # Exemplo
///
/// ```
/// use sil_core::patterns::observer::SilSensor;
/// use sil_core::state::ByteSil;
///
/// struct CameraSensor;
///
/// impl SilSensor for CameraSensor {
///     fn sense(&self) -> [ByteSil; 5] {
///         [
///             ByteSil::new(3, 0),  // L0 Fotônico
///             ByteSil::NULL,       // L1 (não usado)
///             ByteSil::NULL,       // L2 (não usado)
///             ByteSil::NULL,       // L3 (não usado)
///             ByteSil::new(2, 4),  // L4 Dérmico (textura)
///         ]
///     }
/// }
/// ```
pub trait SilSensor: Send + Sync {
    /// Retorna array de 5 ByteSil para camadas L(0-4)
    fn sense(&self) -> [ByteSil; 5];
}

/// Trait para observadores (sensors) — compatibilidade com código existente
pub trait Observer: Send + Sync {
    /// Observa e retorna novo estado com percepção atualizada
    fn observe(&self, state: &SilState) -> SilState;
    
    /// Camada que este observer popula
    fn layer(&self) -> usize;
}

/// Observador de luz (L0 - Fotônico)
#[derive(Debug, Clone)]
pub struct LightObserver {
    /// Intensidade atual (0.0 a 1.0)
    intensity: f64,
}

impl LightObserver {
    pub fn new(intensity: f64) -> Self {
        Self { intensity: intensity.clamp(0.0, 1.0) }
    }
    
    pub fn set_intensity(&mut self, intensity: f64) {
        self.intensity = intensity.clamp(0.0, 1.0);
    }
}

impl Observer for LightObserver {
    fn observe(&self, state: &SilState) -> SilState {
        // Mapeia intensidade para ρ: 0.0 → -8, 1.0 → 7
        let rho = ((self.intensity * 15.0) - 8.0).round() as i8;
        let byte = ByteSil::new(rho, 0);
        state.with_layer(layers::PHOTONIC, byte)
    }
    
    fn layer(&self) -> usize {
        layers::PHOTONIC
    }
}

/// Observador de som (L1 - Acústico)
#[derive(Debug, Clone)]
pub struct SoundObserver {
    /// Volume atual (0.0 a 1.0)
    volume: f64,
    /// Frequência dominante (fase)
    frequency_band: u8,
}

impl SoundObserver {
    pub fn new(volume: f64) -> Self {
        Self { 
            volume: volume.clamp(0.0, 1.0),
            frequency_band: 0,
        }
    }
    
    pub fn with_frequency(mut self, band: u8) -> Self {
        self.frequency_band = band % 16;
        self
    }
}

impl Observer for SoundObserver {
    fn observe(&self, state: &SilState) -> SilState {
        let rho = ((self.volume * 15.0) - 8.0).round() as i8;
        let byte = ByteSil::new(rho, self.frequency_band);
        state.with_layer(layers::ACOUSTIC, byte)
    }
    
    fn layer(&self) -> usize {
        layers::ACOUSTIC
    }
}

/// Observador de temperatura/toque (L4 - Dérmico)
#[derive(Debug, Clone)]
pub struct TouchObserver {
    /// Pressão (0.0 a 1.0)
    pressure: f64,
    /// Temperatura relativa (fase: 0=frio, 8=neutro, 15=quente)
    temperature: u8,
}

impl TouchObserver {
    pub fn new(pressure: f64, temperature: u8) -> Self {
        Self {
            pressure: pressure.clamp(0.0, 1.0),
            temperature: temperature % 16,
        }
    }
}

impl Observer for TouchObserver {
    fn observe(&self, state: &SilState) -> SilState {
        let rho = ((self.pressure * 15.0) - 8.0).round() as i8;
        let byte = ByteSil::new(rho, self.temperature);
        state.with_layer(layers::DERMIC, byte)
    }
    
    fn layer(&self) -> usize {
        layers::DERMIC
    }
}

/// Observador composto: combina múltiplos observers
pub struct CompositeObserver {
    observers: Vec<Box<dyn Observer>>,
}

impl CompositeObserver {
    pub fn new(observers: Vec<Box<dyn Observer>>) -> Self {
        Self { observers }
    }
    
    pub fn empty() -> Self {
        Self { observers: Vec::new() }
    }
    
    pub fn add(&mut self, observer: Box<dyn Observer>) {
        self.observers.push(observer);
    }
    
    /// Observa estado aplicando todos os observers
    pub fn observe(&self, state: &SilState) -> SilState {
        self.observers.iter().fold(*state, |s, o| o.observe(&s))
    }
}

impl SilTransform for CompositeObserver {
    fn transform(&self, state: &SilState) -> SilState {
        self.observe(state)
    }
    
    fn name(&self) -> &'static str {
        "CompositeObserver"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_light_observer() {
        let observer = LightObserver::new(1.0);
        let state = SilState::vacuum();
        let result = observer.observe(&state);
        
        assert_eq!(result.layers[layers::PHOTONIC].rho, 7);
    }
    
    #[test]
    fn test_composite_observer() {
        let composite = CompositeObserver::new(vec![
            Box::new(LightObserver::new(0.5)),
            Box::new(SoundObserver::new(0.5)),
        ]);
        
        let state = SilState::vacuum();
        let result = composite.transform(&state);
        
        // Ambas camadas devem ter sido modificadas
        assert!(!result.layers[layers::PHOTONIC].is_null());
        assert!(!result.layers[layers::ACOUSTIC].is_null());
    }
}
