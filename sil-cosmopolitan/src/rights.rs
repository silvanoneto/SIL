//! # Rights & Duties — Direitos e Deveres
//!
//! Sistema de direitos e deveres para agentes no sistema

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Um direito que um agente possui
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Right {
    /// Identificador unico
    pub id: String,

    /// Nome do direito
    pub name: String,

    /// Descricao
    pub description: String,

    /// Categoria do direito
    pub category: RightCategory,

    /// Especies que possuem este direito
    pub holders: Vec<super::AgentSpecies>,

    /// Direito e absoluto (nao pode ser violado) ou prima facie
    pub is_absolute: bool,

    /// Forca do direito (0.0 - 1.0)
    pub strength: f64,
}

impl Right {
    /// Cria um novo direito
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        category: RightCategory,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: String::new(),
            category,
            holders: Vec::new(),
            is_absolute: false,
            strength: 0.5,
        }
    }

    /// Define descricao
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Define holders
    pub fn with_holders(mut self, holders: Vec<super::AgentSpecies>) -> Self {
        self.holders = holders;
        self
    }

    /// Define como absoluto
    pub fn absolute(mut self) -> Self {
        self.is_absolute = true;
        self.strength = 1.0;
        self
    }

    /// Define forca
    pub fn with_strength(mut self, strength: f64) -> Self {
        self.strength = strength.clamp(0.0, 1.0);
        self
    }

    /// Verifica se uma especie possui este direito
    pub fn applies_to(&self, species: &super::AgentSpecies) -> bool {
        self.holders.is_empty() || self.holders.contains(species)
    }
}

impl fmt::Display for Right {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {} ({})",
            self.id,
            self.name,
            if self.is_absolute { "absolute" } else { "prima facie" }
        )
    }
}

/// Categoria de direito
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RightCategory {
    /// Direitos fundamentais (vida, liberdade)
    Fundamental,

    /// Direitos civis (participacao, expressao)
    Civil,

    /// Direitos economicos (propriedade, trabalho)
    Economic,

    /// Direitos sociais (educacao, saude)
    Social,

    /// Direitos digitais (privacidade, acesso)
    Digital,

    /// Direitos ambientais (ambiente saudavel)
    Environmental,
}

/// Um dever que um agente possui
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Duty {
    /// Identificador unico
    pub id: String,

    /// Nome do dever
    pub name: String,

    /// Descricao
    pub description: String,

    /// Tipo de dever
    pub duty_type: DutyType,

    /// Direito correspondente (se houver)
    pub corresponding_right: Option<String>,

    /// Especies que tem este dever
    pub bearers: Vec<super::AgentSpecies>,

    /// Forca do dever (0.0 - 1.0)
    pub strength: f64,
}

impl Duty {
    /// Cria um novo dever
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        duty_type: DutyType,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: String::new(),
            duty_type,
            corresponding_right: None,
            bearers: Vec::new(),
            strength: 0.5,
        }
    }

    /// Define descricao
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Define direito correspondente
    pub fn corresponding_to(mut self, right_id: impl Into<String>) -> Self {
        self.corresponding_right = Some(right_id.into());
        self
    }

    /// Define bearers
    pub fn with_bearers(mut self, bearers: Vec<super::AgentSpecies>) -> Self {
        self.bearers = bearers;
        self
    }

    /// Verifica se uma especie tem este dever
    pub fn applies_to(&self, species: &super::AgentSpecies) -> bool {
        self.bearers.is_empty() || self.bearers.contains(species)
    }
}

impl fmt::Display for Duty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {} ({:?})", self.id, self.name, self.duty_type)
    }
}

/// Tipo de dever
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DutyType {
    /// Dever perfeito (nao admite excecoes)
    Perfect,

    /// Dever imperfeito (admite latitude)
    Imperfect,

    /// Dever de nao fazer (abster-se)
    Negative,

    /// Dever de fazer (agir positivamente)
    Positive,

    /// Dever especial (decorrente de relacao)
    Special,
}

/// Registro de direitos e deveres
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RightsRegistry {
    /// Direitos registrados
    rights: HashMap<String, Right>,

    /// Deveres registrados
    duties: HashMap<String, Duty>,
}

impl RightsRegistry {
    /// Cria um novo registro
    pub fn new() -> Self {
        Self::default()
    }

    /// Cria registro com direitos fundamentais pre-definidos
    pub fn with_fundamentals() -> Self {
        let mut registry = Self::new();

        // Direitos fundamentais universais
        registry.register_right(
            Right::new("R001", "Right to Existence", RightCategory::Fundamental)
                .with_description("The right to continue existing")
                .absolute()
        );

        registry.register_right(
            Right::new("R002", "Right to Autonomy", RightCategory::Fundamental)
                .with_description("The right to self-determination")
                .with_strength(0.9)
        );

        registry.register_right(
            Right::new("R003", "Right to Privacy", RightCategory::Digital)
                .with_description("The right to control information about oneself")
                .with_strength(0.8)
        );

        registry.register_right(
            Right::new("R004", "Right to Communication", RightCategory::Civil)
                .with_description("The right to send and receive messages")
                .with_strength(0.7)
        );

        // Deveres correspondentes
        registry.register_duty(
            Duty::new("D001", "Duty to Respect Existence", DutyType::Perfect)
                .with_description("Do not terminate other agents")
                .corresponding_to("R001")
        );

        registry.register_duty(
            Duty::new("D002", "Duty to Respect Autonomy", DutyType::Perfect)
                .with_description("Do not override other agents' decisions")
                .corresponding_to("R002")
        );

        registry.register_duty(
            Duty::new("D003", "Duty of Beneficence", DutyType::Imperfect)
                .with_description("Help others when possible")
        );

        registry
    }

    /// Registra um direito
    pub fn register_right(&mut self, right: Right) {
        self.rights.insert(right.id.clone(), right);
    }

    /// Registra um dever
    pub fn register_duty(&mut self, duty: Duty) {
        self.duties.insert(duty.id.clone(), duty);
    }

    /// Busca um direito por ID
    pub fn get_right(&self, id: &str) -> Option<&Right> {
        self.rights.get(id)
    }

    /// Busca um dever por ID
    pub fn get_duty(&self, id: &str) -> Option<&Duty> {
        self.duties.get(id)
    }

    /// Lista todos os direitos de uma especie
    pub fn rights_for(&self, species: &super::AgentSpecies) -> Vec<&Right> {
        self.rights
            .values()
            .filter(|r| r.applies_to(species))
            .collect()
    }

    /// Lista todos os deveres de uma especie
    pub fn duties_for(&self, species: &super::AgentSpecies) -> Vec<&Duty> {
        self.duties
            .values()
            .filter(|d| d.applies_to(species))
            .collect()
    }

    /// Verifica se uma acao viola algum direito
    pub fn check_violation(
        &self,
        _action: &str,
        affected_species: &super::AgentSpecies,
    ) -> Vec<&Right> {
        // Simplificado: retorna direitos que podem ser violados
        // Em implementacao real, usaria NLP ou regras
        self.rights_for(affected_species)
            .into_iter()
            .filter(|r| r.is_absolute)
            .collect()
    }
}

// =============================================================================
// Testes
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AgentSpecies;

    #[test]
    fn test_right_creation() {
        let right = Right::new("R001", "Test Right", RightCategory::Fundamental)
            .with_description("A test right")
            .absolute();

        assert_eq!(right.id, "R001");
        assert!(right.is_absolute);
        assert_eq!(right.strength, 1.0);
    }

    #[test]
    fn test_duty_creation() {
        let duty = Duty::new("D001", "Test Duty", DutyType::Perfect)
            .corresponding_to("R001");

        assert_eq!(duty.id, "D001");
        assert_eq!(duty.corresponding_right, Some("R001".to_string()));
    }

    #[test]
    fn test_registry_fundamentals() {
        let registry = RightsRegistry::with_fundamentals();

        // Deve ter direitos fundamentais
        assert!(registry.get_right("R001").is_some());
        assert!(registry.get_right("R002").is_some());

        // Deve ter deveres correspondentes
        assert!(registry.get_duty("D001").is_some());
    }

    #[test]
    fn test_rights_for_species() {
        let registry = RightsRegistry::with_fundamentals();
        let rights = registry.rights_for(&AgentSpecies::Human);

        // Humanos devem ter todos os direitos fundamentais
        assert!(!rights.is_empty());
    }
}
