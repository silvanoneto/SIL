//! # ByteSil — Unidade Fundamental do Padrão SIL
//!
//! Representação log-polar de número complexo em 8 bits.
//!
//! ## Layout de Bits
//!
//! ```text
//! ┌──────┬──────┬──────┬──────┬──────┬──────┬──────┬──────┐
//! │  ρ₃  │  ρ₂  │  ρ₁  │  ρ₀  │  θ₃  │  θ₂  │  θ₁  │  θ₀  │
//! └──────┴──────┴──────┴──────┴──────┴──────┴──────┴──────┘
//!   bit 7   bit 6   bit 5   bit 4   bit 3   bit 2   bit 1   bit 0
//!   ◄────── LOG-MAGNITUDE ──────►◄────────── FASE ──────────►
//!              ρ ∈ [-8, +7]              θ ∈ [0, 15]
//! ```
//!
//! ## Decodificação
//!
//! - ρ (log-magnitude): `(bits >> 4) - 8` → [-8, +7]
//! - θ (fase): `(bits & 0x0F) × π/8` → [0, 2π)
//! - |z| (magnitude): `e^ρ` → [0.00034, 1097]
//! - z (complexo): `e^(ρ + iθπ/8)`

use num_complex::Complex;
use std::f64::consts::PI;
use std::fmt;

/// Byte de Sil: representação log-polar de número complexo em 8 bits
///
/// # Propriedades
///
/// - **Compacto**: 8 bits por número complexo
/// - **Faixa dinâmica**: 65 dB (de e⁻⁸ ≈ 0.00034 a e⁷ ≈ 1097)
/// - **Operações O(1)**: multiplicação, divisão, potência, raiz
///
/// # Exemplo
///
/// ```
/// use sil_core::state::ByteSil;
///
/// // Criar de magnitude e fase
/// let z = ByteSil::new(0, 4); // |z| = 1, θ = π/2 → z ≈ i
///
/// // Multiplicação O(1)
/// let z2 = z.mul(&z); // i² = -1 → ρ=0, θ=8
///
/// // Converter para Complex<f64>
/// let c = z.to_complex();
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(C)]
pub struct ByteSil {
    /// ρ: log-magnitude ∈ [-8, +7]
    pub rho: i8,
    /// θ: índice de fase ∈ [0, 15]
    pub theta: u8,
}

impl ByteSil {
    // =========================================================================
    // Constantes
    // =========================================================================
    
    /// Valor mínimo de ρ
    pub const RHO_MIN: i8 = -8;
    
    /// Valor máximo de ρ
    pub const RHO_MAX: i8 = 7;
    
    /// Número de valores de fase (16)
    pub const THETA_COUNT: u8 = 16;
    
    /// Magnitude mínima representável (e⁻⁸ ≈ 0.000335)
    pub const MAGNITUDE_MIN: f64 = 0.00033546262790251185;
    
    /// Magnitude máxima representável (e⁷ ≈ 1096.6)
    pub const MAGNITUDE_MAX: f64 = 1096.6331584284585;
    
    // =========================================================================
    // Valores Especiais
    // =========================================================================
    
    /// Null (estado zero): ρ=-8, θ=0
    pub const NULL: Self = Self { rho: -8, theta: 0 };
    
    /// Um (1 + 0i): ρ=0, θ=0
    pub const ONE: Self = Self { rho: 0, theta: 0 };
    
    /// Unidade imaginária i (0 + 1i): ρ=0, θ=4
    pub const I: Self = Self { rho: 0, theta: 4 };
    
    /// Menos um (-1 + 0i): ρ=0, θ=8
    pub const NEG_ONE: Self = Self { rho: 0, theta: 8 };
    
    /// Menos i (0 - 1i): ρ=0, θ=12
    pub const NEG_I: Self = Self { rho: 0, theta: 12 };
    
    /// Máximo positivo real (e⁷ + 0i): ρ=7, θ=0
    pub const MAX: Self = Self { rho: 7, theta: 0 };
    
    // =========================================================================
    // Construtores
    // =========================================================================
    
    /// Cria ByteSil de log-magnitude e índice de fase
    ///
    /// # Argumentos
    ///
    /// - `rho`: Log-magnitude ∈ [-8, 7] (será clampado)
    /// - `theta`: Índice de fase ∈ [0, 15] (será mascarado)
    #[inline]
    pub const fn new(rho: i8, theta: u8) -> Self {
        let rho_clamped = if rho < Self::RHO_MIN {
            Self::RHO_MIN
        } else if rho > Self::RHO_MAX {
            Self::RHO_MAX
        } else {
            rho
        };
        Self {
            rho: rho_clamped,
            theta: theta & 0x0F,
        }
    }
    
    /// Cria ByteSil nulo (estado zero)
    #[inline]
    pub const fn null() -> Self {
        Self::NULL
    }
    
    /// Verifica se é nulo (ρ == -8)
    #[inline]
    pub const fn is_null(&self) -> bool {
        self.rho == Self::RHO_MIN
    }
    
    // =========================================================================
    // Conversões
    // =========================================================================
    
    /// Converte para número complexo
    #[inline]
    pub fn to_complex(&self) -> Complex<f64> {
        let r = (self.rho as f64).exp();
        let t = (self.theta as f64) * PI / 8.0;
        Complex::from_polar(r, t)
    }
    
    /// Cria de número complexo
    pub fn from_complex(z: Complex<f64>) -> Self {
        if z.norm() < Self::MAGNITUDE_MIN {
            return Self::NULL;
        }
        
        let rho = z.norm().ln().round().clamp(Self::RHO_MIN as f64, Self::RHO_MAX as f64) as i8;
        let theta_f = z.arg().rem_euclid(2.0 * PI) / PI * 8.0;
        let theta = (theta_f.round() as u8) % 16;
        
        Self::new(rho, theta)
    }
    
    /// Converte para byte único (serialização)
    #[inline]
    pub const fn to_u8(&self) -> u8 {
        (((self.rho - Self::RHO_MIN) as u8) << 4) | self.theta
    }
    
    /// Cria de byte único (deserialização)
    #[inline]
    pub const fn from_u8(byte: u8) -> Self {
        Self {
            rho: ((byte >> 4) as i8) + Self::RHO_MIN,
            theta: byte & 0x0F,
        }
    }

    /// Retorna coordenadas polares (rho, theta_degrees)
    /// - rho: log-magnitude ∈ [-8, 7]
    /// - theta_degrees: fase em graus ∈ [0, 337.5] (step 22.5°)
    #[inline]
    pub const fn to_polar(&self) -> (i8, u16) {
        // Cada passo de theta = π/8 rad = 22.5°
        // theta ∈ [0, 15] → graus ∈ [0, 337.5]
        let degrees = (self.theta as u16) * 225 / 10; // 22.5 * theta
        (self.rho, degrees)
    }

    // =========================================================================
    // Operações Fundamentais
    // =========================================================================
    
    /// Multiplicação complexa O(1)
    ///
    /// Em coordenadas log-polares: multiplicar = somar log-magnitudes e fases
    #[inline]
    pub const fn mul(&self, other: &ByteSil) -> ByteSil {
        let rho_sum = self.rho as i16 + other.rho as i16;
        let rho = if rho_sum < Self::RHO_MIN as i16 {
            Self::RHO_MIN
        } else if rho_sum > Self::RHO_MAX as i16 {
            Self::RHO_MAX
        } else {
            rho_sum as i8
        };
        
        ByteSil {
            rho,
            theta: (self.theta + other.theta) % 16,
        }
    }
    
    /// Divisão (subtração de log-magnitudes e fases)
    #[inline]
    pub const fn div(&self, other: &ByteSil) -> ByteSil {
        let rho_diff = self.rho as i16 - other.rho as i16;
        let rho = if rho_diff < Self::RHO_MIN as i16 {
            Self::RHO_MIN
        } else if rho_diff > Self::RHO_MAX as i16 {
            Self::RHO_MAX
        } else {
            rho_diff as i8
        };
        
        ByteSil {
            rho,
            theta: ((self.theta as i16 - other.theta as i16 + 16) % 16) as u8,
        }
    }
    
    /// Potenciação (escala log-magnitude, multiplica fase)
    #[inline]
    pub const fn pow(&self, n: i32) -> ByteSil {
        let rho_scaled = self.rho as i32 * n;
        let rho = if rho_scaled < Self::RHO_MIN as i32 {
            Self::RHO_MIN
        } else if rho_scaled > Self::RHO_MAX as i32 {
            Self::RHO_MAX
        } else {
            rho_scaled as i8
        };
        let theta_scaled = ((self.theta as i32 * n) % 16 + 16) % 16;
        
        ByteSil {
            rho,
            theta: theta_scaled as u8,
        }
    }
    
    /// Raiz n-ésima (divide log-magnitude e fase)
    #[inline]
    pub const fn root(&self, n: i32) -> ByteSil {
        if n == 0 {
            return Self::ONE;
        }
        let rho_scaled = self.rho as i32 / n;
        let theta_scaled = (self.theta as i32 / n) % 16;
        
        ByteSil {
            rho: rho_scaled as i8,
            theta: theta_scaled as u8,
        }
    }
    
    /// Inversão (negação de log-magnitude)
    #[inline]
    pub const fn inv(&self) -> ByteSil {
        ByteSil {
            rho: -self.rho,
            theta: (16 - self.theta) % 16,
        }
    }
    
    /// XOR (preserva entropia)
    #[inline]
    pub const fn xor(&self, other: &ByteSil) -> ByteSil {
        ByteSil {
            rho: self.rho ^ other.rho,
            theta: self.theta ^ other.theta,
        }
    }
    
    /// Conjugado (inversão de fase)
    #[inline]
    pub const fn conj(&self) -> ByteSil {
        ByteSil {
            rho: self.rho,
            theta: (16 - self.theta) % 16,
        }
    }
    
    /// Norma (colapso para magnitude): retorna 0-15
    #[inline]
    pub const fn norm(&self) -> u8 {
        (self.rho - Self::RHO_MIN) as u8
    }
    
    /// Fase (colapso para ângulo): retorna 0-15
    #[inline]
    pub const fn phase(&self) -> u8 {
        self.theta
    }
    
    /// Mix: combina dois ByteSil (média log-polar)
    #[inline]
    pub fn mix(&self, other: &ByteSil) -> ByteSil {
        ByteSil {
            rho: ((self.rho as i16 + other.rho as i16) / 2) as i8,
            theta: ((self.theta as u16 + other.theta as u16) / 2) as u8,
        }
    }
}

// =============================================================================
// Traits
// =============================================================================

impl fmt::Debug for ByteSil {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ByteSil(ρ={}, θ={})", self.rho, self.theta)
    }
}

impl fmt::Display for ByteSil {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let z = self.to_complex();
        write!(f, "{:.3}{:+.3}i", z.re, z.im)
    }
}

impl From<u8> for ByteSil {
    fn from(byte: u8) -> Self {
        Self::from_u8(byte)
    }
}

impl From<ByteSil> for u8 {
    fn from(b: ByteSil) -> Self {
        b.to_u8()
    }
}

impl From<Complex<f64>> for ByteSil {
    fn from(z: Complex<f64>) -> Self {
        Self::from_complex(z)
    }
}

impl From<ByteSil> for Complex<f64> {
    fn from(b: ByteSil) -> Self {
        b.to_complex()
    }
}

// =============================================================================
// Serde
// =============================================================================

impl serde::Serialize for ByteSil {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("ByteSil", 2)?;
        state.serialize_field("rho", &self.rho)?;
        state.serialize_field("theta", &self.theta)?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for ByteSil {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct ByteSilData {
            rho: i8,
            theta: u8,
        }
        
        let data = ByteSilData::deserialize(deserializer)?;
        Ok(ByteSil::new(data.rho, data.theta))
    }
}

// =============================================================================
// Testes
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_null() {
        let null = ByteSil::null();
        assert!(null.is_null());
        assert_eq!(null.rho, -8);
        assert_eq!(null.theta, 0);
    }
    
    #[test]
    fn test_one() {
        let one = ByteSil::ONE;
        let z = one.to_complex();
        assert!((z.re - 1.0).abs() < 1e-10);
        assert!(z.im.abs() < 1e-10);
    }
    
    #[test]
    fn test_i_squared() {
        let i = ByteSil::I;
        let i_squared = i.mul(&i);
        assert_eq!(i_squared.theta, 8); // θ = 8 → -1
    }
    
    #[test]
    fn test_serialization_roundtrip() {
        let original = ByteSil::new(3, 7);
        let byte = original.to_u8();
        let restored = ByteSil::from_u8(byte);
        assert_eq!(original, restored);
    }
    
    #[test]
    fn test_xor_self_is_null_ish() {
        let b = ByteSil::new(5, 10);
        let xored = b.xor(&b);
        assert_eq!(xored.rho, 0);
        assert_eq!(xored.theta, 0);
    }
    
    #[test]
    fn test_conjugate() {
        let b = ByteSil::new(2, 3);
        let conj = b.conj();
        assert_eq!(conj.rho, 2);
        assert_eq!(conj.theta, 13); // 16 - 3 = 13
    }
}

// Implementação de operadores
impl std::ops::BitXor for ByteSil {
    type Output = Self;
    
    fn bitxor(self, rhs: Self) -> Self::Output {
        self.xor(&rhs)
    }
}

impl std::ops::BitXor<&ByteSil> for ByteSil {
    type Output = Self;
    
    fn bitxor(self, rhs: &Self) -> Self::Output {
        self.xor(rhs)
    }
}
