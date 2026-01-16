//! # Quantum GPU Bridge — Integration sil-quantum ↔ sil-core
//!
//! Bridge que conecta o sistema de gates quânticas do sil-quantum
//! com a execução GPU do sil-core.
//!
//! ## Arquitetura
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                        QuantumGpuBridge                             │
//! │  ┌───────────────────────────────────────────────────────────────┐  │
//! │  │  sil-quantum                    sil-core                     │  │
//! │  │  QuantumGate trait     →        GpuQuantumState              │  │
//! │  │  Matrix2x2             →        GateMatrix uniform           │  │
//! │  │  QuantumCircuit        →        GPU pipeline execution       │  │
//! │  └───────────────────────────────────────────────────────────────┘  │
//! └─────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Exemplo
//!
//! ```ignore
//! use sil_core::processors::quantum_bridge::QuantumGpuBridge;
//! use sil_quantum::{Hadamard, PauliX, QuantumCircuit};
//!
//! let bridge = QuantumGpuBridge::new()?;
//!
//! // Executar gate único
//! let states = vec![GpuQuantumState::zero(); 1000];
//! let results = bridge.apply_gate(&Hadamard, &states).await?;
//!
//! // Executar circuito
//! let mut circuit = QuantumCircuit::new();
//! circuit.add(Hadamard).add(PauliX).add(Hadamard);
//! let results = bridge.execute_circuit(&circuit, &states).await?;
//! ```

use std::sync::Arc;
use thiserror::Error;

#[cfg(feature = "gpu")]
use super::gpu::{
    GpuContext,
    quantum::{QuantumGpuExecutor, GpuQuantumState, GateMatrix, gate_types},
};

/// Erros do bridge quântico
#[derive(Debug, Error)]
pub enum QuantumBridgeError {
    #[error("GPU not available")]
    GpuNotAvailable,

    #[error("GPU error: {0}")]
    GpuError(String),

    #[error("Invalid gate type: {0}")]
    InvalidGateType(String),

    #[error("Empty circuit")]
    EmptyCircuit,

    #[error("State conversion error: {0}")]
    ConversionError(String),
}

pub type QuantumBridgeResult<T> = Result<T, QuantumBridgeError>;

/// Bridge entre sil-quantum e sil-core GPU
#[cfg(feature = "gpu")]
pub struct QuantumGpuBridge {
    gpu_ctx: Arc<GpuContext>,
    executor: QuantumGpuExecutor,
}

#[cfg(feature = "gpu")]
impl QuantumGpuBridge {
    /// Cria novo bridge
    pub fn new() -> QuantumBridgeResult<Self> {
        if !GpuContext::is_available() {
            return Err(QuantumBridgeError::GpuNotAvailable);
        }

        let gpu_ctx = Arc::new(
            GpuContext::new_sync()
                .map_err(|e| QuantumBridgeError::GpuError(e.to_string()))?,
        );

        let executor = QuantumGpuExecutor::new(&gpu_ctx)
            .map_err(|e| QuantumBridgeError::GpuError(e.to_string()))?;

        Ok(Self { gpu_ctx, executor })
    }

    /// Cria bridge a partir de contexto existente
    pub fn with_context(gpu_ctx: Arc<GpuContext>) -> QuantumBridgeResult<Self> {
        let executor = QuantumGpuExecutor::new(&gpu_ctx)
            .map_err(|e| QuantumBridgeError::GpuError(e.to_string()))?;

        Ok(Self { gpu_ctx, executor })
    }

    /// Aplica Hadamard em batch
    pub async fn apply_hadamard(
        &self,
        states: &[GpuQuantumState],
    ) -> QuantumBridgeResult<Vec<GpuQuantumState>> {
        self.executor
            .apply_hadamard(&self.gpu_ctx, states)
            .await
            .map_err(|e| QuantumBridgeError::GpuError(e.to_string()))
    }

    /// Aplica Pauli-X em batch
    pub async fn apply_pauli_x(
        &self,
        states: &[GpuQuantumState],
    ) -> QuantumBridgeResult<Vec<GpuQuantumState>> {
        self.executor
            .apply_gate(&self.gpu_ctx, states, gate_types::PAULI_X, 0.0)
            .await
            .map_err(|e| QuantumBridgeError::GpuError(e.to_string()))
    }

    /// Aplica Pauli-Y em batch
    pub async fn apply_pauli_y(
        &self,
        states: &[GpuQuantumState],
    ) -> QuantumBridgeResult<Vec<GpuQuantumState>> {
        self.executor
            .apply_gate(&self.gpu_ctx, states, gate_types::PAULI_Y, 0.0)
            .await
            .map_err(|e| QuantumBridgeError::GpuError(e.to_string()))
    }

    /// Aplica Pauli-Z em batch
    pub async fn apply_pauli_z(
        &self,
        states: &[GpuQuantumState],
    ) -> QuantumBridgeResult<Vec<GpuQuantumState>> {
        self.executor
            .apply_gate(&self.gpu_ctx, states, gate_types::PAULI_Z, 0.0)
            .await
            .map_err(|e| QuantumBridgeError::GpuError(e.to_string()))
    }

    /// Aplica rotação X em batch
    pub async fn apply_rotation_x(
        &self,
        states: &[GpuQuantumState],
        theta: f32,
    ) -> QuantumBridgeResult<Vec<GpuQuantumState>> {
        self.executor
            .apply_gate(&self.gpu_ctx, states, gate_types::ROTATION_X, theta)
            .await
            .map_err(|e| QuantumBridgeError::GpuError(e.to_string()))
    }

    /// Aplica rotação Y em batch
    pub async fn apply_rotation_y(
        &self,
        states: &[GpuQuantumState],
        theta: f32,
    ) -> QuantumBridgeResult<Vec<GpuQuantumState>> {
        self.executor
            .apply_gate(&self.gpu_ctx, states, gate_types::ROTATION_Y, theta)
            .await
            .map_err(|e| QuantumBridgeError::GpuError(e.to_string()))
    }

    /// Aplica rotação Z em batch
    pub async fn apply_rotation_z(
        &self,
        states: &[GpuQuantumState],
        theta: f32,
    ) -> QuantumBridgeResult<Vec<GpuQuantumState>> {
        self.executor
            .apply_gate(&self.gpu_ctx, states, gate_types::ROTATION_Z, theta)
            .await
            .map_err(|e| QuantumBridgeError::GpuError(e.to_string()))
    }

    /// Aplica gate de fase em batch
    pub async fn apply_phase(
        &self,
        states: &[GpuQuantumState],
        phi: f32,
    ) -> QuantumBridgeResult<Vec<GpuQuantumState>> {
        self.executor
            .apply_gate(&self.gpu_ctx, states, gate_types::PHASE, phi)
            .await
            .map_err(|e| QuantumBridgeError::GpuError(e.to_string()))
    }

    /// Aplica S gate em batch
    pub async fn apply_s_gate(
        &self,
        states: &[GpuQuantumState],
    ) -> QuantumBridgeResult<Vec<GpuQuantumState>> {
        self.executor
            .apply_gate(&self.gpu_ctx, states, gate_types::S_GATE, 0.0)
            .await
            .map_err(|e| QuantumBridgeError::GpuError(e.to_string()))
    }

    /// Aplica T gate em batch
    pub async fn apply_t_gate(
        &self,
        states: &[GpuQuantumState],
    ) -> QuantumBridgeResult<Vec<GpuQuantumState>> {
        self.executor
            .apply_gate(&self.gpu_ctx, states, gate_types::T_GATE, 0.0)
            .await
            .map_err(|e| QuantumBridgeError::GpuError(e.to_string()))
    }

    /// Aplica matriz 2x2 customizada em batch
    pub async fn apply_matrix(
        &self,
        states: &[GpuQuantumState],
        matrix: GateMatrix,
    ) -> QuantumBridgeResult<Vec<GpuQuantumState>> {
        self.executor
            .apply_matrix(&self.gpu_ctx, states, matrix)
            .await
            .map_err(|e| QuantumBridgeError::GpuError(e.to_string()))
    }

    /// Executa sequência de gates (circuito simples)
    ///
    /// Cada elemento é (gate_type, theta)
    pub async fn execute_gate_sequence(
        &self,
        states: &[GpuQuantumState],
        gates: &[(u32, f32)],
    ) -> QuantumBridgeResult<Vec<GpuQuantumState>> {
        if gates.is_empty() {
            return Err(QuantumBridgeError::EmptyCircuit);
        }

        let mut current = states.to_vec();

        for (gate_type, theta) in gates {
            current = self
                .executor
                .apply_gate(&self.gpu_ctx, &current, *gate_type, *theta)
                .await
                .map_err(|e| QuantumBridgeError::GpuError(e.to_string()))?;
        }

        Ok(current)
    }

    /// Executa circuito usando matrizes compostas (otimizado)
    ///
    /// Calcula M = G_n * G_{n-1} * ... * G_1 e aplica uma vez só
    pub async fn execute_circuit_optimized(
        &self,
        states: &[GpuQuantumState],
        matrices: &[GateMatrix],
    ) -> QuantumBridgeResult<Vec<GpuQuantumState>> {
        if matrices.is_empty() {
            return Err(QuantumBridgeError::EmptyCircuit);
        }

        // Se apenas uma matriz, aplica diretamente
        if matrices.len() == 1 {
            return self.apply_matrix(states, matrices[0]).await;
        }

        // Compõe matrizes: M = G_n * G_{n-1} * ... * G_1
        let composed = compose_matrices(matrices);

        self.apply_matrix(states, composed).await
    }

    /// Retorna referência ao contexto GPU
    pub fn gpu_context(&self) -> &Arc<GpuContext> {
        &self.gpu_ctx
    }
}

/// Compõe múltiplas matrizes 2x2 em uma única matriz
#[cfg(feature = "gpu")]
fn compose_matrices(matrices: &[GateMatrix]) -> GateMatrix {
    if matrices.is_empty() {
        return GateMatrix::default(); // Identidade
    }

    let mut result = matrices[0];

    for m in &matrices[1..] {
        result = multiply_matrices(&result, m);
    }

    result
}

/// Multiplica duas matrizes 2x2 complexas
#[cfg(feature = "gpu")]
fn multiply_matrices(a: &GateMatrix, b: &GateMatrix) -> GateMatrix {
    // (a*b)[i,j] = sum_k a[i,k] * b[k,j]

    // m00 = a00*b00 + a01*b10
    let m00_re = (a.m00_re * b.m00_re - a.m00_im * b.m00_im)
        + (a.m01_re * b.m10_re - a.m01_im * b.m10_im);
    let m00_im = (a.m00_re * b.m00_im + a.m00_im * b.m00_re)
        + (a.m01_re * b.m10_im + a.m01_im * b.m10_re);

    // m01 = a00*b01 + a01*b11
    let m01_re = (a.m00_re * b.m01_re - a.m00_im * b.m01_im)
        + (a.m01_re * b.m11_re - a.m01_im * b.m11_im);
    let m01_im = (a.m00_re * b.m01_im + a.m00_im * b.m01_re)
        + (a.m01_re * b.m11_im + a.m01_im * b.m11_re);

    // m10 = a10*b00 + a11*b10
    let m10_re = (a.m10_re * b.m00_re - a.m10_im * b.m00_im)
        + (a.m11_re * b.m10_re - a.m11_im * b.m10_im);
    let m10_im = (a.m10_re * b.m00_im + a.m10_im * b.m00_re)
        + (a.m11_re * b.m10_im + a.m11_im * b.m10_re);

    // m11 = a10*b01 + a11*b11
    let m11_re = (a.m10_re * b.m01_re - a.m10_im * b.m01_im)
        + (a.m11_re * b.m11_re - a.m11_im * b.m11_im);
    let m11_im = (a.m10_re * b.m01_im + a.m10_im * b.m01_re)
        + (a.m11_re * b.m11_im + a.m11_im * b.m11_re);

    GateMatrix {
        m00_re,
        m00_im,
        m01_re,
        m01_im,
        m10_re,
        m10_im,
        m11_re,
        m11_im,
    }
}

/// Helper para criar GateMatrix a partir de valores conhecidos
#[cfg(feature = "gpu")]
pub mod matrices {
    use super::GateMatrix;

    /// Matriz Hadamard
    pub fn hadamard() -> GateMatrix {
        let h = std::f32::consts::FRAC_1_SQRT_2;
        GateMatrix {
            m00_re: h,
            m00_im: 0.0,
            m01_re: h,
            m01_im: 0.0,
            m10_re: h,
            m10_im: 0.0,
            m11_re: -h,
            m11_im: 0.0,
        }
    }

    /// Matriz Pauli-X
    pub fn pauli_x() -> GateMatrix {
        GateMatrix {
            m00_re: 0.0,
            m00_im: 0.0,
            m01_re: 1.0,
            m01_im: 0.0,
            m10_re: 1.0,
            m10_im: 0.0,
            m11_re: 0.0,
            m11_im: 0.0,
        }
    }

    /// Matriz Pauli-Y
    pub fn pauli_y() -> GateMatrix {
        GateMatrix {
            m00_re: 0.0,
            m00_im: 0.0,
            m01_re: 0.0,
            m01_im: -1.0,
            m10_re: 0.0,
            m10_im: 1.0,
            m11_re: 0.0,
            m11_im: 0.0,
        }
    }

    /// Matriz Pauli-Z
    pub fn pauli_z() -> GateMatrix {
        GateMatrix {
            m00_re: 1.0,
            m00_im: 0.0,
            m01_re: 0.0,
            m01_im: 0.0,
            m10_re: 0.0,
            m10_im: 0.0,
            m11_re: -1.0,
            m11_im: 0.0,
        }
    }

    /// Matriz de rotação X
    pub fn rotation_x(theta: f32) -> GateMatrix {
        let c = (theta * 0.5).cos();
        let s = (theta * 0.5).sin();
        GateMatrix {
            m00_re: c,
            m00_im: 0.0,
            m01_re: 0.0,
            m01_im: -s,
            m10_re: 0.0,
            m10_im: -s,
            m11_re: c,
            m11_im: 0.0,
        }
    }

    /// Matriz de rotação Y
    pub fn rotation_y(theta: f32) -> GateMatrix {
        let c = (theta * 0.5).cos();
        let s = (theta * 0.5).sin();
        GateMatrix {
            m00_re: c,
            m00_im: 0.0,
            m01_re: -s,
            m01_im: 0.0,
            m10_re: s,
            m10_im: 0.0,
            m11_re: c,
            m11_im: 0.0,
        }
    }

    /// Matriz de rotação Z
    pub fn rotation_z(theta: f32) -> GateMatrix {
        let half = theta * 0.5;
        GateMatrix {
            m00_re: (-half).cos(),
            m00_im: (-half).sin(),
            m01_re: 0.0,
            m01_im: 0.0,
            m10_re: 0.0,
            m10_im: 0.0,
            m11_re: half.cos(),
            m11_im: half.sin(),
        }
    }

    /// Matriz de fase
    pub fn phase(phi: f32) -> GateMatrix {
        GateMatrix {
            m00_re: 1.0,
            m00_im: 0.0,
            m01_re: 0.0,
            m01_im: 0.0,
            m10_re: 0.0,
            m10_im: 0.0,
            m11_re: phi.cos(),
            m11_im: phi.sin(),
        }
    }

    /// Matriz S (√Z)
    pub fn s_gate() -> GateMatrix {
        GateMatrix {
            m00_re: 1.0,
            m00_im: 0.0,
            m01_re: 0.0,
            m01_im: 0.0,
            m10_re: 0.0,
            m10_im: 0.0,
            m11_re: 0.0,
            m11_im: 1.0,
        }
    }

    /// Matriz T (π/8)
    pub fn t_gate() -> GateMatrix {
        let h = std::f32::consts::FRAC_1_SQRT_2;
        GateMatrix {
            m00_re: 1.0,
            m00_im: 0.0,
            m01_re: 0.0,
            m01_im: 0.0,
            m10_re: 0.0,
            m10_im: 0.0,
            m11_re: h,
            m11_im: h,
        }
    }

    /// Identidade
    pub fn identity() -> GateMatrix {
        GateMatrix::default()
    }
}

#[cfg(all(test, feature = "gpu"))]
mod tests {
    use super::*;
    use super::matrices::*;

    #[test]
    fn test_matrix_composition_identity() {
        let i = identity();
        let h = hadamard();

        let composed = multiply_matrices(&i, &h);

        // I * H = H
        assert!((composed.m00_re - h.m00_re).abs() < 1e-6);
        assert!((composed.m01_re - h.m01_re).abs() < 1e-6);
    }

    #[test]
    fn test_hzh_equals_x() {
        // H Z H = X
        let h = hadamard();
        let z = pauli_z();
        let x = pauli_x();

        let hz = multiply_matrices(&h, &z);
        let hzh = multiply_matrices(&hz, &h);

        // Verificar que HZH ≈ X
        assert!((hzh.m00_re - x.m00_re).abs() < 1e-6);
        assert!((hzh.m01_re - x.m01_re).abs() < 1e-6);
        assert!((hzh.m10_re - x.m10_re).abs() < 1e-6);
        assert!((hzh.m11_re - x.m11_re).abs() < 1e-6);
    }

    #[test]
    fn test_compose_multiple() {
        let matrices = vec![hadamard(), pauli_z(), hadamard()];
        let composed = compose_matrices(&matrices);

        // Deve ser ≈ X
        let x = pauli_x();
        assert!((composed.m01_re - x.m01_re).abs() < 1e-6);
        assert!((composed.m10_re - x.m10_re).abs() < 1e-6);
    }
}
