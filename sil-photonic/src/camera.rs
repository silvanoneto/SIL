//! Implementação de sensor de câmera (L0)

use serde::{Deserialize, Serialize};
use sil_core::prelude::*;
use sil_core::traits::{SilComponent, Sensor, SensorError};
use crate::error::{PhotonicError, PhotonicResult};
use crate::types::{ImageData, Pixel};

/// Configuração de câmera
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraConfig {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub auto_exposure: bool,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            width: 640,
            height: 480,
            fps: 30,
            auto_exposure: true,
        }
    }
}

/// Sensor de câmera fotônica (L0)
#[derive(Debug, Clone)]
pub struct CameraSensor {
    config: CameraConfig,
    ready: bool,
    frame_count: u64,
}

impl CameraSensor {
    /// Cria nova câmera com configuração padrão
    pub fn new() -> PhotonicResult<Self> {
        Self::with_config(CameraConfig::default())
    }

    /// Cria câmera com configuração específica
    pub fn with_config(config: CameraConfig) -> PhotonicResult<Self> {
        if config.width == 0 || config.height == 0 {
            return Err(PhotonicError::InvalidConfig(
                "Width and height must be > 0".into(),
            ));
        }

        Ok(Self {
            config,
            ready: true,
            frame_count: 0,
        })
    }

    /// Retorna resolução atual
    pub fn resolution(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }

    /// Retorna taxa de frames
    pub fn fps(&self) -> u32 {
        self.config.fps
    }

    /// Número de frames capturados
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }
}

impl Default for CameraSensor {
    fn default() -> Self {
        Self::new().expect("Default CameraSensor creation should not fail")
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// IMPLEMENTAÇÃO DOS TRAITS DO CORE
// ═══════════════════════════════════════════════════════════════════════════════

impl SilComponent for CameraSensor {
    fn name(&self) -> &str {
        "CameraSensor"
    }

    fn layers(&self) -> &[u8] {
        &[0] // L0 = Fotônico
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn is_ready(&self) -> bool {
        self.ready
    }
}

impl Sensor for CameraSensor {
    type RawData = ImageData;
    type Config = CameraConfig;

    fn configure(&mut self, config: Self::Config) -> Result<(), SensorError> {
        if config.width == 0 || config.height == 0 {
            return Err(SensorError::InvalidConfig(
                "Width and height must be > 0".into(),
            ));
        }
        self.config = config;
        Ok(())
    }

    fn read(&mut self) -> Result<Self::RawData, SensorError> {
        if !self.ready {
            return Err(SensorError::NotInitialized);
        }

        // Mock: gera imagem sintética
        // Em produção, aqui faria a captura real da câmera
        let data = self.generate_mock_frame();
        self.frame_count += 1;

        Ok(data)
    }

    fn to_byte_sil(&self, raw: &Self::RawData) -> ByteSil {
        // ρ (magnitude): intensidade média normalizada para [-8, 7]
        let intensity = raw.avg_intensity();
        let rho = ((intensity as f32 / 255.0) * 15.0 - 8.0) as i8;

        // θ (fase): hue médio mapeado para [0, 255]
        let hue = raw.avg_hue();
        let theta = ((hue / 360.0) * 255.0) as u8;

        ByteSil::new(rho.clamp(-8, 7), theta)
    }

    fn target_layer(&self) -> u8 {
        0 // L0
    }

    fn calibrate(&mut self) -> Result<(), SensorError> {
        // Calibração mock
        self.ready = true;
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// IMPLEMENTAÇÃO AUXILIAR
// ═══════════════════════════════════════════════════════════════════════════════

impl CameraSensor {
    /// Gera frame mock para testes
    fn generate_mock_frame(&self) -> ImageData {
        let mut pixels = Vec::with_capacity((self.config.width * self.config.height) as usize);

        // Gradiente horizontal de intensidade
        for y in 0..self.config.height {
            for x in 0..self.config.width {
                let intensity = ((x as f32 / self.config.width as f32) * 255.0) as u8;

                // Variação de cor baseada na linha
                let hue_shift = (y * 360 / self.config.height) as u8;

                let pixel = Pixel::new(
                    intensity,
                    intensity.saturating_sub(hue_shift / 3),
                    intensity.saturating_add(hue_shift / 3),
                );
                pixels.push(pixel);
            }
        }

        ImageData {
            width: self.config.width,
            height: self.config.height,
            pixels,
        }
    }
}
