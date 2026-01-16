//! Transforms — Transformações para I/O SIL
//!
//! Transformações que podem ser encadeadas em pipelines.

use super::SilBuffer;
use crate::state::ByteSil;

/// Trait para transformações de buffer
pub trait SilTransformFn: Send + Sync {
    /// Aplica transformação em buffer
    fn apply(&self, buffer: &SilBuffer) -> SilBuffer;
    
    /// Aplica transformação em único byte
    fn apply_byte(&self, byte: ByteSil) -> ByteSil;
    
    /// Nome da transformação
    fn name(&self) -> &'static str;
    
    /// Clona transformação em Box
    fn clone_box(&self) -> Box<dyn SilTransformFn>;
}

// =============================================================================
// XOR Transform
// =============================================================================

/// XOR com chave fixa
#[derive(Debug, Clone, Copy)]
pub struct Xor(pub u8);

impl SilTransformFn for Xor {
    fn apply(&self, buffer: &SilBuffer) -> SilBuffer {
        buffer.iter().map(|b| self.apply_byte(*b)).collect()
    }
    
    fn apply_byte(&self, byte: ByteSil) -> ByteSil {
        ByteSil::from_u8(byte.to_u8() ^ self.0)
    }
    
    fn name(&self) -> &'static str {
        "Xor"
    }
    
    fn clone_box(&self) -> Box<dyn SilTransformFn> {
        Box::new(*self)
    }
}

/// XOR com chave cíclica (múltiplos bytes)
#[derive(Debug, Clone)]
pub struct XorKey {
    key: Vec<u8>,
}

impl XorKey {
    pub fn new(key: &[u8]) -> Self {
        Self { key: key.to_vec() }
    }
    
    pub fn from_str(key: &str) -> Self {
        Self { key: key.as_bytes().to_vec() }
    }
}

impl SilTransformFn for XorKey {
    fn apply(&self, buffer: &SilBuffer) -> SilBuffer {
        if self.key.is_empty() {
            return buffer.clone();
        }
        buffer.iter()
            .enumerate()
            .map(|(i, b)| {
                let k = self.key[i % self.key.len()];
                ByteSil::from_u8(b.to_u8() ^ k)
            })
            .collect()
    }
    
    fn apply_byte(&self, byte: ByteSil) -> ByteSil {
        if self.key.is_empty() {
            byte
        } else {
            ByteSil::from_u8(byte.to_u8() ^ self.key[0])
        }
    }
    
    fn name(&self) -> &'static str {
        "XorKey"
    }
    
    fn clone_box(&self) -> Box<dyn SilTransformFn> {
        Box::new(self.clone())
    }
}

// =============================================================================
// Rotate Transform (Fase)
// =============================================================================

/// Rotação de fase
#[derive(Debug, Clone, Copy)]
pub struct Rotate(pub u8);

impl SilTransformFn for Rotate {
    fn apply(&self, buffer: &SilBuffer) -> SilBuffer {
        buffer.iter().map(|b| self.apply_byte(*b)).collect()
    }
    
    fn apply_byte(&self, byte: ByteSil) -> ByteSil {
        ByteSil::new(byte.rho, (byte.theta + self.0) % 16)
    }
    
    fn name(&self) -> &'static str {
        "Rotate"
    }
    
    fn clone_box(&self) -> Box<dyn SilTransformFn> {
        Box::new(*self)
    }
}

/// Rotação inversa de fase
#[derive(Debug, Clone, Copy)]
pub struct RotateInv(pub u8);

impl SilTransformFn for RotateInv {
    fn apply(&self, buffer: &SilBuffer) -> SilBuffer {
        buffer.iter().map(|b| self.apply_byte(*b)).collect()
    }
    
    fn apply_byte(&self, byte: ByteSil) -> ByteSil {
        ByteSil::new(byte.rho, (byte.theta + 16 - (self.0 % 16)) % 16)
    }
    
    fn name(&self) -> &'static str {
        "RotateInv"
    }
    
    fn clone_box(&self) -> Box<dyn SilTransformFn> {
        Box::new(*self)
    }
}

// =============================================================================
// Scale Transform (Magnitude)
// =============================================================================

/// Escala de magnitude (bijetivo com wrap-around)
/// 
/// Diferente do SCALE do VSP que satura, este faz wrap-around
/// para garantir bijetividade (reversibilidade perfeita).
#[derive(Debug, Clone, Copy)]
pub struct Scale(pub i8);

impl SilTransformFn for Scale {
    fn apply(&self, buffer: &SilBuffer) -> SilBuffer {
        buffer.iter().map(|b| self.apply_byte(*b)).collect()
    }
    
    fn apply_byte(&self, byte: ByteSil) -> ByteSil {
        // Wrap-around bijetivo: rho vai de -8 a 7 (16 valores)
        // Usamos módulo para garantir reversibilidade
        let rho_normalized = (byte.rho - ByteSil::RHO_MIN) as i16; // 0..15
        let new_rho_normalized = ((rho_normalized + self.0 as i16) % 16 + 16) % 16;
        let new_rho = (new_rho_normalized as i8) + ByteSil::RHO_MIN;
        ByteSil::new(new_rho, byte.theta)
    }
    
    fn name(&self) -> &'static str {
        "Scale"
    }
    
    fn clone_box(&self) -> Box<dyn SilTransformFn> {
        Box::new(*self)
    }
}

/// Escala de magnitude com saturação (não bijetivo)
/// 
/// Use apenas quando saturação é desejada (ex: normalização).
/// Para pipelines reversíveis, use Scale (wrap-around).
#[derive(Debug, Clone, Copy)]
pub struct ScaleSaturate(pub i8);

impl SilTransformFn for ScaleSaturate {
    fn apply(&self, buffer: &SilBuffer) -> SilBuffer {
        buffer.iter().map(|b| self.apply_byte(*b)).collect()
    }
    
    fn apply_byte(&self, byte: ByteSil) -> ByteSil {
        let new_rho = (byte.rho as i16 + self.0 as i16)
            .clamp(ByteSil::RHO_MIN as i16, ByteSil::RHO_MAX as i16) as i8;
        ByteSil::new(new_rho, byte.theta)
    }
    
    fn name(&self) -> &'static str {
        "ScaleSaturate"
    }
    
    fn clone_box(&self) -> Box<dyn SilTransformFn> {
        Box::new(*self)
    }
}

// =============================================================================
// Outras Transforms
// =============================================================================

/// Inversão (negação de magnitude)
#[derive(Debug, Clone, Copy, Default)]
pub struct Invert;

impl SilTransformFn for Invert {
    fn apply(&self, buffer: &SilBuffer) -> SilBuffer {
        buffer.iter().map(|b| self.apply_byte(*b)).collect()
    }
    
    fn apply_byte(&self, byte: ByteSil) -> ByteSil {
        byte.inv()
    }
    
    fn name(&self) -> &'static str {
        "Invert"
    }
    
    fn clone_box(&self) -> Box<dyn SilTransformFn> {
        Box::new(*self)
    }
}

/// Conjugado (inversão de fase)
#[derive(Debug, Clone, Copy, Default)]
pub struct Conjugate;

impl SilTransformFn for Conjugate {
    fn apply(&self, buffer: &SilBuffer) -> SilBuffer {
        buffer.iter().map(|b| self.apply_byte(*b)).collect()
    }
    
    fn apply_byte(&self, byte: ByteSil) -> ByteSil {
        byte.conj()
    }
    
    fn name(&self) -> &'static str {
        "Conjugate"
    }
    
    fn clone_box(&self) -> Box<dyn SilTransformFn> {
        Box::new(*self)
    }
}

/// Identidade (não faz nada)
#[derive(Debug, Clone, Copy, Default)]
pub struct Identity;

impl SilTransformFn for Identity {
    fn apply(&self, buffer: &SilBuffer) -> SilBuffer {
        buffer.clone()
    }
    
    fn apply_byte(&self, byte: ByteSil) -> ByteSil {
        byte
    }
    
    fn name(&self) -> &'static str {
        "Identity"
    }
    
    fn clone_box(&self) -> Box<dyn SilTransformFn> {
        Box::new(*self)
    }
}

/// NOT bitwise
#[derive(Debug, Clone, Copy, Default)]
pub struct Not;

impl SilTransformFn for Not {
    fn apply(&self, buffer: &SilBuffer) -> SilBuffer {
        buffer.iter().map(|b| self.apply_byte(*b)).collect()
    }
    
    fn apply_byte(&self, byte: ByteSil) -> ByteSil {
        ByteSil::from_u8(!byte.to_u8())
    }
    
    fn name(&self) -> &'static str {
        "Not"
    }
    
    fn clone_box(&self) -> Box<dyn SilTransformFn> {
        Box::new(*self)
    }
}

/// AND com máscara
#[derive(Debug, Clone, Copy)]
pub struct And(pub u8);

impl SilTransformFn for And {
    fn apply(&self, buffer: &SilBuffer) -> SilBuffer {
        buffer.iter().map(|b| self.apply_byte(*b)).collect()
    }
    
    fn apply_byte(&self, byte: ByteSil) -> ByteSil {
        ByteSil::from_u8(byte.to_u8() & self.0)
    }
    
    fn name(&self) -> &'static str {
        "And"
    }
    
    fn clone_box(&self) -> Box<dyn SilTransformFn> {
        Box::new(*self)
    }
}

/// OR com máscara
#[derive(Debug, Clone, Copy)]
pub struct Or(pub u8);

impl SilTransformFn for Or {
    fn apply(&self, buffer: &SilBuffer) -> SilBuffer {
        buffer.iter().map(|b| self.apply_byte(*b)).collect()
    }
    
    fn apply_byte(&self, byte: ByteSil) -> ByteSil {
        ByteSil::from_u8(byte.to_u8() | self.0)
    }
    
    fn name(&self) -> &'static str {
        "Or"
    }
    
    fn clone_box(&self) -> Box<dyn SilTransformFn> {
        Box::new(*self)
    }
}

/// Swap nibbles (high <-> low)
#[derive(Debug, Clone, Copy, Default)]
pub struct SwapNibbles;

impl SilTransformFn for SwapNibbles {
    fn apply(&self, buffer: &SilBuffer) -> SilBuffer {
        buffer.iter().map(|b| self.apply_byte(*b)).collect()
    }
    
    fn apply_byte(&self, byte: ByteSil) -> ByteSil {
        let b = byte.to_u8();
        ByteSil::from_u8((b >> 4) | (b << 4))
    }
    
    fn name(&self) -> &'static str {
        "SwapNibbles"
    }
    
    fn clone_box(&self) -> Box<dyn SilTransformFn> {
        Box::new(*self)
    }
}

/// Shift left
#[derive(Debug, Clone, Copy)]
pub struct ShiftLeft(pub u8);

impl SilTransformFn for ShiftLeft {
    fn apply(&self, buffer: &SilBuffer) -> SilBuffer {
        buffer.iter().map(|b| self.apply_byte(*b)).collect()
    }
    
    fn apply_byte(&self, byte: ByteSil) -> ByteSil {
        ByteSil::from_u8(byte.to_u8() << (self.0 & 7))
    }
    
    fn name(&self) -> &'static str {
        "ShiftLeft"
    }
    
    fn clone_box(&self) -> Box<dyn SilTransformFn> {
        Box::new(*self)
    }
}

/// Shift right
#[derive(Debug, Clone, Copy)]
pub struct ShiftRight(pub u8);

impl SilTransformFn for ShiftRight {
    fn apply(&self, buffer: &SilBuffer) -> SilBuffer {
        buffer.iter().map(|b| self.apply_byte(*b)).collect()
    }
    
    fn apply_byte(&self, byte: ByteSil) -> ByteSil {
        ByteSil::from_u8(byte.to_u8() >> (self.0 & 7))
    }
    
    fn name(&self) -> &'static str {
        "ShiftRight"
    }
    
    fn clone_box(&self) -> Box<dyn SilTransformFn> {
        Box::new(*self)
    }
}

/// Rotate left (circular)
#[derive(Debug, Clone, Copy)]
pub struct RotateLeft(pub u8);

impl SilTransformFn for RotateLeft {
    fn apply(&self, buffer: &SilBuffer) -> SilBuffer {
        buffer.iter().map(|b| self.apply_byte(*b)).collect()
    }
    
    fn apply_byte(&self, byte: ByteSil) -> ByteSil {
        let b = byte.to_u8();
        let n = self.0 & 7;
        ByteSil::from_u8((b << n) | (b >> (8 - n)))
    }
    
    fn name(&self) -> &'static str {
        "RotateLeft"
    }
    
    fn clone_box(&self) -> Box<dyn SilTransformFn> {
        Box::new(*self)
    }
}

/// Rotate right (circular)
#[derive(Debug, Clone, Copy)]
pub struct RotateRight(pub u8);

impl SilTransformFn for RotateRight {
    fn apply(&self, buffer: &SilBuffer) -> SilBuffer {
        buffer.iter().map(|b| self.apply_byte(*b)).collect()
    }
    
    fn apply_byte(&self, byte: ByteSil) -> ByteSil {
        let b = byte.to_u8();
        let n = self.0 & 7;
        ByteSil::from_u8((b >> n) | (b << (8 - n)))
    }
    
    fn name(&self) -> &'static str {
        "RotateRight"
    }
    
    fn clone_box(&self) -> Box<dyn SilTransformFn> {
        Box::new(*self)
    }
}

// =============================================================================
// Transform Combinators
// =============================================================================

/// Compõe duas transformações
pub struct Compose<A, B> {
    first: A,
    second: B,
}

impl<A: SilTransformFn, B: SilTransformFn> Compose<A, B> {
    pub fn new(first: A, second: B) -> Self {
        Self { first, second }
    }
}

impl<A: SilTransformFn, B: SilTransformFn> SilTransformFn for Compose<A, B> {
    fn apply(&self, buffer: &SilBuffer) -> SilBuffer {
        let intermediate = self.first.apply(buffer);
        self.second.apply(&intermediate)
    }
    
    fn apply_byte(&self, byte: ByteSil) -> ByteSil {
        self.second.apply_byte(self.first.apply_byte(byte))
    }
    
    fn name(&self) -> &'static str {
        "Compose"
    }
    
    fn clone_box(&self) -> Box<dyn SilTransformFn> {
        // Não podemos clonar tipos genéricos para Box<dyn>
        // Use transformações concretas no pipeline
        unimplemented!("Compose não suporta clone_box - use transformações concretas")
    }
}

/// Aplica transformação N vezes
pub struct Repeat<T> {
    transform: T,
    times: usize,
}

impl<T: SilTransformFn> Repeat<T> {
    pub fn new(transform: T, times: usize) -> Self {
        Self { transform, times }
    }
}

impl<T: SilTransformFn> SilTransformFn for Repeat<T> {
    fn apply(&self, buffer: &SilBuffer) -> SilBuffer {
        let mut result = buffer.clone();
        for _ in 0..self.times {
            result = self.transform.apply(&result);
        }
        result
    }
    
    fn apply_byte(&self, byte: ByteSil) -> ByteSil {
        let mut result = byte;
        for _ in 0..self.times {
            result = self.transform.apply_byte(result);
        }
        result
    }
    
    fn name(&self) -> &'static str {
        "Repeat"
    }
    
    fn clone_box(&self) -> Box<dyn SilTransformFn> {
        // Não podemos clonar tipos genéricos para Box<dyn>
        // Use transformações concretas no pipeline
        unimplemented!("Repeat não suporta clone_box - use transformações concretas")
    }
}

// =============================================================================
// Funções de conveniência
// =============================================================================

/// Cria XOR transform
pub fn xor(key: u8) -> Xor {
    Xor(key)
}

/// Cria XOR com chave múltipla
pub fn xor_key(key: &[u8]) -> XorKey {
    XorKey::new(key)
}

/// Cria rotação de fase
pub fn rotate(delta: u8) -> Rotate {
    Rotate(delta)
}

/// Cria escala de magnitude
pub fn scale(delta: i8) -> Scale {
    Scale(delta)
}

/// Compõe duas transformações
pub fn compose<A: SilTransformFn, B: SilTransformFn>(a: A, b: B) -> Compose<A, B> {
    Compose::new(a, b)
}

/// Repete transformação N vezes
pub fn repeat<T: SilTransformFn>(t: T, n: usize) -> Repeat<T> {
    Repeat::new(t, n)
}

// =============================================================================
// Testes
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_xor_roundtrip() {
        let original = ByteSil::from_u8(0x48);
        let xor = Xor(0x5A);
        let encrypted = xor.apply_byte(original);
        let decrypted = xor.apply_byte(encrypted);
        assert_eq!(decrypted.to_u8(), original.to_u8());
    }
    
    #[test]
    fn test_xor_key() {
        let buffer = SilBuffer::from_str("Hello");
        let key = XorKey::from_str("key");
        let encrypted = key.apply(&buffer);
        let decrypted = key.apply(&encrypted);
        assert_eq!(decrypted.to_bytes(), buffer.to_bytes());
    }
    
    #[test]
    fn test_rotate_phase() {
        let original = ByteSil::new(0, 0);
        let rotated = Rotate(4).apply_byte(original);
        assert_eq!(rotated.theta, 4);
        
        // Rotação completa volta ao original
        let full = Rotate(16).apply_byte(original);
        assert_eq!(full.theta, original.theta);
    }
    
    #[test]
    fn test_scale_bijective() {
        // Scale agora é bijetivo (wrap-around)
        // Testar com todos os 256 valores possíveis
        for byte_val in 0..=255u8 {
            let original = ByteSil::from_u8(byte_val);
            
            // Scale +5
            let scaled = Scale(5).apply_byte(original);
            // Scale -5 deve restaurar
            let restored = Scale(-5).apply_byte(scaled);
            
            assert_eq!(
                restored.to_u8(), original.to_u8(),
                "Scale não é bijetivo para byte 0x{:02X}", byte_val
            );
        }
    }
    
    #[test]
    fn test_scale_wrap_around() {
        // Testar wrap-around explícito
        let max = ByteSil::new(ByteSil::RHO_MAX, 0); // rho = 7
        let scaled = Scale(1).apply_byte(max);
        assert_eq!(scaled.rho, ByteSil::RHO_MIN); // Deve ir para -8
        
        let min = ByteSil::new(ByteSil::RHO_MIN, 0); // rho = -8
        let scaled = Scale(-1).apply_byte(min);
        assert_eq!(scaled.rho, ByteSil::RHO_MAX); // Deve ir para 7
    }
    
    #[test]
    fn test_scale_saturate() {
        // ScaleSaturate ainda satura
        let max = ByteSil::new(ByteSil::RHO_MAX, 0);
        let scaled = ScaleSaturate(10).apply_byte(max);
        assert_eq!(scaled.rho, ByteSil::RHO_MAX); // Saturado em 7
        
        let min = ByteSil::new(ByteSil::RHO_MIN, 0);
        let scaled = ScaleSaturate(-10).apply_byte(min);
        assert_eq!(scaled.rho, ByteSil::RHO_MIN); // Saturado em -8
    }
    
    #[test]
    fn test_swap_nibbles() {
        let original = ByteSil::from_u8(0x48);
        let swapped = SwapNibbles.apply_byte(original);
        assert_eq!(swapped.to_u8(), 0x84);
        
        // Duplo swap restaura
        let restored = SwapNibbles.apply_byte(swapped);
        assert_eq!(restored.to_u8(), original.to_u8());
    }
    
    #[test]
    fn test_compose() {
        let buffer = SilBuffer::from_str("Hi");
        let t = compose(Xor(0x5A), Rotate(4));
        let result = t.apply(&buffer);
        assert_eq!(result.len(), 2);
    }
    
    #[test]
    fn test_repeat() {
        let original = ByteSil::new(0, 0);
        let t = repeat(Rotate(4), 4); // 4 * 4 = 16 = volta
        let result = t.apply_byte(original);
        assert_eq!(result.theta, original.theta);
    }
}
