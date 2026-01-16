//! Tipos de dados fotônicos

use serde::{Deserialize, Serialize};

/// Pixel RGB
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Pixel {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Retorna a intensidade média (grayscale)
    pub fn intensity(&self) -> u8 {
        ((self.r as u32 + self.g as u32 + self.b as u32) / 3) as u8
    }

    /// Retorna o hue (matiz) em graus [0..360)
    pub fn hue(&self) -> f32 {
        let r = self.r as f32 / 255.0;
        let g = self.g as f32 / 255.0;
        let b = self.b as f32 / 255.0;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;

        if delta < 0.001 {
            return 0.0; // Sem saturação
        }

        let hue = if (max - r).abs() < 0.001 {
            60.0 * (((g - b) / delta) % 6.0)
        } else if (max - g).abs() < 0.001 {
            60.0 * (((b - r) / delta) + 2.0)
        } else {
            60.0 * (((r - g) / delta) + 4.0)
        };

        if hue < 0.0 {
            hue + 360.0
        } else {
            hue
        }
    }
}

/// Dados de imagem capturados
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageData {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<Pixel>,
}

impl ImageData {
    /// Cria imagem vazia (preta)
    pub fn black(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            pixels: vec![Pixel::new(0, 0, 0); (width * height) as usize],
        }
    }

    /// Retorna pixel na posição (x, y)
    pub fn get(&self, x: u32, y: u32) -> Option<&Pixel> {
        if x >= self.width || y >= self.height {
            return None;
        }
        self.pixels.get((y * self.width + x) as usize)
    }

    /// Intensidade média da imagem
    pub fn avg_intensity(&self) -> u8 {
        if self.pixels.is_empty() {
            return 0;
        }
        let sum: u32 = self.pixels.iter().map(|p| p.intensity() as u32).sum();
        (sum / self.pixels.len() as u32) as u8
    }

    /// Hue médio da imagem
    pub fn avg_hue(&self) -> f32 {
        if self.pixels.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.pixels.iter().map(|p| p.hue()).sum();
        sum / self.pixels.len() as f32
    }
}

/// Intensidade de luz (lux ou normalizado)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Intensity {
    /// Valor em lux ou normalizado [0.0..1.0]
    pub value: f32,
}

impl Intensity {
    pub fn new(value: f32) -> Self {
        Self {
            value: value.clamp(0.0, 1e6),
        }
    }

    pub fn normalized(&self) -> f32 {
        (self.value / 1000.0).clamp(0.0, 1.0)
    }
}
