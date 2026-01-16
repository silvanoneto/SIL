//! Tipos de dados olfativos

use serde::{Deserialize, Serialize};

/// Tipo de gás/composto químico
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GasType {
    /// Monóxido de carbono
    CO,
    /// Dióxido de carbono
    CO2,
    /// Metano
    CH4,
    /// Amônia
    NH3,
    /// Compostos orgânicos voláteis
    VOC,
    /// Dióxido de nitrogênio
    NO2,
    /// Dióxido de enxofre
    SO2,
    /// Ozônio
    O3,
    /// Hidrogênio
    H2,
    /// Propano
    C3H8,
    /// Álcool (etanol)
    Alcohol,
    /// Gás natural
    NaturalGas,
    /// Outros compostos
    Other(u8),
}

impl GasType {
    /// Retorna ID numérico único para o tipo de gás
    pub fn id(&self) -> u8 {
        match self {
            GasType::CO => 0,
            GasType::CO2 => 1,
            GasType::CH4 => 2,
            GasType::NH3 => 3,
            GasType::VOC => 4,
            GasType::NO2 => 5,
            GasType::SO2 => 6,
            GasType::O3 => 7,
            GasType::H2 => 8,
            GasType::C3H8 => 9,
            GasType::Alcohol => 10,
            GasType::NaturalGas => 11,
            GasType::Other(id) => *id,
        }
    }

    /// Retorna nome legível do gás
    pub fn name(&self) -> &str {
        match self {
            GasType::CO => "Carbon Monoxide",
            GasType::CO2 => "Carbon Dioxide",
            GasType::CH4 => "Methane",
            GasType::NH3 => "Ammonia",
            GasType::VOC => "Volatile Organic Compounds",
            GasType::NO2 => "Nitrogen Dioxide",
            GasType::SO2 => "Sulfur Dioxide",
            GasType::O3 => "Ozone",
            GasType::H2 => "Hydrogen",
            GasType::C3H8 => "Propane",
            GasType::Alcohol => "Alcohol",
            GasType::NaturalGas => "Natural Gas",
            GasType::Other(_) => "Unknown Compound",
        }
    }

    /// Retorna faixa típica de concentração (em PPM)
    pub fn typical_range(&self) -> (f32, f32) {
        match self {
            GasType::CO => (0.0, 100.0),        // 0-100 PPM
            GasType::CO2 => (300.0, 5000.0),    // 300-5000 PPM
            GasType::CH4 => (0.0, 5000.0),      // 0-5000 PPM (LEL ~5%)
            GasType::NH3 => (0.0, 50.0),        // 0-50 PPM
            GasType::VOC => (0.0, 1000.0),      // 0-1000 PPM
            GasType::NO2 => (0.0, 5.0),         // 0-5 PPM
            GasType::SO2 => (0.0, 5.0),         // 0-5 PPM
            GasType::O3 => (0.0, 0.1),          // 0-0.1 PPM
            GasType::H2 => (0.0, 1000.0),       // 0-1000 PPM
            GasType::C3H8 => (0.0, 2000.0),     // 0-2000 PPM
            GasType::Alcohol => (0.0, 500.0),   // 0-500 PPM
            GasType::NaturalGas => (0.0, 1000.0), // 0-1000 PPM
            GasType::Other(_) => (0.0, 1000.0),
        }
    }

    /// Retorna nível de periculosidade (0 = seguro, 100 PPM = muito perigoso)
    pub fn danger_threshold(&self) -> f32 {
        match self {
            GasType::CO => 50.0,           // Perigoso acima de 50 PPM
            GasType::CO2 => 2000.0,        // Perigoso acima de 2000 PPM
            GasType::CH4 => 1000.0,        // Perigoso acima de 1000 PPM
            GasType::NH3 => 25.0,          // Perigoso acima de 25 PPM
            GasType::VOC => 500.0,         // Perigoso acima de 500 PPM
            GasType::NO2 => 1.0,           // Perigoso acima de 1 PPM
            GasType::SO2 => 2.0,           // Perigoso acima de 2 PPM
            GasType::O3 => 0.05,           // Perigoso acima de 0.05 PPM
            GasType::H2 => 1000.0,         // Perigoso acima de 1000 PPM
            GasType::C3H8 => 1000.0,       // Perigoso acima de 1000 PPM
            GasType::Alcohol => 200.0,     // Perigoso acima de 200 PPM
            GasType::NaturalGas => 500.0,  // Perigoso acima de 500 PPM
            GasType::Other(_) => 100.0,
        }
    }
}

/// Concentração de gás em PPM (partes por milhão)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GasConcentration {
    /// Tipo de gás
    pub gas_type: GasType,
    /// Concentração em PPM
    pub ppm: f32,
}

impl GasConcentration {
    pub fn new(gas_type: GasType, ppm: f32) -> Self {
        Self {
            gas_type,
            ppm: ppm.max(0.0),
        }
    }

    /// Retorna concentração normalizada [0.0, 1.0] baseada na faixa típica
    pub fn normalized(&self) -> f32 {
        let (min, max) = self.gas_type.typical_range();
        ((self.ppm - min) / (max - min)).clamp(0.0, 1.0)
    }

    /// Verifica se a concentração está em nível perigoso
    pub fn is_hazardous(&self) -> bool {
        self.ppm >= self.gas_type.danger_threshold()
    }

    /// Retorna nível de perigo [0.0, 1.0+]
    pub fn danger_level(&self) -> f32 {
        (self.ppm / self.gas_type.danger_threshold()).max(0.0)
    }
}

/// Classificação de odor
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OdorClass {
    /// Odor agradável (flores, frutas)
    Pleasant,
    /// Odor neutro (ar limpo)
    Neutral,
    /// Odor desagradável mas não perigoso (suor, lixo)
    Unpleasant,
    /// Odor perigoso (gases tóxicos)
    Hazardous,
}

impl OdorClass {
    /// Retorna ID numérico para classificação
    pub fn id(&self) -> u8 {
        match self {
            OdorClass::Pleasant => 0,
            OdorClass::Neutral => 1,
            OdorClass::Unpleasant => 2,
            OdorClass::Hazardous => 3,
        }
    }

    /// Classifica baseado em tipo de gás e concentração
    pub fn from_gas(gas: &GasConcentration) -> Self {
        if gas.is_hazardous() {
            OdorClass::Hazardous
        } else if gas.danger_level() > 0.5 {
            OdorClass::Unpleasant
        } else if gas.ppm < 1.0 {
            OdorClass::Neutral
        } else {
            // Para concentrações baixas de gases tóxicos, ainda é desagradável
            match gas.gas_type {
                GasType::CO | GasType::NO2 | GasType::SO2 | GasType::O3 | GasType::NH3 => {
                    if gas.ppm > 10.0 {
                        OdorClass::Unpleasant
                    } else {
                        OdorClass::Neutral
                    }
                }
                _ => OdorClass::Neutral,
            }
        }
    }
}

/// Perfil de odor (fingerprint químico)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OdorProfile {
    /// Vetor de concentrações de diferentes gases
    pub compounds: Vec<GasConcentration>,
    /// Classificação geral do odor
    pub classification: OdorClass,
    /// Índice de qualidade do ar (0-500)
    pub air_quality_index: u16,
}

impl OdorProfile {
    /// Cria perfil vazio (ar limpo)
    pub fn clean_air() -> Self {
        Self {
            compounds: vec![],
            classification: OdorClass::Neutral,
            air_quality_index: 0,
        }
    }

    /// Cria perfil a partir de concentrações
    pub fn from_compounds(compounds: Vec<GasConcentration>) -> Self {
        let classification = Self::classify(&compounds);
        let air_quality_index = Self::calculate_aqi(&compounds);

        Self {
            compounds,
            classification,
            air_quality_index,
        }
    }

    /// Classifica perfil geral
    fn classify(compounds: &[GasConcentration]) -> OdorClass {
        if compounds.is_empty() {
            return OdorClass::Neutral;
        }

        // Se qualquer composto é perigoso, o perfil é perigoso
        if compounds.iter().any(|c| c.is_hazardous()) {
            return OdorClass::Hazardous;
        }

        // Se maioria é desagradável
        let unpleasant_count = compounds
            .iter()
            .filter(|c| matches!(OdorClass::from_gas(c), OdorClass::Unpleasant))
            .count();

        if unpleasant_count > compounds.len() / 2 {
            OdorClass::Unpleasant
        } else {
            OdorClass::Neutral
        }
    }

    /// Calcula índice de qualidade do ar (AQI)
    fn calculate_aqi(compounds: &[GasConcentration]) -> u16 {
        if compounds.is_empty() {
            return 0; // Ar limpo
        }

        // AQI simplificado: soma ponderada dos níveis de perigo
        let total: f32 = compounds.iter().map(|c| c.danger_level() * 100.0).sum();
        let avg = total / compounds.len() as f32;

        (avg.min(500.0)) as u16
    }

    /// Retorna composto dominante (maior concentração relativa)
    pub fn dominant_compound(&self) -> Option<&GasConcentration> {
        self.compounds
            .iter()
            .max_by(|a, b| a.normalized().partial_cmp(&b.normalized()).unwrap())
    }

    /// Adiciona composto ao perfil
    pub fn add_compound(&mut self, compound: GasConcentration) {
        self.compounds.push(compound);
        self.classification = Self::classify(&self.compounds);
        self.air_quality_index = Self::calculate_aqi(&self.compounds);
    }
}

/// Assinatura química (fingerprint único)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChemicalSignature {
    /// Padrão de bits representando presença de compostos
    pub pattern: u64,
    /// Intensidade geral
    pub intensity: f32,
}

impl ChemicalSignature {
    pub fn new(pattern: u64, intensity: f32) -> Self {
        Self {
            pattern,
            intensity: intensity.clamp(0.0, 1.0),
        }
    }

    /// Cria assinatura a partir de perfil de odor
    pub fn from_profile(profile: &OdorProfile) -> Self {
        let mut pattern = 0u64;

        // Cada bit representa presença de um tipo de gás
        for compound in &profile.compounds {
            let bit = compound.gas_type.id() as u64;
            if bit < 64 {
                pattern |= 1 << bit;
            }
        }

        // Intensidade é a média das concentrações normalizadas
        let intensity = if profile.compounds.is_empty() {
            0.0
        } else {
            let sum: f32 = profile.compounds.iter().map(|c| c.normalized()).sum();
            sum / profile.compounds.len() as f32
        };

        Self::new(pattern, intensity)
    }

    /// Calcula similaridade com outra assinatura (0.0 = diferente, 1.0 = idêntica)
    pub fn similarity(&self, other: &Self) -> f32 {
        // Similaridade de padrão (Jaccard)
        let intersection = (self.pattern & other.pattern).count_ones();
        let union = (self.pattern | other.pattern).count_ones();

        let pattern_similarity = if union == 0 {
            1.0
        } else {
            intersection as f32 / union as f32
        };

        // Similaridade de intensidade
        let intensity_diff = (self.intensity - other.intensity).abs();
        let intensity_similarity = 1.0 - intensity_diff;

        // Média ponderada
        0.7 * pattern_similarity + 0.3 * intensity_similarity
    }
}

/// Dados de gás capturados pelo sensor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasData {
    /// Perfil completo de odor
    pub profile: OdorProfile,
    /// Assinatura química
    pub signature: ChemicalSignature,
    /// Temperatura ambiente (°C)
    pub temperature: f32,
    /// Umidade relativa (%)
    pub humidity: f32,
}

impl GasData {
    /// Cria dados de ar limpo
    pub fn clean_air(temperature: f32, humidity: f32) -> Self {
        let profile = OdorProfile::clean_air();
        let signature = ChemicalSignature::from_profile(&profile);

        Self {
            profile,
            signature,
            temperature,
            humidity,
        }
    }

    /// Cria dados a partir de compostos detectados
    pub fn from_compounds(
        compounds: Vec<GasConcentration>,
        temperature: f32,
        humidity: f32,
    ) -> Self {
        let profile = OdorProfile::from_compounds(compounds);
        let signature = ChemicalSignature::from_profile(&profile);

        Self {
            profile,
            signature,
            temperature,
            humidity,
        }
    }

    /// Retorna composto dominante
    pub fn dominant_compound(&self) -> Option<&GasConcentration> {
        self.profile.dominant_compound()
    }

    /// Retorna concentração composta normalizada para SIL rho
    /// Mapeia AQI [0, 500] para rho [-8, 7]
    pub fn composite_concentration(&self) -> f32 {
        let aqi = self.profile.air_quality_index as f32;
        // AQI 0 = -8, AQI 500 = +7
        ((aqi / 500.0) * 15.0 - 8.0).clamp(-8.0, 7.0)
    }

    /// Retorna ID do composto dominante ou classificação
    pub fn dominant_signature(&self) -> u8 {
        if let Some(compound) = self.dominant_compound() {
            compound.gas_type.id()
        } else {
            self.profile.classification.id()
        }
    }

    /// Verifica se há gases perigosos presentes
    pub fn is_hazardous(&self) -> bool {
        matches!(self.profile.classification, OdorClass::Hazardous)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gas_type_id() {
        assert_eq!(GasType::CO.id(), 0);
        assert_eq!(GasType::CO2.id(), 1);
        assert_eq!(GasType::Other(42).id(), 42);
    }

    #[test]
    fn test_gas_concentration() {
        let co = GasConcentration::new(GasType::CO, 25.0);
        assert_eq!(co.ppm, 25.0);
        assert!(!co.is_hazardous()); // 25 < 50

        let co_danger = GasConcentration::new(GasType::CO, 75.0);
        assert!(co_danger.is_hazardous()); // 75 > 50
    }

    #[test]
    fn test_odor_classification() {
        let safe_co = GasConcentration::new(GasType::CO, 10.0);
        // CO at 10 PPM: not hazardous, but danger_level = 10/50 = 0.2, so neutral
        assert_eq!(OdorClass::from_gas(&safe_co), OdorClass::Neutral);

        let danger_co = GasConcentration::new(GasType::CO, 60.0);
        assert_eq!(OdorClass::from_gas(&danger_co), OdorClass::Hazardous);

        let unpleasant_co = GasConcentration::new(GasType::CO, 30.0);
        // CO at 30 PPM: danger_level = 30/50 = 0.6 > 0.5, so unpleasant
        assert_eq!(OdorClass::from_gas(&unpleasant_co), OdorClass::Unpleasant);
    }

    #[test]
    fn test_odor_profile() {
        let compounds = vec![
            GasConcentration::new(GasType::CO2, 400.0),
            GasConcentration::new(GasType::VOC, 50.0),
        ];

        let profile = OdorProfile::from_compounds(compounds);
        assert_eq!(profile.compounds.len(), 2);
        assert!(profile.air_quality_index < 100); // Baixo

        let dominant = profile.dominant_compound();
        assert!(dominant.is_some());
    }

    #[test]
    fn test_chemical_signature() {
        let compounds = vec![
            GasConcentration::new(GasType::CO, 10.0),
            GasConcentration::new(GasType::CO2, 400.0),
        ];
        let profile = OdorProfile::from_compounds(compounds);
        let sig = ChemicalSignature::from_profile(&profile);

        // Bit 0 (CO) e bit 1 (CO2) devem estar setados
        assert!(sig.pattern & 0b11 == 0b11);
        assert!(sig.intensity > 0.0);
    }

    #[test]
    fn test_signature_similarity() {
        let sig1 = ChemicalSignature::new(0b1111, 0.5);
        let sig2 = ChemicalSignature::new(0b1111, 0.5);
        assert_eq!(sig1.similarity(&sig2), 1.0);

        let sig3 = ChemicalSignature::new(0b0000, 0.0);
        assert!(sig1.similarity(&sig3) < 0.5);
    }

    #[test]
    fn test_gas_data() {
        let data = GasData::clean_air(20.0, 50.0);
        assert_eq!(data.temperature, 20.0);
        assert_eq!(data.humidity, 50.0);
        assert!(!data.is_hazardous());
        assert_eq!(data.profile.classification, OdorClass::Neutral);
    }
}
