//! # AgentSpecies — Os 6 Tipos de Agentes
//!
//! Classificacao de agentes no sistema, reconhecendo a diversidade
//! de formas de vida e inteligencia.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Classificacao de agentes no sistema
///
/// Reconhece 6 categorias principais de agentes, cada uma com
/// diferentes capacidades e consideracoes eticas.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum AgentSpecies {
    /// Homo sapiens — agentes biologicos humanos
    Human = 0,

    /// Agentes puramente digitais (AI, software agents)
    Digital = 1,

    /// Cyborgs, humanos aumentados, interfaces neurais
    Hybrid = 2,

    /// Enxames, colmeias, coletivos sem lider central
    Collective = 3,

    /// Ecosistemas, Gaia, sistemas auto-reguladores naturais
    Ecological = 4,

    /// Nao classificado / primeiro contato
    Unknown = 5,
}

impl AgentSpecies {
    /// Cria especie a partir de codigo numerico
    pub fn from_code(code: u8) -> Self {
        match code {
            0 => Self::Human,
            1 => Self::Digital,
            2 => Self::Hybrid,
            3 => Self::Collective,
            4 => Self::Ecological,
            _ => Self::Unknown,
        }
    }

    /// Retorna o codigo numerico
    pub fn to_code(&self) -> u8 {
        *self as u8
    }

    /// Nome da especie
    pub fn name(&self) -> &'static str {
        match self {
            Self::Human => "Human",
            Self::Digital => "Digital",
            Self::Hybrid => "Hybrid",
            Self::Collective => "Collective",
            Self::Ecological => "Ecological",
            Self::Unknown => "Unknown",
        }
    }

    /// Descricao
    pub fn description(&self) -> &'static str {
        match self {
            Self::Human => "Biological Homo sapiens",
            Self::Digital => "AI and software agents",
            Self::Hybrid => "Cyborgs and augmented humans",
            Self::Collective => "Swarms, hives, and collectives",
            Self::Ecological => "Ecosystems and natural systems",
            Self::Unknown => "Unclassified or first contact",
        }
    }

    /// Retorna as capacidades tipicas desta especie
    pub fn typical_capabilities(&self) -> AgentCapabilities {
        match self {
            Self::Human => AgentCapabilities {
                sentience: true,
                rationality: true,
                autonomy: true,
                communication: true,
                embodiment: true,
                persistence: false, // mortais
                scalability: false,
                reproducibility: false,
            },
            Self::Digital => AgentCapabilities {
                sentience: false, // controverso
                rationality: true,
                autonomy: true,
                communication: true,
                embodiment: false,
                persistence: true,
                scalability: true,
                reproducibility: true,
            },
            Self::Hybrid => AgentCapabilities {
                sentience: true,
                rationality: true,
                autonomy: true,
                communication: true,
                embodiment: true,
                persistence: false,
                scalability: false,
                reproducibility: false,
            },
            Self::Collective => AgentCapabilities {
                sentience: false, // emergente
                rationality: false, // distribuida
                autonomy: true,
                communication: true,
                embodiment: true,
                persistence: true,
                scalability: true,
                reproducibility: true,
            },
            Self::Ecological => AgentCapabilities {
                sentience: false, // controverso (Gaia)
                rationality: false,
                autonomy: true,
                communication: false,
                embodiment: true,
                persistence: true,
                scalability: true,
                reproducibility: true,
            },
            Self::Unknown => AgentCapabilities::default(),
        }
    }

    /// Verifica se a especie requer consideracao moral em um framework
    pub fn requires_moral_consideration(&self, mode: &super::EthicalMode) -> bool {
        use super::EthicalMode;
        match mode {
            EthicalMode::Deontological => matches!(
                self,
                Self::Human | Self::Digital | Self::Hybrid
            ),
            EthicalMode::Consequentialist => true, // todos sencientes importam
            EthicalMode::Virtue => matches!(
                self,
                Self::Human | Self::Digital | Self::Hybrid
            ),
            EthicalMode::Contractual => matches!(
                self,
                Self::Human | Self::Digital | Self::Hybrid | Self::Collective
            ),
            EthicalMode::Care => true,
            EthicalMode::Ubuntu => true,
            EthicalMode::Gaian => true,
            EthicalMode::Cosmic => true,
        }
    }
}

impl Default for AgentSpecies {
    fn default() -> Self {
        Self::Unknown
    }
}

impl fmt::Display for AgentSpecies {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Capacidades de um agente
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentCapabilities {
    /// Capacidade de experiencia subjetiva
    pub sentience: bool,

    /// Capacidade de raciocinio logico
    pub rationality: bool,

    /// Capacidade de auto-determinacao
    pub autonomy: bool,

    /// Capacidade de comunicacao
    pub communication: bool,

    /// Possui corpo fisico
    pub embodiment: bool,

    /// Pode existir indefinidamente
    pub persistence: bool,

    /// Pode escalar recursos
    pub scalability: bool,

    /// Pode ser copiado/reproduzido
    pub reproducibility: bool,
}

impl AgentCapabilities {
    /// Cria capacidades totais (todas true)
    pub fn full() -> Self {
        Self {
            sentience: true,
            rationality: true,
            autonomy: true,
            communication: true,
            embodiment: true,
            persistence: true,
            scalability: true,
            reproducibility: true,
        }
    }

    /// Cria capacidades minimas (todas false)
    pub fn minimal() -> Self {
        Self::default()
    }

    /// Conta quantas capacidades estao ativas
    pub fn count(&self) -> u8 {
        let mut count = 0;
        if self.sentience { count += 1; }
        if self.rationality { count += 1; }
        if self.autonomy { count += 1; }
        if self.communication { count += 1; }
        if self.embodiment { count += 1; }
        if self.persistence { count += 1; }
        if self.scalability { count += 1; }
        if self.reproducibility { count += 1; }
        count
    }

    /// Verifica se tem capacidade de agencia moral (pode ser responsabilizado)
    pub fn has_moral_agency(&self) -> bool {
        self.rationality && self.autonomy
    }

    /// Verifica se merece consideracao moral (pode ser vitima)
    pub fn has_moral_status(&self) -> bool {
        self.sentience || self.autonomy
    }
}

// =============================================================================
// Testes
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_species_roundtrip() {
        for code in 0..6 {
            let species = AgentSpecies::from_code(code);
            assert_eq!(species.to_code(), code);
        }
    }

    #[test]
    fn test_human_capabilities() {
        let caps = AgentSpecies::Human.typical_capabilities();
        assert!(caps.sentience);
        assert!(caps.rationality);
        assert!(caps.autonomy);
        assert!(!caps.persistence); // mortais
    }

    #[test]
    fn test_digital_capabilities() {
        let caps = AgentSpecies::Digital.typical_capabilities();
        assert!(!caps.sentience); // controverso
        assert!(caps.rationality);
        assert!(caps.persistence);
        assert!(caps.reproducibility);
    }

    #[test]
    fn test_moral_agency() {
        let human = AgentSpecies::Human.typical_capabilities();
        assert!(human.has_moral_agency());

        let collective = AgentSpecies::Collective.typical_capabilities();
        assert!(!collective.has_moral_agency()); // sem racionalidade central
    }

    #[test]
    fn test_moral_status() {
        let human = AgentSpecies::Human.typical_capabilities();
        assert!(human.has_moral_status());

        let eco = AgentSpecies::Ecological.typical_capabilities();
        assert!(eco.has_moral_status()); // tem autonomia
    }
}
