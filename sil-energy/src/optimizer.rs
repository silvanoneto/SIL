//! Otimizador de energia
//!
//! Estratégias para redução automática de consumo energético.

use crate::{
    ProcessorType, PipelineStage,
    snapshot::{EnergySnapshot, EnergyStats},
};
use std::time::Duration;
use std::collections::HashMap;

/// Orçamento de energia
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PowerBudget {
    /// Energia máxima permitida (J)
    pub max_joules: f64,
    /// Potência máxima permitida (W)
    pub max_watts: f64,
    /// Duração do orçamento
    pub duration: Duration,
    /// Energia consumida até agora (J)
    pub consumed_joules: f64,
    /// Alerta em % do orçamento
    pub alert_threshold_percent: f64,
}

impl PowerBudget {
    /// Cria novo orçamento
    pub fn new(max_joules: f64, max_watts: f64, duration: Duration) -> Self {
        Self {
            max_joules,
            max_watts,
            duration,
            consumed_joules: 0.0,
            alert_threshold_percent: 80.0,
        }
    }

    /// Cria orçamento baseado em mWh
    pub fn from_milliwatt_hours(mwh: f64, max_watts: f64, duration: Duration) -> Self {
        Self::new(mwh * 3.6, max_watts, duration) // 1 mWh = 3.6 J
    }

    /// Atualiza consumo
    pub fn consume(&mut self, joules: f64) {
        self.consumed_joules += joules;
    }

    /// Verifica se está dentro do orçamento
    pub fn is_within_budget(&self) -> bool {
        self.consumed_joules <= self.max_joules
    }

    /// Verifica se atingiu threshold de alerta
    pub fn should_alert(&self) -> bool {
        let percent_used = (self.consumed_joules / self.max_joules) * 100.0;
        percent_used >= self.alert_threshold_percent
    }

    /// Retorna porcentagem utilizada
    pub fn percent_used(&self) -> f64 {
        (self.consumed_joules / self.max_joules) * 100.0
    }

    /// Retorna energia disponível (J)
    pub fn available_joules(&self) -> f64 {
        (self.max_joules - self.consumed_joules).max(0.0)
    }

    /// Estima duração restante baseado no consumo atual
    pub fn estimated_remaining(&self, current_watts: f64) -> Duration {
        if current_watts <= 0.0 {
            return Duration::MAX;
        }
        let remaining_joules = self.available_joules();
        let remaining_seconds = remaining_joules / current_watts;
        Duration::from_secs_f64(remaining_seconds)
    }

    /// Reseta orçamento
    pub fn reset(&mut self) {
        self.consumed_joules = 0.0;
    }
}

impl Default for PowerBudget {
    fn default() -> Self {
        Self::new(
            100.0,                        // 100 J
            10.0,                         // 10 W
            Duration::from_secs(60),      // 1 minuto
        )
    }
}

/// Estratégia de otimização
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum OptimizationStrategy {
    /// Sem otimização (performance máxima)
    None,
    /// Reduz frequência quando possível
    FrequencyScaling,
    /// Migra trabalho para processador mais eficiente
    ProcessorMigration,
    /// Batching de operações
    OperationBatching,
    /// Desliga componentes ociosos
    PowerGating,
    /// Combinação de todas as estratégias
    Aggressive,
    /// Balanceamento entre performance e energia
    Balanced,
}

impl Default for OptimizationStrategy {
    fn default() -> Self {
        OptimizationStrategy::Balanced
    }
}

/// Recomendação de otimização
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OptimizationRecommendation {
    /// Tipo de recomendação
    pub strategy: OptimizationStrategy,
    /// Descrição
    pub description: String,
    /// Economia estimada (J)
    pub estimated_savings_joules: f64,
    /// Impacto na performance (0.0 - 1.0, onde 1.0 = sem impacto)
    pub performance_impact: f64,
    /// Prioridade (1-10, onde 10 = máxima)
    pub priority: u8,
    /// Parâmetros sugeridos
    pub parameters: HashMap<String, String>,
}

impl OptimizationRecommendation {
    /// Cria nova recomendação
    pub fn new(
        strategy: OptimizationStrategy,
        description: impl Into<String>,
        estimated_savings: f64,
        performance_impact: f64,
    ) -> Self {
        Self {
            strategy,
            description: description.into(),
            estimated_savings_joules: estimated_savings,
            performance_impact: performance_impact.clamp(0.0, 1.0),
            priority: 5,
            parameters: HashMap::new(),
        }
    }

    /// Define prioridade
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority.min(10);
        self
    }

    /// Adiciona parâmetro
    pub fn with_parameter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.parameters.insert(key.into(), value.into());
        self
    }
}

/// Otimizador de energia
pub struct EnergyOptimizer {
    /// Estratégia atual
    strategy: OptimizationStrategy,
    /// Orçamento de energia
    budget: Option<PowerBudget>,
    /// Estatísticas de referência
    stats: EnergyStats,
    /// Histórico de recomendações aplicadas
    applied_recommendations: Vec<OptimizationRecommendation>,
    /// Energia economizada total (J)
    total_savings_joules: f64,
    /// Mapeamento de eficiência por processador
    processor_efficiency: HashMap<ProcessorType, f64>,
    /// Mapeamento de eficiência por estágio
    stage_efficiency: HashMap<PipelineStage, f64>,
}

impl EnergyOptimizer {
    /// Cria novo otimizador
    pub fn new(strategy: OptimizationStrategy) -> Self {
        let mut processor_efficiency = HashMap::new();
        // Eficiência relativa (ops/J normalizado)
        processor_efficiency.insert(ProcessorType::Cpu, 1.0);
        processor_efficiency.insert(ProcessorType::Gpu, 3.0); // GPU mais eficiente para paralelo
        processor_efficiency.insert(ProcessorType::Npu, 10.0); // NPU muito eficiente para inferência
        processor_efficiency.insert(ProcessorType::Fpga, 5.0);
        processor_efficiency.insert(ProcessorType::Hybrid, 2.0);

        Self {
            strategy,
            budget: None,
            stats: EnergyStats::new(),
            applied_recommendations: Vec::new(),
            total_savings_joules: 0.0,
            processor_efficiency,
            stage_efficiency: HashMap::new(),
        }
    }

    /// Define orçamento de energia
    pub fn with_budget(mut self, budget: PowerBudget) -> Self {
        self.budget = Some(budget);
        self
    }

    /// Atualiza estatísticas com novo snapshot
    pub fn update(&mut self, snapshot: &EnergySnapshot) {
        self.stats.update(snapshot);

        // Atualiza eficiência por estágio
        if let Some(stage) = snapshot.stage {
            let efficiency = snapshot.efficiency();
            self.stage_efficiency.entry(stage)
                .and_modify(|e| *e = (*e + efficiency) / 2.0)
                .or_insert(efficiency);
        }

        // Atualiza orçamento
        if let Some(ref mut budget) = self.budget {
            budget.consume(snapshot.joules);
        }
    }

    /// Analisa e gera recomendações
    pub fn analyze(&self) -> Vec<OptimizationRecommendation> {
        let mut recommendations = Vec::new();

        match self.strategy {
            OptimizationStrategy::None => {}
            OptimizationStrategy::Balanced => {
                recommendations.extend(self.analyze_balanced());
            }
            OptimizationStrategy::Aggressive => {
                recommendations.extend(self.analyze_aggressive());
            }
            _ => {
                recommendations.extend(self.analyze_specific(self.strategy));
            }
        }

        // Ordena por prioridade
        recommendations.sort_by(|a, b| b.priority.cmp(&a.priority));
        recommendations
    }

    /// Análise balanceada
    fn analyze_balanced(&self) -> Vec<OptimizationRecommendation> {
        let mut recommendations = Vec::new();

        // Verifica se está usando muito energia
        if self.stats.avg_watts > 10.0 {
            recommendations.push(
                OptimizationRecommendation::new(
                    OptimizationStrategy::FrequencyScaling,
                    "Reduzir frequência do processador em períodos de baixa carga",
                    self.stats.total_joules * 0.1, // ~10% economia
                    0.9, // 90% performance
                ).with_priority(7)
            );
        }

        // Verifica eficiência por estágio
        for (stage, efficiency) in &self.stage_efficiency {
            if *efficiency < 1e8 { // Menos de 100M ops/J
                let better_proc = stage.preferred_processor();
                recommendations.push(
                    OptimizationRecommendation::new(
                        OptimizationStrategy::ProcessorMigration,
                        format!("Migrar estágio {:?} para {:?} para maior eficiência", stage, better_proc),
                        self.stats.total_joules * 0.15,
                        0.95,
                    ).with_priority(6)
                    .with_parameter("stage", format!("{:?}", stage))
                    .with_parameter("target_processor", format!("{:?}", better_proc))
                );
            }
        }

        // Verifica orçamento
        if let Some(ref budget) = self.budget {
            if budget.should_alert() {
                recommendations.push(
                    OptimizationRecommendation::new(
                        OptimizationStrategy::Aggressive,
                        format!("Orçamento em {:.1}% - aplicar otimizações agressivas", budget.percent_used()),
                        budget.available_joules() * 0.3,
                        0.7,
                    ).with_priority(10)
                );
            }
        }

        recommendations
    }

    /// Análise agressiva
    fn analyze_aggressive(&self) -> Vec<OptimizationRecommendation> {
        let mut recommendations = self.analyze_balanced();

        // Adiciona batching
        recommendations.push(
            OptimizationRecommendation::new(
                OptimizationStrategy::OperationBatching,
                "Agrupar operações em lotes para reduzir overhead",
                self.stats.total_joules * 0.2,
                0.85,
            ).with_priority(8)
            .with_parameter("batch_size", "64")
        );

        // Power gating
        recommendations.push(
            OptimizationRecommendation::new(
                OptimizationStrategy::PowerGating,
                "Desligar camadas não utilizadas durante idle",
                self.stats.total_joules * 0.05,
                1.0,
            ).with_priority(5)
        );

        recommendations
    }

    /// Análise para estratégia específica
    fn analyze_specific(&self, strategy: OptimizationStrategy) -> Vec<OptimizationRecommendation> {
        match strategy {
            OptimizationStrategy::FrequencyScaling => {
                vec![
                    OptimizationRecommendation::new(
                        OptimizationStrategy::FrequencyScaling,
                        "Habilitar frequency scaling dinâmico",
                        self.stats.total_joules * 0.15,
                        0.85,
                    ).with_priority(8)
                ]
            }
            OptimizationStrategy::ProcessorMigration => {
                let mut recs = Vec::new();
                // Sugere migração para o processador mais eficiente disponível
                let (best_proc, _) = self.processor_efficiency.iter()
                    .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                    .unwrap();

                recs.push(
                    OptimizationRecommendation::new(
                        OptimizationStrategy::ProcessorMigration,
                        format!("Migrar cargas intensivas para {:?}", best_proc),
                        self.stats.total_joules * 0.25,
                        0.9,
                    ).with_priority(7)
                );
                recs
            }
            OptimizationStrategy::OperationBatching => {
                vec![
                    OptimizationRecommendation::new(
                        OptimizationStrategy::OperationBatching,
                        "Aumentar tamanho de batch para 128 operações",
                        self.stats.total_joules * 0.2,
                        0.9,
                    ).with_priority(6)
                    .with_parameter("batch_size", "128")
                ]
            }
            OptimizationStrategy::PowerGating => {
                vec![
                    OptimizationRecommendation::new(
                        OptimizationStrategy::PowerGating,
                        "Habilitar power gating para camadas L5-L7 em idle",
                        self.stats.total_joules * 0.1,
                        1.0,
                    ).with_priority(5)
                ]
            }
            _ => Vec::new(),
        }
    }

    /// Aplica recomendação (registra como aplicada)
    pub fn apply_recommendation(&mut self, recommendation: OptimizationRecommendation) {
        self.total_savings_joules += recommendation.estimated_savings_joules;
        self.applied_recommendations.push(recommendation);
    }

    /// Retorna economia total estimada (J)
    pub fn total_savings(&self) -> f64 {
        self.total_savings_joules
    }

    /// Retorna recomendações aplicadas
    pub fn applied_recommendations(&self) -> &[OptimizationRecommendation] {
        &self.applied_recommendations
    }

    /// Verifica se está dentro do orçamento
    pub fn is_within_budget(&self) -> bool {
        self.budget.as_ref().map(|b| b.is_within_budget()).unwrap_or(true)
    }

    /// Retorna orçamento
    pub fn budget(&self) -> Option<&PowerBudget> {
        self.budget.as_ref()
    }

    /// Retorna orçamento mutável
    pub fn budget_mut(&mut self) -> Option<&mut PowerBudget> {
        self.budget.as_mut()
    }

    /// Sugere processador ideal para uma carga de trabalho
    pub fn suggest_processor(&self, operations: u64, is_parallel: bool) -> ProcessorType {
        if is_parallel && operations > 1000 {
            // Para cargas paralelas grandes, GPU é melhor
            ProcessorType::Gpu
        } else if operations > 10000 {
            // Para muitas operações sequenciais, NPU pode ser melhor
            ProcessorType::Npu
        } else {
            // Para cargas leves, CPU é suficiente
            ProcessorType::Cpu
        }
    }

    /// Estima economia potencial para mudança de processador
    pub fn estimate_migration_savings(
        &self,
        from: ProcessorType,
        to: ProcessorType,
        joules_current: f64,
    ) -> f64 {
        let eff_from = self.processor_efficiency.get(&from).unwrap_or(&1.0);
        let eff_to = self.processor_efficiency.get(&to).unwrap_or(&1.0);

        if eff_to > eff_from {
            joules_current * (1.0 - eff_from / eff_to)
        } else {
            0.0 // Não vale a pena migrar
        }
    }

    /// Reseta otimizador
    pub fn reset(&mut self) {
        self.stats.reset();
        self.applied_recommendations.clear();
        self.total_savings_joules = 0.0;
        self.stage_efficiency.clear();
        if let Some(ref mut budget) = self.budget {
            budget.reset();
        }
    }
}

impl Default for EnergyOptimizer {
    fn default() -> Self {
        Self::new(OptimizationStrategy::Balanced)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_power_budget() {
        let mut budget = PowerBudget::new(10.0, 5.0, Duration::from_secs(60));

        budget.consume(3.0);
        assert!(budget.is_within_budget());
        assert!(!budget.should_alert());

        budget.consume(6.0); // Total: 9J (90%)
        assert!(budget.should_alert());

        budget.consume(2.0); // Total: 11J (110%)
        assert!(!budget.is_within_budget());
    }

    #[test]
    fn test_optimizer_recommendations() {
        let mut optimizer = EnergyOptimizer::new(OptimizationStrategy::Balanced);

        // Simula snapshots com alto consumo
        for _ in 0..10 {
            let snapshot = EnergySnapshot::new(
                Duration::from_millis(100),
                1.5, // 1.5J = 15W
                10000,
                0.8,
            );
            optimizer.update(&snapshot);
        }

        let recommendations = optimizer.analyze();
        assert!(!recommendations.is_empty());
    }

    #[test]
    fn test_optimizer_with_budget() {
        let budget = PowerBudget::new(5.0, 10.0, Duration::from_secs(60));
        let mut optimizer = EnergyOptimizer::new(OptimizationStrategy::Balanced)
            .with_budget(budget);

        // Consome 90% do orçamento
        for _ in 0..9 {
            let snapshot = EnergySnapshot::new(
                Duration::from_millis(100),
                0.5,
                1000,
                0.5,
            );
            optimizer.update(&snapshot);
        }

        assert!(optimizer.budget().unwrap().should_alert());

        let recommendations = optimizer.analyze();
        // Deve ter recomendação de alta prioridade por orçamento
        assert!(recommendations.iter().any(|r| r.priority == 10));
    }

    #[test]
    fn test_suggest_processor() {
        let optimizer = EnergyOptimizer::default();

        assert_eq!(optimizer.suggest_processor(100, false), ProcessorType::Cpu);
        assert_eq!(optimizer.suggest_processor(5000, true), ProcessorType::Gpu);
        assert_eq!(optimizer.suggest_processor(50000, false), ProcessorType::Npu);
    }

    #[test]
    fn test_migration_savings() {
        let optimizer = EnergyOptimizer::default();

        let savings = optimizer.estimate_migration_savings(
            ProcessorType::Cpu,
            ProcessorType::Npu,
            10.0, // 10J
        );

        assert!(savings > 0.0); // NPU é mais eficiente
        assert!(savings < 10.0); // Economia < 100%
    }
}
