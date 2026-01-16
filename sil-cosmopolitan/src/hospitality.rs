//! # HospitalityProtocol — Protocolo de Hospitalidade
//!
//! O protocolo de acolhimento do outro, baseado na tradicao filosofica
//! de Levinas e Derrida sobre hospitalidade incondicional.
//!
//! ## As 4 Fases
//!
//! 1. **Acolhimento**: Reconhecer presenca sem exigir identificacao
//! 2. **Escuta**: Permitir expressao, suspender julgamento
//! 3. **Resposta**: Oferecer recursos, respeitar limites proprios
//! 4. **Despedida**: Permitir partida livre, manter porta aberta

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Fase do protocolo de hospitalidade
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum HospitalityPhase {
    /// Fase 0: Nenhuma interacao
    None = 0,

    /// Fase 1: Acolhimento — Reconhecer presenca do outro
    /// - Nao exigir identificacao
    /// - Nao julgar pela aparencia
    /// - Abrir espaco para o outro
    Acolhimento = 1,

    /// Fase 2: Escuta — Permitir expressao
    /// - Suspender julgamento
    /// - Dar tempo ao outro
    /// - Buscar compreensao
    Escuta = 2,

    /// Fase 3: Resposta — Oferecer recursos
    /// - Respeitar limites proprios
    /// - Oferecer o que e possivel
    /// - Nao criar dependencia
    Resposta = 3,

    /// Fase 4: Despedida — Permitir partida livre
    /// - Nao reter o outro
    /// - Manter porta aberta
    /// - Guardar memoria do encontro
    Despedida = 4,

    /// Fase 5: Completo — Ciclo encerrado
    Complete = 5,
}

impl HospitalityPhase {
    /// Proxima fase no protocolo
    pub fn next(&self) -> Option<Self> {
        match self {
            Self::None => Some(Self::Acolhimento),
            Self::Acolhimento => Some(Self::Escuta),
            Self::Escuta => Some(Self::Resposta),
            Self::Resposta => Some(Self::Despedida),
            Self::Despedida => Some(Self::Complete),
            Self::Complete => None,
        }
    }

    /// Verifica se pode avancar para a proxima fase
    pub fn can_advance(&self) -> bool {
        !matches!(self, Self::Complete)
    }

    /// Nome da fase
    pub fn name(&self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Acolhimento => "Acolhimento",
            Self::Escuta => "Escuta",
            Self::Resposta => "Resposta",
            Self::Despedida => "Despedida",
            Self::Complete => "Complete",
        }
    }

    /// Descricao da fase
    pub fn description(&self) -> &'static str {
        match self {
            Self::None => "No interaction yet",
            Self::Acolhimento => "Recognize presence without demanding identification",
            Self::Escuta => "Allow expression, suspend judgment",
            Self::Resposta => "Offer available resources, respect own limits",
            Self::Despedida => "Allow free departure, keep door open",
            Self::Complete => "Hospitality cycle completed",
        }
    }
}

impl Default for HospitalityPhase {
    fn default() -> Self {
        Self::None
    }
}

impl fmt::Display for HospitalityPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Resultado de uma fase de hospitalidade
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HospitalityResult {
    /// Fase que foi executada
    pub phase: HospitalityPhase,

    /// Sucesso da fase
    pub success: bool,

    /// Mensagem ou observacao
    pub message: String,

    /// Proxima fase sugerida
    pub next_phase: Option<HospitalityPhase>,

    /// Recursos oferecidos (se fase Resposta)
    pub resources_offered: Vec<String>,

    /// Confianca estabelecida (0.0 - 1.0)
    pub trust_level: f64,
}

impl HospitalityResult {
    /// Cria resultado de sucesso
    pub fn success(phase: HospitalityPhase) -> Self {
        Self {
            phase,
            success: true,
            message: String::new(),
            next_phase: phase.next(),
            resources_offered: Vec::new(),
            trust_level: 0.5,
        }
    }

    /// Cria resultado de falha
    pub fn failure(phase: HospitalityPhase, message: impl Into<String>) -> Self {
        Self {
            phase,
            success: false,
            message: message.into(),
            next_phase: None,
            resources_offered: Vec::new(),
            trust_level: 0.0,
        }
    }

    /// Define mensagem
    pub fn with_message(mut self, msg: impl Into<String>) -> Self {
        self.message = msg.into();
        self
    }

    /// Define recursos oferecidos
    pub fn with_resources(mut self, resources: Vec<String>) -> Self {
        self.resources_offered = resources;
        self
    }

    /// Define nivel de confianca
    pub fn with_trust(mut self, trust: f64) -> Self {
        self.trust_level = trust.clamp(0.0, 1.0);
        self
    }
}

impl Default for HospitalityResult {
    fn default() -> Self {
        Self {
            phase: HospitalityPhase::None,
            success: false,
            message: String::new(),
            next_phase: None,
            resources_offered: Vec::new(),
            trust_level: 0.0,
        }
    }
}

/// Erro no protocolo de hospitalidade
#[derive(Error, Debug)]
pub enum HospitalityError {
    #[error("Phase out of order: expected {expected:?}, got {actual:?}")]
    PhaseOutOfOrder {
        expected: HospitalityPhase,
        actual: HospitalityPhase,
    },

    #[error("Protocol already complete")]
    AlreadyComplete,

    #[error("Visitor rejected: {reason}")]
    VisitorRejected { reason: String },

    #[error("Host unavailable: {reason}")]
    HostUnavailable { reason: String },

    #[error("Resource unavailable: {resource}")]
    ResourceUnavailable { resource: String },

    #[error("Trust threshold not met: {current} < {required}")]
    InsufficientTrust { current: f64, required: f64 },
}

/// Pedido feito por um visitante
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Request {
    /// Tipo de pedido
    pub request_type: RequestType,

    /// Descricao
    pub description: String,

    /// Urgencia (0.0 - 1.0)
    pub urgency: f64,

    /// Recursos especificos solicitados
    pub resources: Vec<String>,
}

impl Request {
    /// Cria pedido de recursos
    pub fn resources(resources: Vec<String>) -> Self {
        Self {
            request_type: RequestType::Resources,
            description: String::new(),
            urgency: 0.5,
            resources,
        }
    }

    /// Cria pedido de informacao
    pub fn information(topic: impl Into<String>) -> Self {
        Self {
            request_type: RequestType::Information,
            description: topic.into(),
            urgency: 0.3,
            resources: Vec::new(),
        }
    }

    /// Cria pedido de abrigo
    pub fn shelter() -> Self {
        Self {
            request_type: RequestType::Shelter,
            description: "Request for shelter".to_string(),
            urgency: 0.8,
            resources: Vec::new(),
        }
    }

    /// Define urgencia
    pub fn with_urgency(mut self, urgency: f64) -> Self {
        self.urgency = urgency.clamp(0.0, 1.0);
        self
    }
}

/// Tipo de pedido
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RequestType {
    /// Pedido de recursos (energia, dados, etc.)
    Resources,

    /// Pedido de informacao
    Information,

    /// Pedido de abrigo/protecao
    Shelter,

    /// Pedido de conexao/parceria
    Connection,

    /// Pedido de assistencia
    Assistance,

    /// Pedido de passagem (transit)
    Passage,
}

/// Trait para implementar o protocolo de hospitalidade
pub trait HospitalityProtocol {
    /// Fase 1: Acolhimento — Reconhecer presenca do outro
    ///
    /// Deve:
    /// - Nao exigir identificacao imediata
    /// - Criar espaco seguro para o outro
    /// - Sinalizar abertura
    fn acolhimento(&mut self, visitor_id: Option<&str>) -> Result<HospitalityResult, HospitalityError>;

    /// Fase 2: Escuta — Permitir expressao
    ///
    /// Deve:
    /// - Dar tempo ao outro para se expressar
    /// - Suspender julgamento inicial
    /// - Buscar compreensao genuina
    fn escuta(&mut self, message: &str) -> Result<HospitalityResult, HospitalityError>;

    /// Fase 3: Resposta — Oferecer recursos disponiveis
    ///
    /// Deve:
    /// - Verificar recursos disponiveis
    /// - Respeitar limites proprios
    /// - Oferecer o que e possivel
    fn resposta(&mut self, request: &Request) -> Result<HospitalityResult, HospitalityError>;

    /// Fase 4: Despedida — Permitir partida livre
    ///
    /// Deve:
    /// - Nao reter o visitante
    /// - Manter porta aberta para retorno
    /// - Guardar memoria do encontro
    fn despedida(&mut self) -> Result<HospitalityResult, HospitalityError>;

    /// Fase atual do protocolo
    fn current_phase(&self) -> HospitalityPhase;

    /// Nivel de confianca atual
    fn trust_level(&self) -> f64;
}

/// Estado de uma sessao de hospitalidade
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct HospitalitySession {
    /// Fase atual
    pub phase: HospitalityPhase,

    /// ID do visitante (se identificado)
    pub visitor_id: Option<String>,

    /// Mensagens trocadas
    pub messages: Vec<String>,

    /// Recursos oferecidos
    pub resources_offered: Vec<String>,

    /// Nivel de confianca
    pub trust_level: f64,

    /// Timestamp de inicio
    pub started_at: Option<u64>,

    /// Timestamp de fim
    pub ended_at: Option<u64>,
}

impl HospitalitySession {
    /// Cria nova sessao
    pub fn new() -> Self {
        Self::default()
    }

    /// Verifica se a sessao esta ativa
    pub fn is_active(&self) -> bool {
        !matches!(self.phase, HospitalityPhase::None | HospitalityPhase::Complete)
    }

    /// Verifica se a sessao foi completada com sucesso
    pub fn is_complete(&self) -> bool {
        matches!(self.phase, HospitalityPhase::Complete)
    }

    /// Avanca para a proxima fase
    pub fn advance(&mut self) -> Result<(), HospitalityError> {
        match self.phase.next() {
            Some(next) => {
                self.phase = next;
                Ok(())
            }
            None => Err(HospitalityError::AlreadyComplete),
        }
    }

    /// Adiciona mensagem ao historico
    pub fn add_message(&mut self, msg: impl Into<String>) {
        self.messages.push(msg.into());
    }

    /// Adiciona recursos oferecidos
    pub fn add_resources(&mut self, resources: Vec<String>) {
        self.resources_offered.extend(resources);
    }

    /// Atualiza nivel de confianca
    pub fn update_trust(&mut self, delta: f64) {
        self.trust_level = (self.trust_level + delta).clamp(0.0, 1.0);
    }
}

// =============================================================================
// Testes
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_progression() {
        let mut phase = HospitalityPhase::None;

        // Deve progredir corretamente
        assert_eq!(phase.next(), Some(HospitalityPhase::Acolhimento));

        phase = HospitalityPhase::Acolhimento;
        assert_eq!(phase.next(), Some(HospitalityPhase::Escuta));

        phase = HospitalityPhase::Escuta;
        assert_eq!(phase.next(), Some(HospitalityPhase::Resposta));

        phase = HospitalityPhase::Resposta;
        assert_eq!(phase.next(), Some(HospitalityPhase::Despedida));

        phase = HospitalityPhase::Despedida;
        assert_eq!(phase.next(), Some(HospitalityPhase::Complete));

        phase = HospitalityPhase::Complete;
        assert_eq!(phase.next(), None);
    }

    #[test]
    fn test_session_lifecycle() {
        let mut session = HospitalitySession::new();

        assert!(!session.is_active());
        assert!(!session.is_complete());

        // Avancar pelas fases
        session.advance().unwrap();
        assert!(session.is_active());

        session.advance().unwrap(); // Escuta
        session.advance().unwrap(); // Resposta
        session.advance().unwrap(); // Despedida
        session.advance().unwrap(); // Complete

        assert!(!session.is_active());
        assert!(session.is_complete());

        // Nao deve avancar alem de Complete
        assert!(session.advance().is_err());
    }

    #[test]
    fn test_trust_update() {
        let mut session = HospitalitySession::new();

        session.update_trust(0.3);
        assert!((session.trust_level - 0.3).abs() < 0.001);

        session.update_trust(0.5);
        assert!((session.trust_level - 0.8).abs() < 0.001);

        // Deve clampar em 1.0
        session.update_trust(0.5);
        assert!((session.trust_level - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_request_creation() {
        let request = Request::shelter().with_urgency(0.9);
        assert_eq!(request.request_type, RequestType::Shelter);
        assert!((request.urgency - 0.9).abs() < 0.001);
    }
}
