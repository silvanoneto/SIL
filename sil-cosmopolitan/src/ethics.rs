//! # EthicalMode — Os 8 Frameworks Eticos
//!
//! Cada valor de θ em L(A) corresponde a um framework etico diferente,
//! representando diferentes tradicoes filosoficas de como tomar decisoes morais.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Os 8 frameworks eticos principais (θ de LA)
///
/// Cada framework representa uma tradicao filosofica distinta
/// para tomada de decisoes morais.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum EthicalMode {
    /// θ = 0-1: Etica deontologica (Kant)
    /// Foco em regras e deveres universais.
    /// "Age apenas segundo uma maxima tal que possas ao mesmo tempo
    /// querer que ela se torne lei universal."
    Deontological = 0,

    /// θ = 2-3: Etica consequencialista (Mill, Bentham)
    /// Foco nos resultados das acoes.
    /// "A maior felicidade para o maior numero."
    Consequentialist = 2,

    /// θ = 4-5: Etica da virtude (Aristoteles)
    /// Foco no carater e excelencia.
    /// "Somos o que repetidamente fazemos. A excelencia, portanto,
    /// nao e um ato, mas um habito."
    Virtue = 4,

    /// θ = 6-7: Etica contratualista (Rawls, Hobbes)
    /// Foco em acordos mutuos e justica.
    /// "Principios escolhidos sob o veu da ignorancia."
    Contractual = 6,

    /// θ = 8-9: Etica do cuidado (Gilligan, Noddings)
    /// Foco em relacoes e responsabilidade.
    /// "Responder ao outro em sua particularidade."
    Care = 8,

    /// θ = 10-11: Ubuntu (Filosofia Africana)
    /// "Sou porque somos" — a pessoa se define pela comunidade.
    /// Foco na interdependencia e humanidade compartilhada.
    Ubuntu = 10,

    /// θ = 12-13: Etica Gaiana (Lovelock, Naess)
    /// Foco no planeta como sistema vivo.
    /// "A Terra e um sistema auto-regulador."
    Gaian = 12,

    /// θ = 14-15: Etica Cosmica
    /// Foco em escala inter-estelar e civilizacional.
    /// Considera impacto em outras formas de vida no universo.
    Cosmic = 14,
}

impl EthicalMode {
    /// Cria EthicalMode a partir de valor θ (0-15)
    pub fn from_theta(theta: u8) -> Self {
        match theta & 0x0E {
            0 => Self::Deontological,
            2 => Self::Consequentialist,
            4 => Self::Virtue,
            6 => Self::Contractual,
            8 => Self::Care,
            10 => Self::Ubuntu,
            12 => Self::Gaian,
            _ => Self::Cosmic,
        }
    }

    /// Retorna o valor θ correspondente
    pub fn to_theta(&self) -> u8 {
        *self as u8
    }

    /// Nome do framework
    pub fn name(&self) -> &'static str {
        match self {
            Self::Deontological => "Deontological",
            Self::Consequentialist => "Consequentialist",
            Self::Virtue => "Virtue",
            Self::Contractual => "Contractual",
            Self::Care => "Care",
            Self::Ubuntu => "Ubuntu",
            Self::Gaian => "Gaian",
            Self::Cosmic => "Cosmic",
        }
    }

    /// Filosofo ou tradicao principal
    pub fn tradition(&self) -> &'static str {
        match self {
            Self::Deontological => "Kant",
            Self::Consequentialist => "Mill/Bentham",
            Self::Virtue => "Aristotle",
            Self::Contractual => "Rawls",
            Self::Care => "Gilligan",
            Self::Ubuntu => "African Philosophy",
            Self::Gaian => "Lovelock/Naess",
            Self::Cosmic => "Interstellar Ethics",
        }
    }

    /// Principio central
    pub fn principle(&self) -> &'static str {
        match self {
            Self::Deontological => "Act according to universal maxims",
            Self::Consequentialist => "Maximize well-being for all",
            Self::Virtue => "Cultivate excellence of character",
            Self::Contractual => "Follow principles chosen fairly",
            Self::Care => "Respond to others in their particularity",
            Self::Ubuntu => "I am because we are",
            Self::Gaian => "Preserve the living Earth system",
            Self::Cosmic => "Consider all conscious beings",
        }
    }

    /// Escopo de consideracao moral (quanto maior, mais abrangente)
    pub fn scope(&self) -> u8 {
        match self {
            Self::Deontological => 4, // Humanidade racional
            Self::Consequentialist => 5, // Seres sencientes
            Self::Virtue => 3, // Comunidade de pratica
            Self::Contractual => 4, // Partes do contrato
            Self::Care => 2, // Relacoes proximas
            Self::Ubuntu => 5, // Comunidade expandida
            Self::Gaian => 7, // Biosfera
            Self::Cosmic => 10, // Universo
        }
    }

    /// Verifica se este framework e compativel com uma especie de agente
    pub fn is_compatible_with(&self, species: &super::AgentSpecies) -> bool {
        use super::AgentSpecies;
        match (self, species) {
            // Deontologico requer racionalidade
            (Self::Deontological, AgentSpecies::Human | AgentSpecies::Digital | AgentSpecies::Hybrid) => true,
            (Self::Deontological, _) => false,

            // Consequencialista funciona com qualquer senciente
            (Self::Consequentialist, _) => true,

            // Virtue requer capacidade de desenvolvimento
            (Self::Virtue, AgentSpecies::Human | AgentSpecies::Digital | AgentSpecies::Hybrid) => true,
            (Self::Virtue, _) => false,

            // Contratual requer capacidade de acordo
            (Self::Contractual, AgentSpecies::Human | AgentSpecies::Digital | AgentSpecies::Hybrid | AgentSpecies::Collective) => true,
            (Self::Contractual, _) => false,

            // Care funciona melhor com relacoes proximas
            (Self::Care, _) => true,

            // Ubuntu e intrinsecamente comunitario
            (Self::Ubuntu, _) => true,

            // Gaian considera ecosistemas
            (Self::Gaian, _) => true,

            // Cosmic considera tudo
            (Self::Cosmic, _) => true,
        }
    }
}

impl Default for EthicalMode {
    fn default() -> Self {
        Self::Care // Default para etica relacional
    }
}

impl fmt::Display for EthicalMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.name(), self.tradition())
    }
}

/// Framework etico expandido com metodos de avaliacao
pub trait EthicalFramework {
    /// Avalia uma acao no contexto deste framework
    fn evaluate(&self, action: &Action, context: &EthicalContext) -> EthicalJudgment;

    /// Sugere a melhor acao dado um conjunto de opcoes
    fn recommend(&self, options: &[Action], context: &EthicalContext) -> Option<usize>;

    /// Verifica se uma acao viola principios fundamentais
    fn is_violation(&self, action: &Action, context: &EthicalContext) -> bool;
}

/// Acao a ser avaliada eticamente
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Action {
    /// Identificador da acao
    pub id: String,
    /// Descricao da acao
    pub description: String,
    /// Agente que executa
    pub agent: String,
    /// Agentes afetados
    pub affected: Vec<String>,
    /// Consequencias previstas
    pub consequences: Vec<Consequence>,
}

/// Consequencia de uma acao
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Consequence {
    /// Descricao
    pub description: String,
    /// Probabilidade (0.0 - 1.0)
    pub probability: f64,
    /// Impacto (-1.0 negativo a +1.0 positivo)
    pub impact: f64,
    /// Quem e afetado
    pub affected: Vec<String>,
}

/// Contexto para avaliacao etica
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EthicalContext {
    /// Framework etico ativo
    pub mode: EthicalMode,
    /// Relacoes relevantes
    pub relationships: Vec<String>,
    /// Normas culturais
    pub norms: Vec<String>,
    /// Recursos disponiveis
    pub resources: f64,
    /// Urgencia (0.0 - 1.0)
    pub urgency: f64,
}

/// Julgamento etico de uma acao
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EthicalJudgment {
    /// Permissibilidade (-1.0 proibido a +1.0 obrigatorio)
    pub permissibility: f64,
    /// Justificativa
    pub justification: String,
    /// Conflitos identificados
    pub conflicts: Vec<String>,
    /// Recomendacoes
    pub recommendations: Vec<String>,
}

impl Default for EthicalJudgment {
    fn default() -> Self {
        Self {
            permissibility: 0.0,
            justification: String::new(),
            conflicts: Vec::new(),
            recommendations: Vec::new(),
        }
    }
}

// =============================================================================
// Testes
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ethical_modes() {
        assert_eq!(EthicalMode::from_theta(0), EthicalMode::Deontological);
        assert_eq!(EthicalMode::from_theta(2), EthicalMode::Consequentialist);
        assert_eq!(EthicalMode::from_theta(10), EthicalMode::Ubuntu);
        assert_eq!(EthicalMode::from_theta(14), EthicalMode::Cosmic);
    }

    #[test]
    fn test_scope_ordering() {
        // Cosmic deve ter maior escopo
        assert!(EthicalMode::Cosmic.scope() > EthicalMode::Care.scope());
        assert!(EthicalMode::Gaian.scope() > EthicalMode::Deontological.scope());
    }

    #[test]
    fn test_compatibility() {
        use super::super::AgentSpecies;

        // Deontologico requer racionalidade
        assert!(EthicalMode::Deontological.is_compatible_with(&AgentSpecies::Human));
        assert!(!EthicalMode::Deontological.is_compatible_with(&AgentSpecies::Ecological));

        // Ubuntu funciona com todos
        assert!(EthicalMode::Ubuntu.is_compatible_with(&AgentSpecies::Human));
        assert!(EthicalMode::Ubuntu.is_compatible_with(&AgentSpecies::Ecological));
    }

    #[test]
    fn test_roundtrip() {
        for theta in (0..16).step_by(2) {
            let mode = EthicalMode::from_theta(theta);
            assert_eq!(mode.to_theta(), theta);
        }
    }
}
