//! # SilState — Estado Completo de 16 Camadas
//!
//! Representa o estado de um nó na rede SIL como 16 ByteSil.
//!
//! ## Layout de Memória
//!
//! ```text
//! ┌────────────────────────────────────────────────────────────────┐
//! │                    SIL STATE (128 bits)                        │
//! ├────────────────────────────────────────────────────────────────┤
//! │  L(0) L(1) L(2) L(3) L(4) L(5) L(6) L(7)                      │
//! │  FOT  ACU  OLF  GUS  DER  ELE  PSI  AMB                       │
//! │  ◄──────── PERCEPÇÃO ────────►◄─── PROCESSO ───►              │
//! │                                                                │
//! │  L(8) L(9) L(A) L(B) L(C) L(D) L(E) L(F)                      │
//! │  CIB  GEO  COS  SIN  QUA  SUP  ENT  COL                       │
//! │  ◄───── INTERAÇÃO ────►◄─ EME ─►◄────── META ──────►          │
//! └────────────────────────────────────────────────────────────────┘
//! ```

use super::{ByteSil, NUM_LAYERS, layers};
use num_complex::Complex;
use std::fmt;

/// Estratégias de colapso de estado
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollapseStrategy {
    /// XOR de todas as camadas
    Xor,
    /// Soma complexa
    Sum,
    /// Primeira camada (L0)
    First,
    /// Última camada (LF)
    Last,
}

/// Estado completo SIL: 16 camadas × ByteSil = 128 bits
///
/// # Princípio
///
/// > *"Estado é sagrado — nunca modifique in-place, sempre crie novo."*
///
/// # Exemplo
///
/// ```
/// use sil_core::state::{SilState, ByteSil};
///
/// // Estado neutro (todas camadas em |z|=1, θ=0)
/// let state = SilState::neutral();
///
/// // Acessar camada específica
/// let photonic = state.get(0);
///
/// // Criar novo estado com camada modificada
/// let new_state = state.with_layer(0, ByteSil::new(3, 4));
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C, align(16))]
pub struct SilState {
    /// Array de 16 ByteSil, um por camada
    pub layers: [ByteSil; NUM_LAYERS],
}

impl SilState {
    // =========================================================================
    // Construtores
    // =========================================================================
    
    /// Estado vácuo: todas as camadas em null (ρ=-8, θ=0)
    pub const fn vacuum() -> Self {
        Self {
            layers: [ByteSil::NULL; NUM_LAYERS],
        }
    }
    
    /// Estado neutro: todas as camadas em ONE (ρ=0, θ=0)
    pub const fn neutral() -> Self {
        Self {
            layers: [ByteSil::ONE; NUM_LAYERS],
        }
    }
    
    /// Estado máximo: todas as camadas em MAX (ρ=7, θ=0)
    pub const fn maximum() -> Self {
        Self {
            layers: [ByteSil::MAX; NUM_LAYERS],
        }
    }
    
    /// Cria de array de ByteSil
    pub const fn from_layers(layers: [ByteSil; NUM_LAYERS]) -> Self {
        Self { layers }
    }
    
    /// Cria de array de bytes
    pub fn from_bytes(bytes: &[u8; NUM_LAYERS]) -> Self {
        let mut layers = [ByteSil::NULL; NUM_LAYERS];
        for (i, &byte) in bytes.iter().enumerate() {
            layers[i] = ByteSil::from_u8(byte);
        }
        Self { layers }
    }
    
    // =========================================================================
    // Acessores (Imutáveis)
    // =========================================================================
    
    /// Acessa camada por índice
    #[inline]
    pub const fn get(&self, index: usize) -> ByteSil {
        self.layers[index]
    }

    /// Alias para `get()` — acessa camada por índice
    #[inline]
    pub const fn layer(&self, index: usize) -> ByteSil {
        self.get(index)
    }
    
    /// Retorna novo estado com camada modificada (imutável)
    #[inline]
    pub fn with_layer(&self, index: usize, value: ByteSil) -> Self {
        let mut new = *self;
        new.layers[index] = value;
        new
    }
    
    /// Modifica camada no lugar (mutável)
    #[inline]
    pub fn set_layer(&mut self, index: usize, value: ByteSil) {
        self.layers[index] = value;
    }
    
    // =========================================================================
    // Grupos de Camadas
    // =========================================================================
    
    /// Camadas de percepção L(0-4)
    pub fn perception(&self) -> [ByteSil; 5] {
        [
            self.layers[layers::PHOTONIC],
            self.layers[layers::ACOUSTIC],
            self.layers[layers::OLFACTORY],
            self.layers[layers::GUSTATORY],
            self.layers[layers::DERMIC],
        ]
    }
    
    /// Camadas de processamento L(5-7)
    pub fn processing(&self) -> [ByteSil; 3] {
        [
            self.layers[layers::ELECTRONIC],
            self.layers[layers::PSYCHOMOTOR],
            self.layers[layers::ENVIRONMENTAL],
        ]
    }
    
    /// Camadas de interação L(8-A)
    pub fn interaction(&self) -> [ByteSil; 3] {
        [
            self.layers[layers::CYBERNETIC],
            self.layers[layers::GEOPOLITICAL],
            self.layers[layers::COSMOPOLITICAL],
        ]
    }
    
    /// Camadas de emergência L(B-C)
    pub fn emergence(&self) -> [ByteSil; 2] {
        [
            self.layers[layers::SYNERGIC],
            self.layers[layers::QUANTUM],
        ]
    }
    
    /// Camadas meta L(D-F)
    pub fn meta(&self) -> [ByteSil; 3] {
        [
            self.layers[layers::SUPERPOSITION],
            self.layers[layers::ENTANGLEMENT],
            self.layers[layers::COLLAPSE],
        ]
    }
    
    // =========================================================================
    // Operações sobre Estado
    // =========================================================================
    
    /// Produto tensorial (combinar dois estados)
    pub fn tensor(&self, other: &SilState) -> SilState {
        let mut layers = [ByteSil::NULL; NUM_LAYERS];
        for i in 0..NUM_LAYERS {
            layers[i] = self.layers[i].mul(&other.layers[i]);
        }
        SilState { layers }
    }
    
    /// XOR de dois estados
    pub fn xor(&self, other: &SilState) -> SilState {
        let mut layers = [ByteSil::NULL; NUM_LAYERS];
        for i in 0..NUM_LAYERS {
            layers[i] = self.layers[i].xor(&other.layers[i]);
        }
        SilState { layers }
    }
    
    /// Projeção (extrair subconjunto de camadas)
    pub fn project(&self, mask: u16) -> SilState {
        let mut layers = [ByteSil::NULL; NUM_LAYERS];
        for i in 0..NUM_LAYERS {
            if mask & (1 << i) != 0 {
                layers[i] = self.layers[i];
            }
        }
        SilState { layers }
    }
    
    /// Colapso (reduzir a uma camada)
    pub fn collapse(&self, strategy: CollapseStrategy) -> ByteSil {
        match strategy {
            CollapseStrategy::Xor => {
                self.layers.iter().fold(ByteSil::NULL, |a, b| a.xor(b))
            }
            CollapseStrategy::Sum => {
                let z: Complex<f64> = self.layers.iter()
                    .map(|l| l.to_complex())
                    .sum();
                ByteSil::from_complex(z)
            }
            CollapseStrategy::First => self.layers[0],
            CollapseStrategy::Last => self.layers[NUM_LAYERS - 1],
        }
    }
    
    /// Hash (fingerprint do estado) — 128 bits
    pub fn hash(&self) -> u128 {
        let mut h = 0u128;
        for (i, layer) in self.layers.iter().enumerate() {
            h |= (layer.to_u8() as u128) << (i * 8);
        }
        h
    }
    
    // =========================================================================
    // Serialização
    // =========================================================================
    
    /// Serializa para 16 bytes
    pub fn to_bytes(&self) -> [u8; NUM_LAYERS] {
        let mut bytes = [0u8; NUM_LAYERS];
        for (i, layer) in self.layers.iter().enumerate() {
            bytes[i] = layer.to_u8();
        }
        bytes
    }
}

// =============================================================================
// Traits
// =============================================================================

impl Default for SilState {
    fn default() -> Self {
        Self::neutral()
    }
}

impl fmt::Debug for SilState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SilState[")?;
        for (i, layer) in self.layers.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "L{:X}:{:?}", i, layer)?;
        }
        write!(f, "]")
    }
}

impl fmt::Display for SilState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "┌─────────────────────────────────────────────────┐")?;
        writeln!(f, "│ PERCEPÇÃO     │ L0:{} L1:{} L2:{} L3:{} L4:{}", 
            self.layers[0], self.layers[1], self.layers[2], self.layers[3], self.layers[4])?;
        writeln!(f, "│ PROCESSAMENTO │ L5:{} L6:{} L7:{}", 
            self.layers[5], self.layers[6], self.layers[7])?;
        writeln!(f, "│ INTERAÇÃO     │ L8:{} L9:{} LA:{}", 
            self.layers[8], self.layers[9], self.layers[10])?;
        writeln!(f, "│ EMERGÊNCIA    │ LB:{} LC:{}", 
            self.layers[11], self.layers[12])?;
        writeln!(f, "│ META          │ LD:{} LE:{} LF:{}", 
            self.layers[13], self.layers[14], self.layers[15])?;
        write!(f, "└─────────────────────────────────────────────────┘")
    }
}

// =============================================================================
// Serde
// =============================================================================

impl serde::Serialize for SilState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.layers.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for SilState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let layers = <[ByteSil; NUM_LAYERS]>::deserialize(deserializer)?;
        Ok(SilState { layers })
    }
}

// =============================================================================
// Testes
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vacuum() {
        let state = SilState::vacuum();
        for layer in &state.layers {
            assert!(layer.is_null());
        }
    }
    
    #[test]
    fn test_neutral() {
        let state = SilState::neutral();
        for layer in &state.layers {
            assert_eq!(*layer, ByteSil::ONE);
        }
    }
    
    #[test]
    fn test_with_layer_immutability() {
        let original = SilState::neutral();
        let modified = original.with_layer(0, ByteSil::new(5, 3));
        
        // Original não muda
        assert_eq!(original.layers[0], ByteSil::ONE);
        // Novo estado tem a camada modificada
        assert_eq!(modified.layers[0], ByteSil::new(5, 3));
    }
    
    #[test]
    fn test_serialization_roundtrip() {
        let original = SilState::neutral()
            .with_layer(0, ByteSil::new(3, 7))
            .with_layer(15, ByteSil::new(-2, 10));
        
        let bytes = original.to_bytes();
        let restored = SilState::from_bytes(&bytes);
        
        assert_eq!(original, restored);
    }
    
    #[test]
    fn test_xor_self_is_zero() {
        let state = SilState::neutral()
            .with_layer(5, ByteSil::new(3, 9));
        
        let xored = state.xor(&state);
        
        for layer in &xored.layers {
            assert_eq!(layer.rho, 0);
            assert_eq!(layer.theta, 0);
        }
    }
    
    #[test]
    fn test_collapse_xor() {
        let state = SilState::vacuum();
        let collapsed = state.collapse(CollapseStrategy::Xor);
        assert!(collapsed.is_null());
    }
    
    #[test]
    fn test_layer_groups() {
        let state = SilState::neutral();
        
        assert_eq!(state.perception().len(), 5);
        assert_eq!(state.processing().len(), 3);
        assert_eq!(state.interaction().len(), 3);
        assert_eq!(state.emergence().len(), 2);
        assert_eq!(state.meta().len(), 3);
        
        // Total = 16
        assert_eq!(5 + 3 + 3 + 2 + 3, NUM_LAYERS);
    }
}
