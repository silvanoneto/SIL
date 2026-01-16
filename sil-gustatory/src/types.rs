//! Tipos de dados gustativos

use serde::{Deserialize, Serialize};

/// Nível de pH (0.0 - 14.0)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PhLevel {
    /// Valor do pH (0.0 = ácido máximo, 7.0 = neutro, 14.0 = básico máximo)
    pub value: f32,
}

impl PhLevel {
    /// Cria novo valor de pH
    pub fn new(value: f32) -> Result<Self, &'static str> {
        if value < 0.0 || value > 14.0 {
            return Err("pH must be between 0.0 and 14.0");
        }
        Ok(Self { value })
    }

    /// Cria pH sem validação (usa clamp)
    pub fn clamped(value: f32) -> Self {
        Self {
            value: value.clamp(0.0, 14.0),
        }
    }

    /// Retorna pH normalizado para ρ (rho) range [-8, 7]
    /// pH 0-14 mapeia para rho -8 a +7
    pub fn to_rho(&self) -> i8 {
        // pH 0 → -8, pH 7 → 0, pH 14 → +7
        // Linear mapping: pH [0, 14] → rho [-8, 7]
        // Formula: rho = ((pH - 7) / 14) * 15 - 0.5
        let normalized = (self.value - 7.0) / 7.0; // -1.0 a +1.0
        let rho = (normalized * 7.5).round() as i8;
        rho.clamp(-8, 7)
    }

    /// Verifica se é ácido (pH < 7)
    pub fn is_acidic(&self) -> bool {
        self.value < 7.0
    }

    /// Verifica se é neutro (pH ≈ 7)
    pub fn is_neutral(&self) -> bool {
        (self.value - 7.0).abs() < 0.5
    }

    /// Verifica se é básico/alcalino (pH > 7)
    pub fn is_basic(&self) -> bool {
        self.value > 7.0
    }
}

impl Default for PhLevel {
    fn default() -> Self {
        Self { value: 7.0 } // Neutro
    }
}

/// Tipo de gosto básico
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TasteType {
    /// Doce (açúcares)
    Sweet,
    /// Azedo (ácidos)
    Sour,
    /// Salgado (sais)
    Salty,
    /// Amargo (alcaloides)
    Bitter,
    /// Umami (glutamato)
    Umami,
}

impl TasteType {
    /// Mapeia tipo de gosto para θ (theta) range [0, 255]
    /// Cada gosto ocupa ~51 unidades (255/5)
    pub fn to_theta(&self) -> u8 {
        match self {
            TasteType::Sweet => 0,    // 0-50
            TasteType::Sour => 51,    // 51-101
            TasteType::Salty => 102,  // 102-152
            TasteType::Bitter => 153, // 153-203
            TasteType::Umami => 204,  // 204-254
        }
    }

    /// Converte de θ (theta) para TasteType
    pub fn from_theta(theta: u8) -> Self {
        match theta {
            0..=50 => TasteType::Sweet,
            51..=101 => TasteType::Sour,
            102..=152 => TasteType::Salty,
            153..=203 => TasteType::Bitter,
            _ => TasteType::Umami,
        }
    }

    /// Retorna todos os tipos de gosto
    pub fn all() -> [TasteType; 5] {
        [
            TasteType::Sweet,
            TasteType::Sour,
            TasteType::Salty,
            TasteType::Bitter,
            TasteType::Umami,
        ]
    }
}

/// Perfil de gosto com intensidades para cada tipo básico
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TasteProfile {
    /// Intensidade de doce (0.0 - 1.0)
    pub sweet: f32,
    /// Intensidade de azedo (0.0 - 1.0)
    pub sour: f32,
    /// Intensidade de salgado (0.0 - 1.0)
    pub salty: f32,
    /// Intensidade de amargo (0.0 - 1.0)
    pub bitter: f32,
    /// Intensidade de umami (0.0 - 1.0)
    pub umami: f32,
}

impl TasteProfile {
    /// Cria perfil neutro (sem gosto)
    pub fn neutral() -> Self {
        Self {
            sweet: 0.0,
            sour: 0.0,
            salty: 0.0,
            bitter: 0.0,
            umami: 0.0,
        }
    }

    /// Cria perfil com um único gosto dominante
    pub fn single(taste: TasteType, intensity: f32) -> Self {
        let intensity = intensity.clamp(0.0, 1.0);
        let mut profile = Self::neutral();
        match taste {
            TasteType::Sweet => profile.sweet = intensity,
            TasteType::Sour => profile.sour = intensity,
            TasteType::Salty => profile.salty = intensity,
            TasteType::Bitter => profile.bitter = intensity,
            TasteType::Umami => profile.umami = intensity,
        }
        profile
    }

    /// Retorna o gosto dominante e sua intensidade
    pub fn dominant(&self) -> (TasteType, f32) {
        let tastes = [
            (TasteType::Sweet, self.sweet),
            (TasteType::Sour, self.sour),
            (TasteType::Salty, self.salty),
            (TasteType::Bitter, self.bitter),
            (TasteType::Umami, self.umami),
        ];

        tastes
            .iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|&(t, i)| (t, i))
            .unwrap_or((TasteType::Sweet, 0.0))
    }

    /// Retorna intensidade total (soma de todos os gostos)
    pub fn total_intensity(&self) -> f32 {
        self.sweet + self.sour + self.salty + self.bitter + self.umami
    }

    /// Normaliza todas as intensidades para somar 1.0
    pub fn normalize(&mut self) {
        let total = self.total_intensity();
        if total > 0.0 {
            self.sweet /= total;
            self.sour /= total;
            self.salty /= total;
            self.bitter /= total;
            self.umami /= total;
        }
    }

    /// Retorna intensidade de um tipo específico
    pub fn intensity_of(&self, taste: TasteType) -> f32 {
        match taste {
            TasteType::Sweet => self.sweet,
            TasteType::Sour => self.sour,
            TasteType::Salty => self.salty,
            TasteType::Bitter => self.bitter,
            TasteType::Umami => self.umami,
        }
    }
}

impl Default for TasteProfile {
    fn default() -> Self {
        Self::neutral()
    }
}

/// Salinidade (concentração de sal)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Salinity {
    /// Partes por milhão (ppm) ou mg/L
    pub ppm: f32,
}

impl Salinity {
    /// Cria nova salinidade
    pub fn new(ppm: f32) -> Self {
        Self {
            ppm: ppm.max(0.0),
        }
    }

    /// Salinidade da água doce (< 500 ppm)
    pub fn is_fresh_water(&self) -> bool {
        self.ppm < 500.0
    }

    /// Salinidade de água salobra (500-30000 ppm)
    pub fn is_brackish_water(&self) -> bool {
        self.ppm >= 500.0 && self.ppm < 30000.0
    }

    /// Salinidade de água salgada (>= 30000 ppm, ~35000 para oceano)
    pub fn is_salt_water(&self) -> bool {
        self.ppm >= 30000.0
    }

    /// Normaliza para range [0.0, 1.0] (0 = água doce, 1.0 = água do mar)
    pub fn normalized(&self) -> f32 {
        (self.ppm / 35000.0).clamp(0.0, 1.0)
    }
}

impl Default for Salinity {
    fn default() -> Self {
        Self { ppm: 0.0 }
    }
}

/// Dados completos de leitura gustativa
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TasteData {
    /// Nível de pH
    pub ph: PhLevel,
    /// Perfil de gosto
    pub profile: TasteProfile,
    /// Salinidade
    pub salinity: Salinity,
    /// Condutividade elétrica (µS/cm) - correlaciona com TDS
    pub conductivity: Option<f32>,
    /// Sólidos dissolvidos totais (TDS) em ppm
    pub tds: Option<f32>,
}

impl TasteData {
    /// Cria dados neutros (água pura)
    pub fn neutral() -> Self {
        Self {
            ph: PhLevel::default(),
            profile: TasteProfile::neutral(),
            salinity: Salinity::default(),
            conductivity: None,
            tds: None,
        }
    }

    /// Cria dados com pH e perfil
    pub fn new(ph: PhLevel, profile: TasteProfile) -> Self {
        Self {
            ph,
            profile,
            salinity: Salinity::default(),
            conductivity: None,
            tds: None,
        }
    }

    /// Cria dados completos
    pub fn full(
        ph: PhLevel,
        profile: TasteProfile,
        salinity: Salinity,
        conductivity: Option<f32>,
        tds: Option<f32>,
    ) -> Self {
        Self {
            ph,
            profile,
            salinity,
            conductivity,
            tds,
        }
    }

    /// Retorna gosto dominante
    pub fn dominant_taste(&self) -> TasteType {
        self.profile.dominant().0
    }

    /// Retorna intensidade do gosto dominante
    pub fn dominant_intensity(&self) -> f32 {
        self.profile.dominant().1
    }
}

impl Default for TasteData {
    fn default() -> Self {
        Self::neutral()
    }
}
