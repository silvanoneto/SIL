//! Differential privacy for federated learning

#[derive(Debug, Clone)]
pub struct DifferentialPrivacy {
    pub epsilon: f32,
    pub delta: f32,
}

impl DifferentialPrivacy {
    pub fn new(epsilon: f32, delta: f32) -> Self {
        Self { epsilon, delta }
    }

    pub fn add_noise(&self, _data: &[f32]) -> Vec<f32> {
        // TODO: Add Laplace/Gaussian noise
        vec![]
    }
}
