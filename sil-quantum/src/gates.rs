//! # Quantum Gates — Portas Quânticas para SIL
//!
//! Implementa portas quânticas padrão para manipulação de estados.
//!
//! ## Gates Implementadas
//!
//! - **Single-qubit**: H (Hadamard), X, Y, Z (Pauli), S, T (Phase)
//! - **Two-qubit**: CNOT, CZ, SWAP
//! - **Rotation**: Rx, Ry, Rz

use serde::{Deserialize, Serialize};
use std::f64::consts::{FRAC_1_SQRT_2, PI};
use std::fmt;

/// Regime quântico (interpretação theta de LC)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum QuantumRegime {
    /// Clássico (decoerência total)
    Classical = 0,
    /// Semiclássico (alguma coerência)
    Semiclassical = 2,
    /// Coerente (superposição estável)
    #[default]
    Coherent = 4,
    /// Emaranhado (correlações não-locais)
    Entangled = 6,
    /// Topológico (proteção topológica)
    Topological = 8,
    /// Adiabático (evolução lenta)
    Adiabatic = 10,
    /// Medição (colapso iminente)
    Measurement = 12,
    /// Unitário (evolução reversível)
    Unitary = 14,
}

impl QuantumRegime {
    /// Cria QuantumRegime a partir de theta (0-15)
    pub fn from_theta(theta: u8) -> Self {
        match theta & 0b1110 {
            0 => Self::Classical,
            2 => Self::Semiclassical,
            4 => Self::Coherent,
            6 => Self::Entangled,
            8 => Self::Topological,
            10 => Self::Adiabatic,
            12 => Self::Measurement,
            14 => Self::Unitary,
            _ => Self::Coherent,
        }
    }

    /// Converte para theta
    pub fn to_theta(self) -> u8 {
        self as u8
    }

    /// Verifica se preserva coerência
    pub fn is_coherent(&self) -> bool {
        !matches!(self, Self::Classical | Self::Measurement)
    }

    /// Verifica se é emaranhado
    pub fn is_entangled(&self) -> bool {
        matches!(self, Self::Entangled | Self::Topological)
    }

    /// Nível de quanticidade (0-7)
    pub fn quantum_level(&self) -> u8 {
        (self.to_theta() / 2) as u8
    }

    /// Nome descritivo
    pub fn name(&self) -> &'static str {
        match self {
            Self::Classical => "Classical",
            Self::Semiclassical => "Semiclassical",
            Self::Coherent => "Coherent",
            Self::Entangled => "Entangled",
            Self::Topological => "Topological",
            Self::Adiabatic => "Adiabatic",
            Self::Measurement => "Measurement",
            Self::Unitary => "Unitary",
        }
    }
}

impl fmt::Display for QuantumRegime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Matriz 2x2 complexa para gates single-qubit
#[derive(Clone, Copy, Debug)]
pub struct Matrix2x2 {
    /// Elementos: [[a, b], [c, d]]
    pub elements: [[Complex; 2]; 2],
}

/// Número complexo simples
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Complex {
    pub re: f64,
    pub im: f64,
}

impl Complex {
    /// Cria número complexo
    pub const fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    /// Zero complexo
    pub const ZERO: Self = Self { re: 0.0, im: 0.0 };

    /// Um complexo
    pub const ONE: Self = Self { re: 1.0, im: 0.0 };

    /// Unidade imaginária
    pub const I: Self = Self { re: 0.0, im: 1.0 };

    /// Conjugado
    pub fn conj(self) -> Self {
        Self { re: self.re, im: -self.im }
    }

    /// Módulo ao quadrado
    pub fn norm_sq(self) -> f64 {
        self.re * self.re + self.im * self.im
    }

    /// Módulo
    pub fn abs(self) -> f64 {
        self.norm_sq().sqrt()
    }

    /// Fase (argumento)
    pub fn arg(self) -> f64 {
        self.im.atan2(self.re)
    }

    /// Exponencial complexa: e^(i*theta)
    pub fn from_polar(theta: f64) -> Self {
        Self {
            re: theta.cos(),
            im: theta.sin(),
        }
    }

    /// Multiplicação
    pub fn mul(self, other: Self) -> Self {
        Self {
            re: self.re * other.re - self.im * other.im,
            im: self.re * other.im + self.im * other.re,
        }
    }

    /// Adição
    pub fn add(self, other: Self) -> Self {
        Self {
            re: self.re + other.re,
            im: self.im + other.im,
        }
    }

    /// Multiplicação por escalar
    pub fn scale(self, s: f64) -> Self {
        Self {
            re: self.re * s,
            im: self.im * s,
        }
    }
}

impl Matrix2x2 {
    /// Cria matriz identidade
    pub fn identity() -> Self {
        Self {
            elements: [
                [Complex::ONE, Complex::ZERO],
                [Complex::ZERO, Complex::ONE],
            ],
        }
    }

    /// Aplica gate a um estado [alpha, beta]
    pub fn apply(&self, state: [Complex; 2]) -> [Complex; 2] {
        let [alpha, beta] = state;
        let [[a, b], [c, d]] = self.elements;

        [
            a.mul(alpha).add(b.mul(beta)),
            c.mul(alpha).add(d.mul(beta)),
        ]
    }

    /// Multiplicação de matrizes
    pub fn mul(&self, other: &Matrix2x2) -> Matrix2x2 {
        let [[a, b], [c, d]] = self.elements;
        let [[e, f], [g, h]] = other.elements;

        Matrix2x2 {
            elements: [
                [
                    a.mul(e).add(b.mul(g)),
                    a.mul(f).add(b.mul(h)),
                ],
                [
                    c.mul(e).add(d.mul(g)),
                    c.mul(f).add(d.mul(h)),
                ],
            ],
        }
    }

    /// Transposta conjugada (dagger)
    pub fn dagger(&self) -> Matrix2x2 {
        let [[a, b], [c, d]] = self.elements;
        Matrix2x2 {
            elements: [
                [a.conj(), c.conj()],
                [b.conj(), d.conj()],
            ],
        }
    }
}

/// Trait para portas quânticas
pub trait QuantumGate: Send + Sync {
    /// Nome da porta
    fn name(&self) -> &'static str;

    /// Matriz da porta (2x2 para single-qubit)
    fn matrix(&self) -> Matrix2x2;

    /// Verifica se é unitária
    fn is_unitary(&self) -> bool {
        let m = self.matrix();
        let m_dag = m.dagger();
        let product = m.mul(&m_dag);

        // Verifica se produto é identidade
        let [[a, b], [c, d]] = product.elements;
        (a.re - 1.0).abs() < 1e-10
            && a.im.abs() < 1e-10
            && b.norm_sq() < 1e-10
            && c.norm_sq() < 1e-10
            && (d.re - 1.0).abs() < 1e-10
            && d.im.abs() < 1e-10
    }

    /// Aplica a um estado
    fn apply(&self, state: [Complex; 2]) -> [Complex; 2] {
        self.matrix().apply(state)
    }
}

// =============================================================================
// Portas Padrão
// =============================================================================

/// Porta Hadamard: cria superposição
#[derive(Clone, Copy, Debug, Default)]
pub struct Hadamard;

impl QuantumGate for Hadamard {
    fn name(&self) -> &'static str {
        "H"
    }

    fn matrix(&self) -> Matrix2x2 {
        let h = FRAC_1_SQRT_2;
        Matrix2x2 {
            elements: [
                [Complex::new(h, 0.0), Complex::new(h, 0.0)],
                [Complex::new(h, 0.0), Complex::new(-h, 0.0)],
            ],
        }
    }
}

/// Porta Pauli-X (NOT quântico)
#[derive(Clone, Copy, Debug, Default)]
pub struct PauliX;

impl QuantumGate for PauliX {
    fn name(&self) -> &'static str {
        "X"
    }

    fn matrix(&self) -> Matrix2x2 {
        Matrix2x2 {
            elements: [
                [Complex::ZERO, Complex::ONE],
                [Complex::ONE, Complex::ZERO],
            ],
        }
    }
}

/// Porta Pauli-Y
#[derive(Clone, Copy, Debug, Default)]
pub struct PauliY;

impl QuantumGate for PauliY {
    fn name(&self) -> &'static str {
        "Y"
    }

    fn matrix(&self) -> Matrix2x2 {
        Matrix2x2 {
            elements: [
                [Complex::ZERO, Complex::new(0.0, -1.0)],
                [Complex::new(0.0, 1.0), Complex::ZERO],
            ],
        }
    }
}

/// Porta Pauli-Z (phase flip)
#[derive(Clone, Copy, Debug, Default)]
pub struct PauliZ;

impl QuantumGate for PauliZ {
    fn name(&self) -> &'static str {
        "Z"
    }

    fn matrix(&self) -> Matrix2x2 {
        Matrix2x2 {
            elements: [
                [Complex::ONE, Complex::ZERO],
                [Complex::ZERO, Complex::new(-1.0, 0.0)],
            ],
        }
    }
}

/// Porta S (√Z)
#[derive(Clone, Copy, Debug, Default)]
pub struct SGate;

impl QuantumGate for SGate {
    fn name(&self) -> &'static str {
        "S"
    }

    fn matrix(&self) -> Matrix2x2 {
        Matrix2x2 {
            elements: [
                [Complex::ONE, Complex::ZERO],
                [Complex::ZERO, Complex::I],
            ],
        }
    }
}

/// Porta T (π/8)
#[derive(Clone, Copy, Debug, Default)]
pub struct TGate;

impl QuantumGate for TGate {
    fn name(&self) -> &'static str {
        "T"
    }

    fn matrix(&self) -> Matrix2x2 {
        let angle = PI / 4.0;
        Matrix2x2 {
            elements: [
                [Complex::ONE, Complex::ZERO],
                [Complex::ZERO, Complex::from_polar(angle)],
            ],
        }
    }
}

/// Porta de rotação em X
#[derive(Clone, Copy, Debug)]
pub struct RotationX {
    pub theta: f64,
}

impl RotationX {
    pub fn new(theta: f64) -> Self {
        Self { theta }
    }
}

impl QuantumGate for RotationX {
    fn name(&self) -> &'static str {
        "Rx"
    }

    fn matrix(&self) -> Matrix2x2 {
        let c = (self.theta / 2.0).cos();
        let s = (self.theta / 2.0).sin();
        Matrix2x2 {
            elements: [
                [Complex::new(c, 0.0), Complex::new(0.0, -s)],
                [Complex::new(0.0, -s), Complex::new(c, 0.0)],
            ],
        }
    }
}

/// Porta de rotação em Y
#[derive(Clone, Copy, Debug)]
pub struct RotationY {
    pub theta: f64,
}

impl RotationY {
    pub fn new(theta: f64) -> Self {
        Self { theta }
    }
}

impl QuantumGate for RotationY {
    fn name(&self) -> &'static str {
        "Ry"
    }

    fn matrix(&self) -> Matrix2x2 {
        let c = (self.theta / 2.0).cos();
        let s = (self.theta / 2.0).sin();
        Matrix2x2 {
            elements: [
                [Complex::new(c, 0.0), Complex::new(-s, 0.0)],
                [Complex::new(s, 0.0), Complex::new(c, 0.0)],
            ],
        }
    }
}

/// Porta de rotação em Z
#[derive(Clone, Copy, Debug)]
pub struct RotationZ {
    pub theta: f64,
}

impl RotationZ {
    pub fn new(theta: f64) -> Self {
        Self { theta }
    }
}

impl QuantumGate for RotationZ {
    fn name(&self) -> &'static str {
        "Rz"
    }

    fn matrix(&self) -> Matrix2x2 {
        let half = self.theta / 2.0;
        Matrix2x2 {
            elements: [
                [Complex::from_polar(-half), Complex::ZERO],
                [Complex::ZERO, Complex::from_polar(half)],
            ],
        }
    }
}

/// Porta de fase genérica
#[derive(Clone, Copy, Debug)]
pub struct Phase {
    pub phi: f64,
}

impl Phase {
    pub fn new(phi: f64) -> Self {
        Self { phi }
    }
}

impl QuantumGate for Phase {
    fn name(&self) -> &'static str {
        "P"
    }

    fn matrix(&self) -> Matrix2x2 {
        Matrix2x2 {
            elements: [
                [Complex::ONE, Complex::ZERO],
                [Complex::ZERO, Complex::from_polar(self.phi)],
            ],
        }
    }
}

// =============================================================================
// Testes
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quantum_regime_from_theta() {
        assert_eq!(QuantumRegime::from_theta(0), QuantumRegime::Classical);
        assert_eq!(QuantumRegime::from_theta(4), QuantumRegime::Coherent);
        assert_eq!(QuantumRegime::from_theta(14), QuantumRegime::Unitary);
    }

    #[test]
    fn test_quantum_regime_roundtrip() {
        for theta in (0..16).step_by(2) {
            let regime = QuantumRegime::from_theta(theta);
            assert_eq!(regime.to_theta(), theta);
        }
    }

    #[test]
    fn test_hadamard_unitary() {
        let h = Hadamard;
        assert!(h.is_unitary());
    }

    #[test]
    fn test_pauli_gates_unitary() {
        assert!(PauliX.is_unitary());
        assert!(PauliY.is_unitary());
        assert!(PauliZ.is_unitary());
    }

    #[test]
    fn test_hadamard_creates_superposition() {
        let h = Hadamard;
        let zero = [Complex::ONE, Complex::ZERO];

        let result = h.apply(zero);

        // |+⟩ = (|0⟩ + |1⟩)/√2
        let expected = FRAC_1_SQRT_2;
        assert!((result[0].re - expected).abs() < 1e-10);
        assert!((result[1].re - expected).abs() < 1e-10);
    }

    #[test]
    fn test_pauli_x_flips() {
        let x = PauliX;
        let zero = [Complex::ONE, Complex::ZERO];

        let result = x.apply(zero);

        // X|0⟩ = |1⟩
        assert!(result[0].norm_sq() < 1e-10);
        assert!((result[1].re - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_pauli_z_phase() {
        let z = PauliZ;
        let one = [Complex::ZERO, Complex::ONE];

        let result = z.apply(one);

        // Z|1⟩ = -|1⟩
        assert!(result[0].norm_sq() < 1e-10);
        assert!((result[1].re + 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_hadamard_self_inverse() {
        let h = Hadamard;
        let zero = [Complex::ONE, Complex::ZERO];

        // H² = I
        let result = h.apply(h.apply(zero));

        assert!((result[0].re - 1.0).abs() < 1e-10);
        assert!(result[1].norm_sq() < 1e-10);
    }

    #[test]
    fn test_rotation_gates() {
        let rx = RotationX::new(PI);
        let ry = RotationY::new(PI);
        let rz = RotationZ::new(PI);

        assert!(rx.is_unitary());
        assert!(ry.is_unitary());
        assert!(rz.is_unitary());
    }

    #[test]
    fn test_s_gate() {
        let s = SGate;
        assert!(s.is_unitary());

        // S² = Z
        let s2 = s.matrix().mul(&s.matrix());
        let z = PauliZ.matrix();

        assert!((s2.elements[1][1].re - z.elements[1][1].re).abs() < 1e-10);
    }

    #[test]
    fn test_t_gate() {
        let t = TGate;
        assert!(t.is_unitary());
    }

    #[test]
    fn test_complex_arithmetic() {
        let a = Complex::new(1.0, 2.0);
        let b = Complex::new(3.0, 4.0);

        let sum = a.add(b);
        assert_eq!(sum.re, 4.0);
        assert_eq!(sum.im, 6.0);

        let product = a.mul(b);
        assert_eq!(product.re, -5.0); // 1*3 - 2*4
        assert_eq!(product.im, 10.0); // 1*4 + 2*3
    }
}
