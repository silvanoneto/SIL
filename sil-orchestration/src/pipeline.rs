//! Pipeline de execução do orquestrador

use sil_core::prelude::*;
use crate::error::{OrchestrationError, OrchestrationResult};

/// Estágio do pipeline
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub enum PipelineStage {
    /// Fase de sensoriamento (L0-L4)
    Sense = 0,
    /// Fase de processamento (L5-L7)
    Process = 1,
    /// Fase de atuação (L6)
    Actuate = 2,
    /// Fase de rede (L8)
    Network = 3,
    /// Fase de governança (L9-LA)
    Govern = 4,
    /// Fase de enxame (LB)
    Swarm = 5,
    /// Fase quântica (LC-LF)
    Quantum = 6,
}

impl PipelineStage {
    /// Retorna camadas relacionadas ao estágio
    pub fn layers(&self) -> Vec<LayerId> {
        match self {
            PipelineStage::Sense => vec![0, 1, 2, 3, 4],
            PipelineStage::Process => vec![5, 7],
            PipelineStage::Actuate => vec![6],
            PipelineStage::Network => vec![8],
            PipelineStage::Govern => vec![9, 10],
            PipelineStage::Swarm => vec![11],
            PipelineStage::Quantum => vec![12, 13, 14, 15],
        }
    }

    /// Retorna próximo estágio
    pub fn next(&self) -> Option<PipelineStage> {
        match self {
            PipelineStage::Sense => Some(PipelineStage::Process),
            PipelineStage::Process => Some(PipelineStage::Actuate),
            PipelineStage::Actuate => Some(PipelineStage::Network),
            PipelineStage::Network => Some(PipelineStage::Govern),
            PipelineStage::Govern => Some(PipelineStage::Swarm),
            PipelineStage::Swarm => Some(PipelineStage::Quantum),
            PipelineStage::Quantum => None,
        }
    }

    /// Lista todos os estágios em ordem
    pub fn all() -> Vec<PipelineStage> {
        vec![
            PipelineStage::Sense,
            PipelineStage::Process,
            PipelineStage::Actuate,
            PipelineStage::Network,
            PipelineStage::Govern,
            PipelineStage::Swarm,
            PipelineStage::Quantum,
        ]
    }
}

/// Pipeline de execução
#[derive(Debug, Clone)]
pub struct Pipeline {
    /// Estágios habilitados
    enabled_stages: Vec<PipelineStage>,
    /// Estágio atual
    current_stage: Option<PipelineStage>,
    /// Número de ciclos executados
    cycles: u64,
}

impl Pipeline {
    /// Cria novo pipeline com todos os estágios
    pub fn new() -> Self {
        Self {
            enabled_stages: PipelineStage::all(),
            current_stage: None,
            cycles: 0,
        }
    }

    /// Cria pipeline com estágios específicos
    pub fn with_stages(stages: Vec<PipelineStage>) -> OrchestrationResult<Self> {
        if stages.is_empty() {
            return Err(OrchestrationError::InvalidPipeline(
                "Pipeline must have at least one stage".into(),
            ));
        }

        let mut sorted_stages = stages;
        sorted_stages.sort();

        Ok(Self {
            enabled_stages: sorted_stages,
            current_stage: None,
            cycles: 0,
        })
    }

    /// Retorna estágios habilitados
    pub fn enabled_stages(&self) -> &[PipelineStage] {
        &self.enabled_stages
    }

    /// Retorna estágio atual
    pub fn current_stage(&self) -> Option<PipelineStage> {
        self.current_stage
    }

    /// Inicia pipeline
    pub fn start(&mut self) {
        self.current_stage = self.enabled_stages.first().copied();
        self.cycles = 0;
    }

    /// Avança para próximo estágio
    pub fn next_stage(&mut self) -> Option<PipelineStage> {
        if let Some(current) = self.current_stage {
            // Encontra índice do estágio atual
            if let Some(idx) = self.enabled_stages.iter().position(|&s| s == current) {
                // Pega próximo estágio habilitado
                if idx + 1 < self.enabled_stages.len() {
                    self.current_stage = Some(self.enabled_stages[idx + 1]);
                } else {
                    // Fim do pipeline, volta ao início
                    self.current_stage = self.enabled_stages.first().copied();
                    self.cycles += 1;
                }
            }
        }
        self.current_stage
    }

    /// Verifica se estágio está habilitado
    pub fn is_stage_enabled(&self, stage: PipelineStage) -> bool {
        self.enabled_stages.contains(&stage)
    }

    /// Habilita estágio
    pub fn enable_stage(&mut self, stage: PipelineStage) {
        if !self.enabled_stages.contains(&stage) {
            self.enabled_stages.push(stage);
            self.enabled_stages.sort();
        }
    }

    /// Desabilita estágio
    pub fn disable_stage(&mut self, stage: PipelineStage) -> OrchestrationResult<()> {
        if self.enabled_stages.len() <= 1 {
            return Err(OrchestrationError::InvalidPipeline(
                "Cannot disable last stage".into(),
            ));
        }
        self.enabled_stages.retain(|&s| s != stage);
        Ok(())
    }

    /// Retorna número de ciclos completos
    pub fn cycles(&self) -> u64 {
        self.cycles
    }

    /// Reseta pipeline
    pub fn reset(&mut self) {
        self.current_stage = None;
        self.cycles = 0;
    }

    /// Verifica se pipeline está rodando
    pub fn is_running(&self) -> bool {
        self.current_stage.is_some()
    }
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_new() {
        let pipeline = Pipeline::new();
        assert_eq!(pipeline.enabled_stages().len(), 7);
        assert_eq!(pipeline.cycles(), 0);
        assert!(!pipeline.is_running());
    }

    #[test]
    fn test_pipeline_with_stages() {
        let stages = vec![PipelineStage::Sense, PipelineStage::Process];
        let pipeline = Pipeline::with_stages(stages).unwrap();
        assert_eq!(pipeline.enabled_stages().len(), 2);
    }

    #[test]
    fn test_pipeline_start() {
        let mut pipeline = Pipeline::new();
        pipeline.start();
        assert!(pipeline.is_running());
        assert_eq!(pipeline.current_stage(), Some(PipelineStage::Sense));
    }

    #[test]
    fn test_pipeline_next_stage() {
        let mut pipeline = Pipeline::new();
        pipeline.start();

        assert_eq!(pipeline.current_stage(), Some(PipelineStage::Sense));

        pipeline.next_stage();
        assert_eq!(pipeline.current_stage(), Some(PipelineStage::Process));

        pipeline.next_stage();
        assert_eq!(pipeline.current_stage(), Some(PipelineStage::Actuate));
    }

    #[test]
    fn test_pipeline_cycle() {
        let stages = vec![PipelineStage::Sense, PipelineStage::Process];
        let mut pipeline = Pipeline::with_stages(stages).unwrap();
        pipeline.start();

        assert_eq!(pipeline.cycles(), 0);
        assert_eq!(pipeline.current_stage(), Some(PipelineStage::Sense));

        pipeline.next_stage();
        assert_eq!(pipeline.current_stage(), Some(PipelineStage::Process));

        // Próximo volta ao início e incrementa ciclo
        pipeline.next_stage();
        assert_eq!(pipeline.current_stage(), Some(PipelineStage::Sense));
        assert_eq!(pipeline.cycles(), 1);
    }

    #[test]
    fn test_pipeline_enable_disable_stage() {
        let mut pipeline = Pipeline::with_stages(vec![PipelineStage::Sense]).unwrap();

        assert!(!pipeline.is_stage_enabled(PipelineStage::Process));

        pipeline.enable_stage(PipelineStage::Process);
        assert!(pipeline.is_stage_enabled(PipelineStage::Process));

        pipeline.disable_stage(PipelineStage::Process).unwrap();
        assert!(!pipeline.is_stage_enabled(PipelineStage::Process));
    }

    #[test]
    fn test_pipeline_cannot_disable_last_stage() {
        let mut pipeline = Pipeline::with_stages(vec![PipelineStage::Sense]).unwrap();
        let result = pipeline.disable_stage(PipelineStage::Sense);
        assert!(result.is_err());
    }

    #[test]
    fn test_pipeline_reset() {
        let mut pipeline = Pipeline::new();
        pipeline.start();
        pipeline.next_stage();
        pipeline.next_stage();

        assert!(pipeline.is_running());

        pipeline.reset();
        assert!(!pipeline.is_running());
        assert_eq!(pipeline.cycles(), 0);
    }

    #[test]
    fn test_pipeline_stage_layers() {
        let sense_layers = PipelineStage::Sense.layers();
        assert_eq!(sense_layers, vec![0, 1, 2, 3, 4]);

        let process_layers = PipelineStage::Process.layers();
        assert_eq!(process_layers, vec![5, 7]);
    }

    #[test]
    fn test_pipeline_stage_next() {
        assert_eq!(PipelineStage::Sense.next(), Some(PipelineStage::Process));
        assert_eq!(PipelineStage::Process.next(), Some(PipelineStage::Actuate));
        assert_eq!(PipelineStage::Quantum.next(), None);
    }

    #[test]
    fn test_pipeline_stage_all() {
        let all_stages = PipelineStage::all();
        assert_eq!(all_stages.len(), 7);
        assert_eq!(all_stages[0], PipelineStage::Sense);
        assert_eq!(all_stages[6], PipelineStage::Quantum);
    }

    #[test]
    fn test_empty_pipeline_stages() {
        let result = Pipeline::with_stages(vec![]);
        assert!(result.is_err());
    }
}
