//! # 👁️ Perception — Transformações de Percepção L(0-4)
//!
//! Camadas sensoriais: Fotônico, Acústico, Olfativo, Gustativo, Dérmico.
//!
//! ## Pattern: Observer
//!
//! Sensores populam as camadas de percepção.

use crate::state::{ByteSil, SilState, layers};
use super::SilTransform;

/// Trait para sensores que populam L(0-4)
pub trait SilSensor: Send + Sync {
    /// Captura dados sensoriais para as 5 camadas de percepção
    fn sense(&self) -> [ByteSil; 5];
    
    /// Nome do sensor
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

/// Transformação que aplica dados de sensor ao estado
pub struct SensorTransform<S: SilSensor> {
    sensor: S,
}

impl<S: SilSensor> SensorTransform<S> {
    pub fn new(sensor: S) -> Self {
        Self { sensor }
    }
}

impl<S: SilSensor> SilTransform for SensorTransform<S> {
    fn transform(&self, state: &SilState) -> SilState {
        let sensed = self.sensor.sense();
        
        state
            .with_layer(layers::PHOTONIC, sensed[0])
            .with_layer(layers::ACOUSTIC, sensed[1])
            .with_layer(layers::OLFACTORY, sensed[2])
            .with_layer(layers::GUSTATORY, sensed[3])
            .with_layer(layers::DERMIC, sensed[4])
    }
    
    fn name(&self) -> &'static str {
        "SensorTransform"
    }
}

// =============================================================================
// Sensores Básicos
// =============================================================================

/// Sensor nulo (retorna NULL em todas as camadas)
#[derive(Debug, Clone, Copy, Default)]
pub struct NullSensor;

impl SilSensor for NullSensor {
    fn sense(&self) -> [ByteSil; 5] {
        [ByteSil::NULL; 5]
    }
    
    fn name(&self) -> &'static str {
        "NullSensor"
    }
}

/// Sensor neutro (retorna ONE em todas as camadas)
#[derive(Debug, Clone, Copy, Default)]
pub struct NeutralSensor;

impl SilSensor for NeutralSensor {
    fn sense(&self) -> [ByteSil; 5] {
        [ByteSil::ONE; 5]
    }
    
    fn name(&self) -> &'static str {
        "NeutralSensor"
    }
}

/// Sensor constante (retorna valor fixo)
#[derive(Debug, Clone, Copy)]
pub struct ConstantSensor {
    pub values: [ByteSil; 5],
}

impl ConstantSensor {
    pub fn new(values: [ByteSil; 5]) -> Self {
        Self { values }
    }
    
    pub fn uniform(value: ByteSil) -> Self {
        Self { values: [value; 5] }
    }
}

impl SilSensor for ConstantSensor {
    fn sense(&self) -> [ByteSil; 5] {
        self.values
    }
    
    fn name(&self) -> &'static str {
        "ConstantSensor"
    }
}

// =============================================================================
// Transformações específicas de percepção
// =============================================================================

/// Amplifica camadas de percepção
#[derive(Debug, Clone, Copy)]
pub struct PerceptionAmplify(pub i8);

impl SilTransform for PerceptionAmplify {
    fn transform(&self, state: &SilState) -> SilState {
        let mut layers = state.layers;
        
        for i in 0..5 {
            let new_rho = (layers[i].rho as i16 + self.0 as i16)
                .clamp(-8, 7) as i8;
            layers[i].rho = new_rho;
        }
        
        SilState::from_layers(layers)
    }
    
    fn name(&self) -> &'static str {
        "PerceptionAmplify"
    }
}

/// Normaliza percepção (ρ médio → 0)
#[derive(Debug, Clone, Copy, Default)]
pub struct PerceptionNormalize;

impl SilTransform for PerceptionNormalize {
    fn transform(&self, state: &SilState) -> SilState {
        let perception = state.perception();
        
        // Média de ρ
        let avg_rho: i16 = perception.iter()
            .map(|b| b.rho as i16)
            .sum::<i16>() / 5;
        
        let mut layers = state.layers;
        for i in 0..5 {
            let new_rho = (layers[i].rho as i16 - avg_rho)
                .clamp(-8, 7) as i8;
            layers[i].rho = new_rho;
        }
        
        SilState::from_layers(layers)
    }
    
    fn name(&self) -> &'static str {
        "PerceptionNormalize"
    }
}

// =============================================================================
// Testes
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_null_sensor() {
        let sensor = NullSensor;
        let sensed = sensor.sense();
        
        for b in &sensed {
            assert!(b.is_null());
        }
    }
    
    #[test]
    fn test_sensor_transform() {
        let state = SilState::vacuum();
        let transform = SensorTransform::new(NeutralSensor);
        
        let result = transform.transform(&state);
        
        // Percepção deve ser ONE
        for i in 0..5 {
            assert_eq!(result.layers[i], ByteSil::ONE);
        }
        
        // Resto continua vácuo
        for i in 5..16 {
            assert!(result.layers[i].is_null());
        }
    }
    
    #[test]
    fn test_perception_amplify() {
        let state = SilState::neutral();
        let amplified = PerceptionAmplify(2).transform(&state);
        
        // Percepção amplificada
        for i in 0..5 {
            assert_eq!(amplified.layers[i].rho, 2);
        }
        
        // Resto inalterado
        for i in 5..16 {
            assert_eq!(amplified.layers[i].rho, 0);
        }
    }
}
