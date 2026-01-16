//! # BitDeSil — Unidade Fundamental de Informacao Multidimensional
//!
//! O Bit de Sil reinterpreta o bit classico como entidade multidimensional
//! que habita simultaneamente multiplos dominios matematicos.
//!
//! ## As 7 Faces do Bit
//!
//! | Face | Nome | Dominio | Quantum |
//! |------|------|---------|---------|
//! | 0 | Classica | {0, 1} | 1 |
//! | 1 | Rotacional | e^{ikπ/8} | π/8 |
//! | 2 | Logaritmica | e^{±1} | e |
//! | 3 | Fibonacci | φ^{±1} | φ |
//! | 4 | Topologica | Z (winding) | 1 volta |
//! | 5 | Holomorfa | zero/polo | singularidade |
//! | 6 | Quantica | C² | |ψ⟩ |
//!
//! ## Representacao
//!
//! ```text
//! b_Sil = (b_c, b_θ, b_ρ, b_φ, b_w, b_h, b_q)
//! ```
//!
//! ## Filosofia
//!
//! > *"O bit nao e um ponto — e um portal. Nao armazena informacao — a transforma."*

use num_complex::Complex;
use std::f64::consts::PI;
use std::fmt;

use super::ByteSil;

/// Constante phi (numero de ouro)
pub const PHI: f64 = 1.618033988749895;

/// Inverso de phi
pub const PHI_INV: f64 = 0.6180339887498949;

/// Bit de Sil — unidade fundamental de informacao multidimensional
///
/// Representa as 7 faces do bit simultaneamente:
/// - Face 0: Classica (0 ou 1)
/// - Face 1: Rotacional (fase em unidades de π/8)
/// - Face 2: Logaritmica (magnitude em escala log)
/// - Face 3: Fibonacci (potencia de φ)
/// - Face 4: Topologica (winding number)
/// - Face 5: Holomorfa (zero ou polo)
/// - Face 6: Quantica (superposicao α|0⟩ + β|1⟩)
#[derive(Clone, Copy, PartialEq)]
pub struct BitDeSil {
    // =========================================================================
    // Face 0: Classica
    // =========================================================================
    /// Face classica: {0, 1}
    /// - false = 0 (estado base)
    /// - true = 1 (estado excitado)
    pub classical: bool,

    // =========================================================================
    // Face 1: Rotacional
    // =========================================================================
    /// Face rotacional: e^{iπk/8}, k ∈ [0, 15]
    /// Cada unidade representa 22.5° de rotacao no plano complexo.
    pub phase: u8,

    // =========================================================================
    // Face 2: Logaritmica
    // =========================================================================
    /// Face logaritmica: e^ρ, ρ ∈ [-8, +7]
    /// Magnitude em escala logaritmica natural.
    pub magnitude: i8,

    // =========================================================================
    // Face 3: Fibonacci
    // =========================================================================
    /// Face Fibonacci: φ^n
    /// Potencia do numero de ouro, representa crescimento natural.
    pub fibonacci_power: i8,

    // =========================================================================
    // Face 4: Topologica
    // =========================================================================
    /// Face topologica: winding number
    /// Numero de voltas ao redor da origem (invariante topologico).
    pub winding: i8,

    // =========================================================================
    // Face 5: Holomorfa
    // =========================================================================
    /// Face holomorfa: zero (false) ou polo (true)
    /// Indica tipo de singularidade no plano complexo.
    pub is_pole: bool,

    // =========================================================================
    // Face 6: Quantica
    // =========================================================================
    /// Face quantica: amplitude de |0⟩
    /// |α|² = probabilidade de medir 0
    pub alpha: f32,

    /// Face quantica: amplitude de |1⟩
    /// |β|² = probabilidade de medir 1
    pub beta: f32,
}

impl BitDeSil {
    // =========================================================================
    // Constantes
    // =========================================================================

    /// Bit de Sil nulo (vazio, estado zero)
    pub const NULL: Self = Self {
        classical: false,
        phase: 0,
        magnitude: -8,
        fibonacci_power: 0,
        winding: 0,
        is_pole: false,
        alpha: 1.0,
        beta: 0.0,
    };

    /// Bit de Sil unitario (1 + 0i)
    pub const ONE: Self = Self {
        classical: false,
        phase: 0,
        magnitude: 0,
        fibonacci_power: 0,
        winding: 0,
        is_pole: false,
        alpha: 1.0,
        beta: 0.0,
    };

    /// Bit de Sil imaginario puro (0 + 1i)
    pub const I: Self = Self {
        classical: false,
        phase: 4, // π/2
        magnitude: 0,
        fibonacci_power: 0,
        winding: 0,
        is_pole: false,
        alpha: 0.707107, // 1/√2
        beta: 0.707107,
    };

    /// Bit de Sil negativo (-1 + 0i)
    pub const NEG_ONE: Self = Self {
        classical: true,
        phase: 8, // π
        magnitude: 0,
        fibonacci_power: 0,
        winding: 0,
        is_pole: false,
        alpha: 0.0,
        beta: 1.0,
    };

    // =========================================================================
    // Construtores
    // =========================================================================

    /// Cria BitDeSil com valores especificos para todas as 7 faces
    pub const fn new(
        classical: bool,
        phase: u8,
        magnitude: i8,
        fibonacci_power: i8,
        winding: i8,
        is_pole: bool,
        alpha: f32,
        beta: f32,
    ) -> Self {
        Self {
            classical,
            phase: phase & 0x0F,
            magnitude: if magnitude < -8 {
                -8
            } else if magnitude > 7 {
                7
            } else {
                magnitude
            },
            fibonacci_power,
            winding,
            is_pole,
            alpha,
            beta,
        }
    }

    /// Cria BitDeSil a partir de um byte (8 bits)
    ///
    /// Layout do byte:
    /// ```text
    /// [ρ₃ ρ₂ ρ₁ ρ₀ | θ₃ θ₂ θ₁ θ₀]
    ///  bits 7-4      bits 3-0
    /// ```
    pub fn from_byte(byte: u8) -> Self {
        let magnitude = ((byte >> 4) as i8) - 8; // bits [7:4] -> [-8, +7]
        let phase = byte & 0x0F; // bits [3:0] -> [0, 15]

        // Derivar outras faces do byte
        let classical = magnitude >= 0;
        let is_pole = magnitude > 3; // alta magnitude = polo
        let winding = (phase / 4) as i8; // cada quadrante = 1 winding

        // Fibonacci: aproximar pela magnitude
        let fibonacci_power = (magnitude as f64 / 0.4812).round() as i8;

        // Quantico: distribuir baseado na fase
        let theta = (phase as f64) * PI / 8.0;
        let alpha = (theta / 2.0).cos() as f32;
        let beta = (theta / 2.0).sin() as f32;

        Self {
            classical,
            phase,
            magnitude,
            fibonacci_power,
            winding,
            is_pole,
            alpha,
            beta,
        }
    }

    /// Cria BitDeSil a partir de ByteSil
    pub fn from_byte_sil(bs: &ByteSil) -> Self {
        Self::from_byte(bs.to_u8())
    }

    /// Cria BitDeSil em superposicao igual (|+⟩ = (|0⟩ + |1⟩)/√2)
    pub fn superposition() -> Self {
        Self {
            classical: false,
            phase: 0,
            magnitude: 0,
            fibonacci_power: 0,
            winding: 0,
            is_pole: false,
            alpha: 0.707107, // 1/√2
            beta: 0.707107,
        }
    }

    // =========================================================================
    // Conversoes
    // =========================================================================

    /// Converte para ByteSil (perde informacao das faces 3-6)
    pub fn to_byte_sil(&self) -> ByteSil {
        ByteSil::new(self.magnitude, self.phase)
    }

    /// Converte para byte unico
    pub fn to_byte(&self) -> u8 {
        self.to_byte_sil().to_u8()
    }

    /// Converte para numero complexo usando faces 1 e 2
    pub fn to_complex(&self) -> Complex<f64> {
        let r = (self.magnitude as f64).exp();
        let theta = (self.phase as f64) * PI / 8.0;
        Complex::from_polar(r, theta)
    }

    /// Cria de numero complexo
    pub fn from_complex(z: Complex<f64>) -> Self {
        let bs = ByteSil::from_complex(z);
        Self::from_byte_sil(&bs)
    }

    /// Retorna potencia de phi correspondente (Face 3)
    pub fn phi_power(&self) -> f64 {
        PHI.powi(self.fibonacci_power as i32)
    }

    // =========================================================================
    // Operacoes por Face
    // =========================================================================

    // --- Face 0: Classica ---

    /// Negacao classica (NOT)
    pub fn not_classical(&self) -> Self {
        Self {
            classical: !self.classical,
            ..*self
        }
    }

    // --- Face 1: Rotacional ---

    /// Rotacao por n quanta de fase (cada quantum = π/8 = 22.5°)
    pub fn rotate(&self, n: i8) -> Self {
        let new_phase = ((self.phase as i8 + n).rem_euclid(16)) as u8;
        Self {
            phase: new_phase,
            winding: self.winding + (n / 16), // atualiza winding se passar de 360°
            ..*self
        }
    }

    /// Negacao rotacional (rotacao de 180° = 8 quanta)
    pub fn not_rotational(&self) -> Self {
        self.rotate(8)
    }

    // --- Face 2: Logaritmica ---

    /// Escala por n quanta de magnitude
    pub fn scale(&self, n: i8) -> Self {
        let new_mag = (self.magnitude as i16 + n as i16).clamp(-8, 7) as i8;
        Self {
            magnitude: new_mag,
            classical: new_mag >= 0,
            ..*self
        }
    }

    // --- Face 3: Fibonacci ---

    /// Incrementa potencia de phi (crescimento aureo)
    pub fn grow(&self) -> Self {
        Self {
            fibonacci_power: self.fibonacci_power.saturating_add(1),
            ..*self
        }
    }

    /// Decrementa potencia de phi
    pub fn shrink(&self) -> Self {
        Self {
            fibonacci_power: self.fibonacci_power.saturating_sub(1),
            ..*self
        }
    }

    // --- Face 4: Topologica ---

    /// Adiciona uma volta (winding +1)
    pub fn wind(&self) -> Self {
        Self {
            winding: self.winding.saturating_add(1),
            ..*self
        }
    }

    /// Remove uma volta (winding -1)
    pub fn unwind(&self) -> Self {
        Self {
            winding: self.winding.saturating_sub(1),
            ..*self
        }
    }

    // --- Face 5: Holomorfa ---

    /// Inverte tipo de singularidade (zero <-> polo)
    pub fn invert_singularity(&self) -> Self {
        Self {
            is_pole: !self.is_pole,
            ..*self
        }
    }

    // --- Face 6: Quantica ---

    /// Aplica porta Hadamard
    pub fn hadamard(&self) -> Self {
        let sqrt2_inv = 0.707107_f32;
        Self {
            alpha: sqrt2_inv * (self.alpha + self.beta),
            beta: sqrt2_inv * (self.alpha - self.beta),
            ..*self
        }
    }

    /// Aplica porta Pauli-X (NOT quantico)
    pub fn pauli_x(&self) -> Self {
        Self {
            alpha: self.beta,
            beta: self.alpha,
            classical: !self.classical,
            ..*self
        }
    }

    /// Aplica porta Pauli-Z (flip de fase)
    pub fn pauli_z(&self) -> Self {
        Self {
            beta: -self.beta,
            ..*self
        }
    }

    /// Aplica porta Pauli-Y (rotacao + flip)
    /// Y|0⟩ = i|1⟩, Y|1⟩ = -i|0⟩
    pub fn pauli_y(&self) -> Self {
        Self {
            alpha: -self.beta,
            beta: self.alpha,
            classical: !self.classical,
            phase: (self.phase + 4) & 0x0F, // +π/2
            ..*self
        }
    }

    /// Colapso quantico (medicao)
    /// Retorna o resultado da medicao (0 ou 1) e o estado colapsado
    pub fn collapse(&self, random_value: f32) -> (bool, Self) {
        let prob_zero = self.alpha * self.alpha;
        let result = random_value >= prob_zero;

        let collapsed = if result {
            // Mediu |1⟩
            Self {
                classical: true,
                alpha: 0.0,
                beta: 1.0,
                ..*self
            }
        } else {
            // Mediu |0⟩
            Self {
                classical: false,
                alpha: 1.0,
                beta: 0.0,
                ..*self
            }
        };

        (result, collapsed)
    }

    /// Normaliza amplitudes quanticas para |α|² + |β|² = 1
    pub fn normalize(&self) -> Self {
        let norm = (self.alpha * self.alpha + self.beta * self.beta).sqrt();
        if norm < f32::EPSILON {
            return Self::ONE;
        }
        Self {
            alpha: self.alpha / norm,
            beta: self.beta / norm,
            ..*self
        }
    }

    // =========================================================================
    // Operacoes Compostas
    // =========================================================================

    /// Multiplicacao de bits de Sil (soma log-polar + composicao quantica)
    pub fn multiply(&self, other: &Self) -> Self {
        Self {
            classical: self.classical && other.classical,
            phase: (self.phase + other.phase) & 0x0F,
            magnitude: (self.magnitude as i16 + other.magnitude as i16).clamp(-8, 7) as i8,
            fibonacci_power: self.fibonacci_power + other.fibonacci_power,
            winding: self.winding + other.winding,
            is_pole: self.is_pole || other.is_pole,
            alpha: self.alpha * other.alpha - self.beta * other.beta,
            beta: self.alpha * other.beta + self.beta * other.alpha,
        }
        .normalize()
    }

    /// Divisao (subtracao log-polar)
    pub fn divide(&self, other: &Self) -> Self {
        Self {
            classical: self.classical && !other.classical,
            phase: ((self.phase as i8 - other.phase as i8 + 16) % 16) as u8,
            magnitude: (self.magnitude as i16 - other.magnitude as i16).clamp(-8, 7) as i8,
            fibonacci_power: self.fibonacci_power - other.fibonacci_power,
            winding: self.winding - other.winding,
            is_pole: self.is_pole && !other.is_pole,
            alpha: self.alpha,
            beta: self.beta,
        }
    }

    /// XOR (distancia no plano complexo)
    pub fn xor(&self, other: &Self) -> Self {
        Self {
            classical: self.classical != other.classical,
            phase: self.phase ^ other.phase,
            magnitude: self.magnitude ^ other.magnitude,
            fibonacci_power: (self.fibonacci_power - other.fibonacci_power).abs(),
            winding: self.winding - other.winding,
            is_pole: self.is_pole != other.is_pole,
            alpha: (self.alpha - other.alpha).abs(),
            beta: (self.beta - other.beta).abs(),
        }
        .normalize()
    }

    /// Conjugado (inversao de fase + beta)
    pub fn conjugate(&self) -> Self {
        Self {
            phase: (16 - self.phase) % 16,
            beta: -self.beta,
            ..*self
        }
    }

    // =========================================================================
    // Predicados
    // =========================================================================

    /// Verifica se esta em superposicao (nao colapsado)
    pub fn is_superposed(&self) -> bool {
        let prob_zero = self.alpha * self.alpha;
        prob_zero > 0.001 && prob_zero < 0.999
    }

    /// Verifica se e nulo (magnitude minima)
    pub fn is_null(&self) -> bool {
        self.magnitude == -8
    }

    /// Verifica se e unitario (magnitude zero)
    pub fn is_unit(&self) -> bool {
        self.magnitude == 0
    }

    /// Retorna probabilidade de medir |0⟩
    pub fn prob_zero(&self) -> f32 {
        self.alpha * self.alpha
    }

    /// Retorna probabilidade de medir |1⟩
    pub fn prob_one(&self) -> f32 {
        self.beta * self.beta
    }
}

// =============================================================================
// Traits
// =============================================================================

impl Default for BitDeSil {
    fn default() -> Self {
        Self::ONE
    }
}

impl fmt::Debug for BitDeSil {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "BitDeSil(c={}, θ={}, ρ={}, φ^{}, w={}, {}pole, {:.2}|0⟩+{:.2}|1⟩)",
            self.classical as u8,
            self.phase,
            self.magnitude,
            self.fibonacci_power,
            self.winding,
            if self.is_pole { "" } else { "!" },
            self.alpha,
            self.beta
        )
    }
}

impl fmt::Display for BitDeSil {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let z = self.to_complex();
        write!(
            f,
            "{:.3}{:+.3}i [w={}, φ^{}]",
            z.re, z.im, self.winding, self.fibonacci_power
        )
    }
}

impl From<u8> for BitDeSil {
    fn from(byte: u8) -> Self {
        Self::from_byte(byte)
    }
}

impl From<ByteSil> for BitDeSil {
    fn from(bs: ByteSil) -> Self {
        Self::from_byte_sil(&bs)
    }
}

impl From<BitDeSil> for ByteSil {
    fn from(bit: BitDeSil) -> Self {
        bit.to_byte_sil()
    }
}

impl From<BitDeSil> for u8 {
    fn from(bit: BitDeSil) -> Self {
        bit.to_byte()
    }
}

// =============================================================================
// Operadores
// =============================================================================

impl std::ops::Not for BitDeSil {
    type Output = Self;

    /// NOT = rotacao de 180° (8 quanta de fase)
    fn not(self) -> Self::Output {
        self.not_rotational()
    }
}

impl std::ops::BitAnd for BitDeSil {
    type Output = Self;

    /// AND = multiplicacao
    fn bitand(self, rhs: Self) -> Self::Output {
        self.multiply(&rhs)
    }
}

impl std::ops::BitXor for BitDeSil {
    type Output = Self;

    /// XOR = distancia
    fn bitxor(self, rhs: Self) -> Self::Output {
        self.xor(&rhs)
    }
}

impl std::ops::Mul for BitDeSil {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        self.multiply(&rhs)
    }
}

impl std::ops::Div for BitDeSil {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        self.divide(&rhs)
    }
}

// =============================================================================
// Serde
// =============================================================================

impl serde::Serialize for BitDeSil {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("BitDeSil", 8)?;
        state.serialize_field("classical", &self.classical)?;
        state.serialize_field("phase", &self.phase)?;
        state.serialize_field("magnitude", &self.magnitude)?;
        state.serialize_field("fibonacci_power", &self.fibonacci_power)?;
        state.serialize_field("winding", &self.winding)?;
        state.serialize_field("is_pole", &self.is_pole)?;
        state.serialize_field("alpha", &self.alpha)?;
        state.serialize_field("beta", &self.beta)?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for BitDeSil {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct BitDeSilData {
            classical: bool,
            phase: u8,
            magnitude: i8,
            fibonacci_power: i8,
            winding: i8,
            is_pole: bool,
            alpha: f32,
            beta: f32,
        }

        let data = BitDeSilData::deserialize(deserializer)?;
        Ok(BitDeSil::new(
            data.classical,
            data.phase,
            data.magnitude,
            data.fibonacci_power,
            data.winding,
            data.is_pole,
            data.alpha,
            data.beta,
        ))
    }
}

// =============================================================================
// Testes
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seven_faces() {
        let bit = BitDeSil::from_byte(0x84); // byte = 0b1000_0100 -> ρ=8-8=0, θ=4

        // Face 0: Classica (magnitude >= 0 -> true)
        assert!(bit.classical); // magnitude 0 >= 0

        // Face 1: Rotacional
        assert_eq!(bit.phase, 4); // π/2

        // Face 2: Logaritmica
        assert_eq!(bit.magnitude, 0); // |z| = 1

        // Face 4: Topologica
        assert_eq!(bit.winding, 1); // phase 4 / 4 = 1 quadrante

        // Face 6: Quantica
        assert!(bit.alpha > 0.0 && bit.beta > 0.0);
    }

    #[test]
    fn test_hadamard_gate() {
        // |0⟩ -> |+⟩
        let zero = BitDeSil {
            alpha: 1.0,
            beta: 0.0,
            ..BitDeSil::ONE
        };
        let plus = zero.hadamard();

        assert!((plus.alpha - 0.707107).abs() < 0.001);
        assert!((plus.beta - 0.707107).abs() < 0.001);
    }

    #[test]
    fn test_pauli_x() {
        // |0⟩ -> |1⟩
        let zero = BitDeSil {
            alpha: 1.0,
            beta: 0.0,
            classical: false,
            ..BitDeSil::ONE
        };
        let one = zero.pauli_x();

        assert!((one.alpha - 0.0).abs() < 0.001);
        assert!((one.beta - 1.0).abs() < 0.001);
        assert!(one.classical);
    }

    #[test]
    fn test_rotation() {
        let bit = BitDeSil::ONE;

        // Rotacao de 90° (4 quanta)
        let rotated = bit.rotate(4);
        assert_eq!(rotated.phase, 4);

        // Rotacao de 180° (8 quanta) = NOT
        let negated = bit.not_rotational();
        assert_eq!(negated.phase, 8);

        // Rotacao completa (16 quanta) volta ao original
        let full = bit.rotate(16);
        assert_eq!(full.phase, 0);
    }

    #[test]
    fn test_fibonacci_growth() {
        let bit = BitDeSil::ONE;

        // Crescimento aureo
        let grown = bit.grow().grow().grow();
        assert_eq!(grown.fibonacci_power, 3);

        // φ³ ≈ 4.236
        let expected = PHI.powi(3);
        assert!((grown.phi_power() - expected).abs() < 0.001);
    }

    #[test]
    fn test_multiplication() {
        let a = BitDeSil::from_byte(0x84); // ρ=0, θ=4 (i)
        let b = BitDeSil::from_byte(0x84); // ρ=0, θ=4 (i)

        let c = a.multiply(&b); // i * i = -1

        assert_eq!(c.phase, 8); // θ=8 corresponde a -1
        assert_eq!(c.magnitude, 0); // magnitude se soma: 0 + 0 = 0
    }

    #[test]
    fn test_collapse() {
        let superposed = BitDeSil::superposition();
        assert!(superposed.is_superposed());

        // Colapso para |0⟩
        let (result0, collapsed0) = superposed.collapse(0.2);
        assert!(!result0);
        assert!(!collapsed0.is_superposed());
        assert!((collapsed0.alpha - 1.0).abs() < 0.001);

        // Colapso para |1⟩
        let (result1, collapsed1) = superposed.collapse(0.8);
        assert!(result1);
        assert!(!collapsed1.is_superposed());
        assert!((collapsed1.beta - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_xor_self_is_identity() {
        let bit = BitDeSil::from_byte(0xA5);
        let xored = bit.xor(&bit);

        assert_eq!(xored.phase, 0);
        assert_eq!(xored.magnitude, 0);
        assert!(!xored.classical);
    }

    #[test]
    fn test_pauli_y() {
        // |0⟩ -> Y -> i|1⟩
        let zero = BitDeSil {
            alpha: 1.0,
            beta: 0.0,
            classical: false,
            ..BitDeSil::ONE
        };
        let result = zero.pauli_y();

        // alpha = -beta_orig = 0, beta = alpha_orig = 1
        assert!((result.alpha - 0.0).abs() < 0.001);
        assert!((result.beta - 1.0).abs() < 0.001);
        assert!(result.classical); // flip
    }

    #[test]
    fn test_hadamard_twice_returns_original() {
        // H * H = I (identidade)
        let bit = BitDeSil {
            alpha: 1.0,
            beta: 0.0,
            ..BitDeSil::ONE
        };
        let after_h1 = bit.hadamard();
        let after_h2 = after_h1.hadamard();

        assert!((after_h2.alpha - 1.0).abs() < 0.01);
        assert!((after_h2.beta - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_byte_roundtrip() {
        for byte in 0..=255u8 {
            let bit = BitDeSil::from_byte(byte);
            let recovered = bit.to_byte();
            assert_eq!(byte, recovered, "Failed for byte {:#04x}", byte);
        }
    }
}
