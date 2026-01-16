//! # GPU-Accelerated Quantum Gates
//!
//! Extensão do sistema de gates quânticas para execução em GPU via WGSL.
//!
//! ## Arquitetura
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    GpuQuantumGate Trait                     │
//! │  ┌───────────────────────────────────────────────────────┐  │
//! │  │  gate_type() → u32         (tipo do gate)             │  │
//! │  │  to_matrix_uniform() → [f32; 8]  (matriz GPU)         │  │
//! │  │  apply_batch_cpu()   (fallback CPU)                   │  │
//! │  └───────────────────────────────────────────────────────┘  │
//! └─────────────────────────────────────────────────────────────┘
//!          │
//!          ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │                   quantum_gates.wgsl                        │
//! │  ┌───────────────────────────────────────────────────────┐  │
//! │  │  apply_gate kernel (switch por tipo)                  │  │
//! │  │  apply_matrix kernel (matriz customizada)             │  │
//! │  └───────────────────────────────────────────────────────┘  │
//! └─────────────────────────────────────────────────────────────┘
//! ```

use crate::gates::{
    Complex, Hadamard, Matrix2x2, PauliX, PauliY, PauliZ, Phase, QuantumGate, RotationX,
    RotationY, RotationZ, SGate, TGate,
};

/// Tipos de gate para o shader WGSL
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum GpuGateType {
    Hadamard = 0,
    PauliX = 1,
    PauliY = 2,
    PauliZ = 3,
    RotationX = 4,
    RotationY = 5,
    RotationZ = 6,
    Phase = 7,
    SGate = 8,
    TGate = 9,
    Custom = 255,
}

/// Uniform buffer para parâmetros do gate
#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct GateParamsUniform {
    pub num_states: u32,
    pub gate_type: u32,
    pub theta: f32,
    pub phi: f32,
}

/// Uniform buffer para matriz customizada
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct GateMatrixUniform {
    /// m00 real, m00 imag, m01 real, m01 imag
    pub m00_re: f32,
    pub m00_im: f32,
    pub m01_re: f32,
    pub m01_im: f32,
    /// m10 real, m10 imag, m11 real, m11 imag
    pub m10_re: f32,
    pub m10_im: f32,
    pub m11_re: f32,
    pub m11_im: f32,
}

impl Default for GateMatrixUniform {
    fn default() -> Self {
        // Identidade por padrão
        Self {
            m00_re: 1.0,
            m00_im: 0.0,
            m01_re: 0.0,
            m01_im: 0.0,
            m10_re: 0.0,
            m10_im: 0.0,
            m11_re: 1.0,
            m11_im: 0.0,
        }
    }
}

impl From<Matrix2x2> for GateMatrixUniform {
    fn from(m: Matrix2x2) -> Self {
        let [[a, b], [c, d]] = m.elements;
        Self {
            m00_re: a.re as f32,
            m00_im: a.im as f32,
            m01_re: b.re as f32,
            m01_im: b.im as f32,
            m10_re: c.re as f32,
            m10_im: c.im as f32,
            m11_re: d.re as f32,
            m11_im: d.im as f32,
        }
    }
}

/// Estado quântico em formato GPU-compatível
///
/// Layout: [α_re, α_im, β_re, β_im] para cada estado
#[derive(Clone, Debug, Default)]
pub struct GpuQuantumState {
    /// Amplitude α (|0⟩)
    pub alpha: (f32, f32),
    /// Amplitude β (|1⟩)
    pub beta: (f32, f32),
}

impl GpuQuantumState {
    /// Cria estado |0⟩
    pub fn zero() -> Self {
        Self {
            alpha: (1.0, 0.0),
            beta: (0.0, 0.0),
        }
    }

    /// Cria estado |1⟩
    pub fn one() -> Self {
        Self {
            alpha: (0.0, 0.0),
            beta: (1.0, 0.0),
        }
    }

    /// Cria estado |+⟩ = (|0⟩ + |1⟩)/√2
    pub fn plus() -> Self {
        let sqrt2_inv = std::f32::consts::FRAC_1_SQRT_2;
        Self {
            alpha: (sqrt2_inv, 0.0),
            beta: (sqrt2_inv, 0.0),
        }
    }

    /// Cria estado |-⟩ = (|0⟩ - |1⟩)/√2
    pub fn minus() -> Self {
        let sqrt2_inv = std::f32::consts::FRAC_1_SQRT_2;
        Self {
            alpha: (sqrt2_inv, 0.0),
            beta: (-sqrt2_inv, 0.0),
        }
    }

    /// Converte para array f32 para GPU
    pub fn to_floats(&self) -> [f32; 4] {
        [self.alpha.0, self.alpha.1, self.beta.0, self.beta.1]
    }

    /// Cria a partir de array f32
    pub fn from_floats(data: [f32; 4]) -> Self {
        Self {
            alpha: (data[0], data[1]),
            beta: (data[2], data[3]),
        }
    }

    /// Probabilidade de medir |0⟩
    pub fn prob_zero(&self) -> f32 {
        self.alpha.0 * self.alpha.0 + self.alpha.1 * self.alpha.1
    }

    /// Probabilidade de medir |1⟩
    pub fn prob_one(&self) -> f32 {
        self.beta.0 * self.beta.0 + self.beta.1 * self.beta.1
    }

    /// Verifica normalização
    pub fn is_normalized(&self, epsilon: f32) -> bool {
        let total = self.prob_zero() + self.prob_one();
        (total - 1.0).abs() < epsilon
    }
}

/// Trait para gates quânticas com aceleração GPU
///
/// Estende QuantumGate com métodos específicos para execução em GPU
/// via shaders WGSL.
pub trait GpuQuantumGate: QuantumGate {
    /// Tipo do gate para o shader (switch statement)
    fn gate_type(&self) -> GpuGateType;

    /// Parâmetro theta (para gates de rotação)
    fn theta(&self) -> f32 {
        0.0
    }

    /// Parâmetro phi (reservado para extensões)
    fn phi(&self) -> f32 {
        0.0
    }

    /// Converte para uniform de parâmetros
    fn to_params_uniform(&self, num_states: u32) -> GateParamsUniform {
        GateParamsUniform {
            num_states,
            gate_type: self.gate_type() as u32,
            theta: self.theta(),
            phi: self.phi(),
        }
    }

    /// Converte matriz para uniform GPU
    fn to_matrix_uniform(&self) -> GateMatrixUniform {
        self.matrix().into()
    }

    /// Aplica gate em batch (fallback CPU)
    fn apply_batch_cpu(&self, states: &[GpuQuantumState]) -> Vec<GpuQuantumState> {
        states
            .iter()
            .map(|state| {
                let input = [
                    Complex::new(state.alpha.0 as f64, state.alpha.1 as f64),
                    Complex::new(state.beta.0 as f64, state.beta.1 as f64),
                ];
                let output = self.matrix().apply(input);
                GpuQuantumState {
                    alpha: (output[0].re as f32, output[0].im as f32),
                    beta: (output[1].re as f32, output[1].im as f32),
                }
            })
            .collect()
    }

}

/// Funções utilitárias para conversão de estados
pub mod gpu_utils {
    use super::GpuQuantumState;

    /// Converte estados para buffer f32 para GPU
    pub fn states_to_buffer(states: &[GpuQuantumState]) -> Vec<f32> {
        let mut buffer = Vec::with_capacity(states.len() * 4);
        for state in states {
            buffer.extend_from_slice(&state.to_floats());
        }
        buffer
    }

    /// Converte buffer f32 de volta para estados
    pub fn buffer_to_states(buffer: &[f32]) -> Vec<GpuQuantumState> {
        buffer
            .chunks_exact(4)
            .map(|chunk| GpuQuantumState::from_floats([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect()
    }
}

// ─────────────────────────────────────────────────────────────────────────
//  Implementações para gates existentes
// ─────────────────────────────────────────────────────────────────────────

impl GpuQuantumGate for Hadamard {
    fn gate_type(&self) -> GpuGateType {
        GpuGateType::Hadamard
    }
}

impl GpuQuantumGate for PauliX {
    fn gate_type(&self) -> GpuGateType {
        GpuGateType::PauliX
    }
}

impl GpuQuantumGate for PauliY {
    fn gate_type(&self) -> GpuGateType {
        GpuGateType::PauliY
    }
}

impl GpuQuantumGate for PauliZ {
    fn gate_type(&self) -> GpuGateType {
        GpuGateType::PauliZ
    }
}

impl GpuQuantumGate for RotationX {
    fn gate_type(&self) -> GpuGateType {
        GpuGateType::RotationX
    }

    fn theta(&self) -> f32 {
        self.theta as f32
    }
}

impl GpuQuantumGate for RotationY {
    fn gate_type(&self) -> GpuGateType {
        GpuGateType::RotationY
    }

    fn theta(&self) -> f32 {
        self.theta as f32
    }
}

impl GpuQuantumGate for RotationZ {
    fn gate_type(&self) -> GpuGateType {
        GpuGateType::RotationZ
    }

    fn theta(&self) -> f32 {
        self.theta as f32
    }
}

impl GpuQuantumGate for Phase {
    fn gate_type(&self) -> GpuGateType {
        GpuGateType::Phase
    }

    fn theta(&self) -> f32 {
        self.phi as f32
    }
}

impl GpuQuantumGate for SGate {
    fn gate_type(&self) -> GpuGateType {
        GpuGateType::SGate
    }
}

impl GpuQuantumGate for TGate {
    fn gate_type(&self) -> GpuGateType {
        GpuGateType::TGate
    }
}

// ─────────────────────────────────────────────────────────────────────────
//  Gate customizada com matriz arbitrária
// ─────────────────────────────────────────────────────────────────────────

/// Gate customizada definida por matriz 2x2
#[derive(Clone, Debug)]
pub struct CustomGate {
    name: String,
    matrix: Matrix2x2,
}

impl CustomGate {
    /// Cria gate customizada
    pub fn new(name: impl Into<String>, matrix: Matrix2x2) -> Self {
        Self {
            name: name.into(),
            matrix,
        }
    }
}

impl QuantumGate for CustomGate {
    fn name(&self) -> &'static str {
        // Leak do nome para retornar &'static str
        // Em produção, usar enum ou string pool
        Box::leak(self.name.clone().into_boxed_str())
    }

    fn matrix(&self) -> Matrix2x2 {
        self.matrix
    }
}

impl GpuQuantumGate for CustomGate {
    fn gate_type(&self) -> GpuGateType {
        GpuGateType::Custom
    }
}

// ─────────────────────────────────────────────────────────────────────────
//  Circuito quântico para execução em batch
// ─────────────────────────────────────────────────────────────────────────

/// Circuito quântico composto de múltiplos gates
#[derive(Default)]
pub struct QuantumCircuit {
    gates: Vec<Box<dyn GpuQuantumGate>>,
}

impl QuantumCircuit {
    /// Cria circuito vazio
    pub fn new() -> Self {
        Self::default()
    }

    /// Adiciona gate ao circuito
    pub fn add<G: GpuQuantumGate + 'static>(&mut self, gate: G) -> &mut Self {
        self.gates.push(Box::new(gate));
        self
    }

    /// Retorna número de gates
    pub fn len(&self) -> usize {
        self.gates.len()
    }

    /// Verifica se circuito está vazio
    pub fn is_empty(&self) -> bool {
        self.gates.is_empty()
    }

    /// Retorna referência aos gates
    pub fn gates(&self) -> &[Box<dyn GpuQuantumGate>] {
        &self.gates
    }

    /// Executa circuito em CPU (fallback)
    pub fn execute_cpu(&self, states: &[GpuQuantumState]) -> Vec<GpuQuantumState> {
        let mut current = states.to_vec();
        for gate in &self.gates {
            current = gate.apply_batch_cpu(&current);
        }
        current
    }

    /// Calcula matriz composta do circuito
    pub fn composed_matrix(&self) -> Option<Matrix2x2> {
        if self.gates.is_empty() {
            return None;
        }

        let mut result = self.gates[0].matrix();
        for gate in &self.gates[1..] {
            result = gate.matrix().mul(&result);
        }
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_gpu_state_zero() {
        let state = GpuQuantumState::zero();
        assert!((state.prob_zero() - 1.0).abs() < 1e-6);
        assert!(state.prob_one().abs() < 1e-6);
    }

    #[test]
    fn test_gpu_state_plus() {
        let state = GpuQuantumState::plus();
        assert!((state.prob_zero() - 0.5).abs() < 1e-6);
        assert!((state.prob_one() - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_hadamard_cpu() {
        let h = Hadamard;
        let states = vec![GpuQuantumState::zero()];
        let result = h.apply_batch_cpu(&states);

        assert!((result[0].prob_zero() - 0.5).abs() < 1e-6);
        assert!((result[0].prob_one() - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_pauli_x_cpu() {
        let x = PauliX;
        let states = vec![GpuQuantumState::zero()];
        let result = x.apply_batch_cpu(&states);

        assert!(result[0].prob_zero().abs() < 1e-6);
        assert!((result[0].prob_one() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_circuit_cpu() {
        let mut circuit = QuantumCircuit::new();
        circuit.add(Hadamard).add(PauliZ).add(Hadamard);

        // H Z H = X
        let states = vec![GpuQuantumState::zero()];
        let result = circuit.execute_cpu(&states);

        // Deve ser |1⟩
        assert!(result[0].prob_zero().abs() < 1e-6);
        assert!((result[0].prob_one() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_gate_type_mapping() {
        assert_eq!(Hadamard.gate_type() as u32, 0);
        assert_eq!(PauliX.gate_type() as u32, 1);
        assert_eq!(PauliY.gate_type() as u32, 2);
        assert_eq!(PauliZ.gate_type() as u32, 3);
    }

    #[test]
    fn test_rotation_theta() {
        let rx = RotationX::new(std::f64::consts::PI / 2.0);
        assert!((rx.theta() - PI / 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_states_buffer_roundtrip() {
        use super::gpu_utils::{states_to_buffer, buffer_to_states};

        let states = vec![
            GpuQuantumState::zero(),
            GpuQuantumState::one(),
            GpuQuantumState::plus(),
        ];

        let buffer = states_to_buffer(&states);
        let recovered = buffer_to_states(&buffer);

        assert_eq!(states.len(), recovered.len());
        for (orig, rec) in states.iter().zip(recovered.iter()) {
            assert!((orig.alpha.0 - rec.alpha.0).abs() < 1e-6);
            assert!((orig.beta.0 - rec.beta.0).abs() < 1e-6);
        }
    }
}
