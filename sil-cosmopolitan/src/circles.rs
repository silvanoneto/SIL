//! # MoralCircle — Circulos de Consideracao Moral
//!
//! Baseado na ideia de Peter Singer de "circulo moral em expansao",
//! onde a consideracao moral historicamente se expande para incluir
//! mais seres.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Circulo de consideracao moral
///
/// Representa diferentes niveis de abrangencia na consideracao moral,
/// do mais restrito (Self) ao mais amplo (Cosmic).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum MoralCircle {
    /// Apenas o proprio agente
    Self_ = 0,

    /// Familiares e amigos proximos
    Kin = 1,

    /// Comunidade local
    Community = 2,

    /// Nacao ou grupo etnico
    Nation = 3,

    /// Toda a humanidade
    Humanity = 4,

    /// Todos os seres sencientes
    Sentient = 5,

    /// Todos os seres vivos
    Life = 6,

    /// Ecossistemas e Gaia
    Biosphere = 7,

    /// Todo o universo/cosmos
    Cosmic = 8,
}

impl MoralCircle {
    /// Cria circulo a partir do valor rho (magnitude etica)
    pub fn from_rho(rho: i8) -> Self {
        // rho vai de -8 a +7, mapeamos para 0-8
        let normalized = ((rho + 8) as f64 / 15.0 * 8.0).round() as u8;
        Self::from_index(normalized)
    }

    /// Cria circulo a partir de indice
    pub fn from_index(index: u8) -> Self {
        match index {
            0 => Self::Self_,
            1 => Self::Kin,
            2 => Self::Community,
            3 => Self::Nation,
            4 => Self::Humanity,
            5 => Self::Sentient,
            6 => Self::Life,
            7 => Self::Biosphere,
            _ => Self::Cosmic,
        }
    }

    /// Retorna o indice do circulo
    pub fn index(&self) -> u8 {
        *self as u8
    }

    /// Converte para valor rho
    pub fn to_rho(&self) -> i8 {
        // Mapear 0-8 para -8 a +7
        let index = *self as i8;
        (index * 2) - 8 + (index / 4)
    }

    /// Nome do circulo
    pub fn name(&self) -> &'static str {
        match self {
            Self::Self_ => "Self",
            Self::Kin => "Kin",
            Self::Community => "Community",
            Self::Nation => "Nation",
            Self::Humanity => "Humanity",
            Self::Sentient => "Sentient Beings",
            Self::Life => "All Life",
            Self::Biosphere => "Biosphere",
            Self::Cosmic => "Cosmic",
        }
    }

    /// Descricao do circulo
    pub fn description(&self) -> &'static str {
        match self {
            Self::Self_ => "Only the agent itself",
            Self::Kin => "Family and close friends",
            Self::Community => "Local community and neighbors",
            Self::Nation => "Nation, ethnicity, or large group",
            Self::Humanity => "All human beings",
            Self::Sentient => "All sentient beings capable of suffering",
            Self::Life => "All living organisms",
            Self::Biosphere => "Ecosystems and Earth as a whole",
            Self::Cosmic => "All existence in the universe",
        }
    }

    /// Exemplos de membros deste circulo
    pub fn examples(&self) -> &'static [&'static str] {
        match self {
            Self::Self_ => &["The individual agent"],
            Self::Kin => &["Parents", "Children", "Siblings", "Close friends"],
            Self::Community => &["Neighbors", "Colleagues", "Local organizations"],
            Self::Nation => &["Fellow citizens", "Cultural group members"],
            Self::Humanity => &["All humans", "Future generations"],
            Self::Sentient => &["Humans", "Animals", "AI (potentially)"],
            Self::Life => &["Plants", "Fungi", "Bacteria", "All organisms"],
            Self::Biosphere => &["Rainforests", "Oceans", "Gaia"],
            Self::Cosmic => &["Alien life", "Cosmic processes", "Existence itself"],
        }
    }

    /// Verifica se este circulo contem outro
    pub fn contains(&self, other: &MoralCircle) -> bool {
        *self >= *other
    }

    /// Proximo circulo mais amplo
    pub fn expand(&self) -> Option<Self> {
        match self {
            Self::Self_ => Some(Self::Kin),
            Self::Kin => Some(Self::Community),
            Self::Community => Some(Self::Nation),
            Self::Nation => Some(Self::Humanity),
            Self::Humanity => Some(Self::Sentient),
            Self::Sentient => Some(Self::Life),
            Self::Life => Some(Self::Biosphere),
            Self::Biosphere => Some(Self::Cosmic),
            Self::Cosmic => None,
        }
    }

    /// Proximo circulo mais restrito
    pub fn contract(&self) -> Option<Self> {
        match self {
            Self::Self_ => None,
            Self::Kin => Some(Self::Self_),
            Self::Community => Some(Self::Kin),
            Self::Nation => Some(Self::Community),
            Self::Humanity => Some(Self::Nation),
            Self::Sentient => Some(Self::Humanity),
            Self::Life => Some(Self::Sentient),
            Self::Biosphere => Some(Self::Life),
            Self::Cosmic => Some(Self::Biosphere),
        }
    }

    /// Circulo sugerido para um framework etico
    pub fn suggested_for(mode: &super::EthicalMode) -> Self {
        use super::EthicalMode;
        match mode {
            EthicalMode::Deontological => Self::Humanity,
            EthicalMode::Consequentialist => Self::Sentient,
            EthicalMode::Virtue => Self::Community,
            EthicalMode::Contractual => Self::Nation,
            EthicalMode::Care => Self::Kin,
            EthicalMode::Ubuntu => Self::Community,
            EthicalMode::Gaian => Self::Biosphere,
            EthicalMode::Cosmic => Self::Cosmic,
        }
    }
}

impl Default for MoralCircle {
    fn default() -> Self {
        Self::Humanity // Default etico comum
    }
}

impl fmt::Display for MoralCircle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Representa uma expansao ou contracao do circulo moral
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CircleExpansion {
    /// Circulo original
    pub from: MoralCircle,

    /// Circulo alvo
    pub to: MoralCircle,

    /// Razao para a mudanca
    pub reason: String,

    /// Nivel de confianca na mudanca (0.0 - 1.0)
    pub confidence: f64,

    /// Se a mudanca e permanente ou temporaria
    pub permanent: bool,
}

impl CircleExpansion {
    /// Cria expansao
    pub fn expand(from: MoralCircle, reason: impl Into<String>) -> Option<Self> {
        from.expand().map(|to| Self {
            from,
            to,
            reason: reason.into(),
            confidence: 0.5,
            permanent: false,
        })
    }

    /// Cria contracao
    pub fn contract(from: MoralCircle, reason: impl Into<String>) -> Option<Self> {
        from.contract().map(|to| Self {
            from,
            to,
            reason: reason.into(),
            confidence: 0.5,
            permanent: false,
        })
    }

    /// Cria mudanca direta
    pub fn direct(from: MoralCircle, to: MoralCircle, reason: impl Into<String>) -> Self {
        Self {
            from,
            to,
            reason: reason.into(),
            confidence: 0.5,
            permanent: false,
        }
    }

    /// Define confianca
    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Define como permanente
    pub fn permanent(mut self) -> Self {
        self.permanent = true;
        self
    }

    /// Verifica se e uma expansao (aumenta circulo)
    pub fn is_expansion(&self) -> bool {
        self.to > self.from
    }

    /// Verifica se e uma contracao (diminui circulo)
    pub fn is_contraction(&self) -> bool {
        self.to < self.from
    }

    /// Magnitude da mudanca (quantos niveis)
    pub fn magnitude(&self) -> i8 {
        (self.to as i8) - (self.from as i8)
    }
}

// =============================================================================
// Testes
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circle_ordering() {
        assert!(MoralCircle::Cosmic > MoralCircle::Self_);
        assert!(MoralCircle::Humanity > MoralCircle::Kin);
        assert!(MoralCircle::Sentient > MoralCircle::Humanity);
    }

    #[test]
    fn test_containment() {
        assert!(MoralCircle::Humanity.contains(&MoralCircle::Kin));
        assert!(MoralCircle::Cosmic.contains(&MoralCircle::Self_));
        assert!(!MoralCircle::Self_.contains(&MoralCircle::Kin));
    }

    #[test]
    fn test_expansion() {
        let mut circle = MoralCircle::Self_;

        // Deve expandir ate Cosmic
        while let Some(expanded) = circle.expand() {
            assert!(expanded > circle);
            circle = expanded;
        }

        assert_eq!(circle, MoralCircle::Cosmic);
    }

    #[test]
    fn test_contraction() {
        let mut circle = MoralCircle::Cosmic;

        // Deve contrair ate Self
        while let Some(contracted) = circle.contract() {
            assert!(contracted < circle);
            circle = contracted;
        }

        assert_eq!(circle, MoralCircle::Self_);
    }

    #[test]
    fn test_suggested_circles() {
        use super::super::EthicalMode;

        assert_eq!(
            MoralCircle::suggested_for(&EthicalMode::Gaian),
            MoralCircle::Biosphere
        );
        assert_eq!(
            MoralCircle::suggested_for(&EthicalMode::Care),
            MoralCircle::Kin
        );
    }

    #[test]
    fn test_circle_expansion() {
        let expansion = CircleExpansion::expand(
            MoralCircle::Humanity,
            "Recognized animal sentience"
        ).unwrap();

        assert!(expansion.is_expansion());
        assert_eq!(expansion.to, MoralCircle::Sentient);
        assert_eq!(expansion.magnitude(), 1);
    }
}
