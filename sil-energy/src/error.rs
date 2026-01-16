//! Erros do módulo de energia

use thiserror::Error;

/// Resultado de operações de energia
pub type EnergyResult<T> = Result<T, EnergyError>;

/// Erros de medição e modelagem de energia
#[derive(Debug, Error)]
pub enum EnergyError {
    /// Modelo de energia não suportado
    #[error("Modelo de energia não suportado: {0}")]
    UnsupportedModel(String),

    /// Medição não iniciada
    #[error("Medição não foi iniciada. Chame begin_measurement() primeiro")]
    MeasurementNotStarted,

    /// Medição já em progresso
    #[error("Medição já está em progresso")]
    MeasurementInProgress,

    /// Erro de calibração
    #[error("Erro de calibração: {0}")]
    CalibrationError(String),

    /// Orçamento de energia excedido
    #[error("Orçamento de energia excedido: {consumed:.6} J > {budget:.6} J")]
    BudgetExceeded { consumed: f64, budget: f64 },

    /// Potência limite excedida
    #[error("Potência limite excedida: {current:.2} W > {limit:.2} W")]
    PowerLimitExceeded { current: f64, limit: f64 },

    /// Erro de I/O
    #[error("Erro de I/O: {0}")]
    IoError(#[from] std::io::Error),

    /// Erro de sistema
    #[error("Erro de sistema: {0}")]
    SystemError(String),

    /// Recurso não disponível
    #[error("Recurso de energia não disponível: {0}")]
    ResourceUnavailable(String),

    /// Overflow numérico
    #[error("Overflow numérico ao calcular energia")]
    NumericOverflow,

    /// Configuração inválida
    #[error("Configuração de energia inválida: {0}")]
    InvalidConfig(String),
}
