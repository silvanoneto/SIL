//! # Territory — Territorios e Fronteiras
//!
//! Estruturas para representar territorios digitais sob jurisdicao de um no.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Identificador unico de territorio
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TerritoryId(pub u64);

impl fmt::Display for TerritoryId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "T-{:016x}", self.0)
    }
}

/// Territorio sob jurisdicao de um no
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Territory {
    /// Identificador unico
    pub id: TerritoryId,

    /// Nome do territorio
    pub name: String,

    /// Descricao
    pub description: String,

    /// Fronteiras com outros territorios
    pub borders: Vec<Border>,

    /// Recursos do territorio
    pub resources: Vec<Resource>,

    /// Populacao (numero de agentes)
    pub population: u64,

    /// Hash de integridade do estado
    pub state_hash: [u8; 32],
}

impl Territory {
    /// Cria novo territorio
    pub fn new(id: TerritoryId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            description: String::new(),
            borders: Vec::new(),
            resources: Vec::new(),
            population: 0,
            state_hash: [0u8; 32],
        }
    }

    /// Define descricao
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Define populacao
    pub fn with_population(mut self, pop: u64) -> Self {
        self.population = pop;
        self
    }

    /// Adiciona fronteira
    pub fn add_border(&mut self, border: Border) {
        self.borders.push(border);
    }

    /// Adiciona recurso
    pub fn add_resource(&mut self, resource: Resource) {
        self.resources.push(resource);
    }

    /// Verifica se ha fronteira com outro territorio
    pub fn borders_with(&self, other: &TerritoryId) -> bool {
        self.borders.iter().any(|b| &b.neighbor == other)
    }

    /// Obtem fronteira especifica
    pub fn get_border(&self, neighbor: &TerritoryId) -> Option<&Border> {
        self.borders.iter().find(|b| &b.neighbor == neighbor)
    }

    /// Calcula permeabilidade media das fronteiras
    pub fn average_permeability(&self) -> f64 {
        if self.borders.is_empty() {
            return 1.0;
        }
        let sum: f64 = self.borders.iter().map(|b| b.permeability).sum();
        sum / self.borders.len() as f64
    }

    /// Total de recursos
    pub fn total_resources(&self) -> f64 {
        self.resources.iter().map(|r| r.amount).sum()
    }
}

impl Default for Territory {
    fn default() -> Self {
        Self::new(TerritoryId(0), "default")
    }
}

/// Fronteira entre territorios
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Border {
    /// Territorio vizinho
    pub neighbor: TerritoryId,

    /// Permeabilidade (0.0 = fechada, 1.0 = aberta)
    pub permeability: f64,

    /// Protocolo de fronteira
    pub protocol: BorderProtocol,

    /// Tipo de fronteira
    pub border_type: BorderType,
}

impl Border {
    /// Cria fronteira com vizinho
    pub fn new(neighbor: TerritoryId) -> Self {
        Self {
            neighbor,
            permeability: 0.5,
            protocol: BorderProtocol::Standard,
            border_type: BorderType::Digital,
        }
    }

    /// Define permeabilidade
    pub fn with_permeability(mut self, perm: f64) -> Self {
        self.permeability = perm.clamp(0.0, 1.0);
        self
    }

    /// Define protocolo
    pub fn with_protocol(mut self, protocol: BorderProtocol) -> Self {
        self.protocol = protocol;
        self
    }

    /// Verifica se fronteira esta aberta
    pub fn is_open(&self) -> bool {
        self.permeability > 0.5
    }

    /// Verifica se fronteira esta fechada
    pub fn is_closed(&self) -> bool {
        self.permeability < 0.1
    }
}

/// Protocolo de fronteira
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BorderProtocol {
    /// Protocolo padrao (verificacao basica)
    Standard,

    /// Fronteira aberta (livre transito)
    Open,

    /// Fronteira controlada (autorizacao necessaria)
    Controlled,

    /// Fronteira restrita (lista de permitidos)
    Restricted,

    /// Fronteira fechada (sem transito)
    Closed,

    /// Protocolo de quarentena
    Quarantine,

    /// Protocolo de emergencia
    Emergency,

    /// Protocolo customizado
    Custom,
}

/// Tipo de fronteira
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BorderType {
    /// Fronteira digital (dados, mensagens)
    Digital,

    /// Fronteira fisica (hardware, localizacao)
    Physical,

    /// Fronteira semantica (significado, contexto)
    Semantic,

    /// Fronteira de codigo (execucao, permissoes)
    Code,
}

/// Recurso de um territorio
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Resource {
    /// Tipo de recurso
    pub resource_type: ResourceType,

    /// Nome
    pub name: String,

    /// Quantidade
    pub amount: f64,

    /// Renovavel?
    pub renewable: bool,
}

impl Resource {
    /// Cria novo recurso
    pub fn new(resource_type: ResourceType, name: impl Into<String>, amount: f64) -> Self {
        Self {
            resource_type,
            name: name.into(),
            amount,
            renewable: false,
        }
    }

    /// Define como renovavel
    pub fn renewable(mut self) -> Self {
        self.renewable = true;
        self
    }
}

/// Tipo de recurso
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    /// Capacidade computacional
    Compute,

    /// Armazenamento
    Storage,

    /// Largura de banda
    Bandwidth,

    /// Energia
    Energy,

    /// Dados
    Data,

    /// Reputacao
    Reputation,

    /// Tokens/moeda
    Token,
}

// =============================================================================
// Testes
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_territory_creation() {
        let territory = Territory::new(TerritoryId(1), "TestLand")
            .with_description("A test territory")
            .with_population(100);

        assert_eq!(territory.id, TerritoryId(1));
        assert_eq!(territory.name, "TestLand");
        assert_eq!(territory.population, 100);
    }

    #[test]
    fn test_border_creation() {
        let border = Border::new(TerritoryId(2))
            .with_permeability(0.8)
            .with_protocol(BorderProtocol::Open);

        assert!(border.is_open());
        assert!(!border.is_closed());
    }

    #[test]
    fn test_territory_with_borders() {
        let mut territory = Territory::new(TerritoryId(1), "Main");

        territory.add_border(Border::new(TerritoryId(2)).with_permeability(0.7));
        territory.add_border(Border::new(TerritoryId(3)).with_permeability(0.3));

        assert!(territory.borders_with(&TerritoryId(2)));
        assert!(!territory.borders_with(&TerritoryId(4)));

        let avg = territory.average_permeability();
        assert!((avg - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_resource_creation() {
        let resource = Resource::new(ResourceType::Compute, "CPU", 100.0)
            .renewable();

        assert!(resource.renewable);
        assert_eq!(resource.amount, 100.0);
    }
}
