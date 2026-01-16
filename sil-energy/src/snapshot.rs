//! Snapshots e estatísticas de energia

use crate::PipelineStage;
use std::time::{Duration, Instant};
use std::collections::HashMap;

/// Snapshot de uma medição de energia
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EnergySnapshot {
    /// Timestamp da medição (ms desde epoch ou início)
    pub timestamp_ms: u64,
    /// Duração da medição
    #[serde(with = "duration_serde")]
    pub duration: Duration,
    /// Estágio do pipeline (se aplicável)
    pub stage: Option<PipelineStage>,
    /// Energia consumida em Joules
    pub joules: f64,
    /// Potência média em Watts
    pub watts: f64,
    /// Número de operações executadas
    pub operations: u64,
    /// Utilização do processador (0.0 - 1.0)
    pub utilization: f32,
    /// Energia por operação (J/op)
    pub joules_per_op: f64,
    /// Ciclos de CPU (se disponível)
    pub cpu_cycles: Option<u64>,
}

impl EnergySnapshot {
    /// Cria snapshot com valores calculados
    pub fn new(
        duration: Duration,
        joules: f64,
        operations: u64,
        utilization: f32,
    ) -> Self {
        let watts = if duration.as_secs_f64() > 0.0 {
            joules / duration.as_secs_f64()
        } else {
            0.0
        };

        let joules_per_op = if operations > 0 {
            joules / operations as f64
        } else {
            0.0
        };

        Self {
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
            duration,
            stage: None,
            joules,
            watts,
            operations,
            utilization,
            joules_per_op,
            cpu_cycles: None,
        }
    }

    /// Adiciona informação de estágio
    pub fn with_stage(mut self, stage: PipelineStage) -> Self {
        self.stage = Some(stage);
        self
    }

    /// Adiciona ciclos de CPU
    pub fn with_cpu_cycles(mut self, cycles: u64) -> Self {
        self.cpu_cycles = Some(cycles);
        self
    }

    /// Retorna eficiência energética (ops/J)
    pub fn efficiency(&self) -> f64 {
        if self.joules > 0.0 {
            self.operations as f64 / self.joules
        } else {
            f64::INFINITY
        }
    }

    /// Converte para mWh (mili-Watt-hora)
    pub fn milliwatt_hours(&self) -> f64 {
        self.joules / 3.6 // 1 Wh = 3600 J, 1 mWh = 3.6 J
    }
}

impl Default for EnergySnapshot {
    fn default() -> Self {
        Self::new(Duration::ZERO, 0.0, 0, 0.0)
    }
}

/// Estatísticas agregadas de energia
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct EnergyStats {
    /// Número de medições
    pub count: u64,
    /// Energia total (J)
    pub total_joules: f64,
    /// Energia mínima por medição (J)
    pub min_joules: f64,
    /// Energia máxima por medição (J)
    pub max_joules: f64,
    /// Energia média por medição (J)
    pub avg_joules: f64,
    /// Potência média (W)
    pub avg_watts: f64,
    /// Potência máxima (W)
    pub max_watts: f64,
    /// Total de operações
    pub total_operations: u64,
    /// Eficiência média (ops/J)
    pub avg_efficiency: f64,
    /// Tempo total de medição
    #[serde(with = "duration_serde")]
    pub total_duration: Duration,
    /// Energia por estágio
    pub stage_energy: HashMap<String, f64>,
}

impl EnergyStats {
    /// Cria estatísticas vazias
    pub fn new() -> Self {
        Self {
            min_joules: f64::INFINITY,
            max_joules: f64::NEG_INFINITY,
            ..Default::default()
        }
    }

    /// Atualiza estatísticas com novo snapshot
    pub fn update(&mut self, snapshot: &EnergySnapshot) {
        self.count += 1;
        self.total_joules += snapshot.joules;
        self.total_operations += snapshot.operations;
        self.total_duration += snapshot.duration;

        if snapshot.joules < self.min_joules {
            self.min_joules = snapshot.joules;
        }
        if snapshot.joules > self.max_joules {
            self.max_joules = snapshot.joules;
        }
        if snapshot.watts > self.max_watts {
            self.max_watts = snapshot.watts;
        }

        // Atualiza médias
        self.avg_joules = self.total_joules / self.count as f64;

        if self.total_duration.as_secs_f64() > 0.0 {
            self.avg_watts = self.total_joules / self.total_duration.as_secs_f64();
        }

        if self.total_joules > 0.0 {
            self.avg_efficiency = self.total_operations as f64 / self.total_joules;
        }

        // Energia por estágio
        if let Some(stage) = &snapshot.stage {
            let key = format!("{:?}", stage);
            *self.stage_energy.entry(key).or_insert(0.0) += snapshot.joules;
        }
    }

    /// Reseta estatísticas
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Retorna energia em mWh
    pub fn total_milliwatt_hours(&self) -> f64 {
        self.total_joules / 3.6
    }
}

/// Relatório completo de energia
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EnergyReport {
    /// Nome do relatório
    pub name: String,
    /// Timestamp de geração
    pub generated_at: String,
    /// Estatísticas gerais
    pub stats: EnergyStats,
    /// Snapshots recentes (últimos N)
    pub recent_snapshots: Vec<EnergySnapshot>,
    /// Orçamento de energia (se definido)
    pub budget_joules: Option<f64>,
    /// Porcentagem do orçamento usada
    pub budget_used_percent: Option<f64>,
    /// Recomendações de otimização
    pub recommendations: Vec<String>,
}

impl EnergyReport {
    /// Cria relatório a partir de estatísticas
    pub fn from_stats(name: impl Into<String>, stats: EnergyStats) -> Self {
        Self {
            name: name.into(),
            generated_at: chrono_lite_now(),
            stats,
            recent_snapshots: Vec::new(),
            budget_joules: None,
            budget_used_percent: None,
            recommendations: Vec::new(),
        }
    }

    /// Adiciona orçamento de energia
    pub fn with_budget(mut self, budget_joules: f64) -> Self {
        self.budget_joules = Some(budget_joules);
        self.budget_used_percent = Some((self.stats.total_joules / budget_joules) * 100.0);
        self
    }

    /// Adiciona snapshots recentes
    pub fn with_snapshots(mut self, snapshots: Vec<EnergySnapshot>) -> Self {
        self.recent_snapshots = snapshots;
        self
    }

    /// Gera recomendações baseadas nas estatísticas
    pub fn generate_recommendations(&mut self) {
        self.recommendations.clear();

        // Análise de eficiência
        if self.stats.avg_efficiency < 1e9 {
            self.recommendations.push(
                "Eficiência baixa: considere usar NPU para operações de inferência".into()
            );
        }

        // Análise de potência
        if self.stats.max_watts > 50.0 {
            self.recommendations.push(
                "Picos de potência detectados: considere distribuir carga no tempo".into()
            );
        }

        // Análise por estágio
        for (stage, energy) in &self.stats.stage_energy {
            let percent = (energy / self.stats.total_joules) * 100.0;
            if percent > 40.0 {
                self.recommendations.push(
                    format!("Estágio '{}' consome {:.1}% da energia total - candidato para otimização", stage, percent)
                );
            }
        }

        // Análise de orçamento
        if let Some(used) = self.budget_used_percent {
            if used > 80.0 {
                self.recommendations.push(
                    format!("Orçamento de energia {:.1}% utilizado - considere reduzir frequência de operações", used)
                );
            }
        }

        if self.recommendations.is_empty() {
            self.recommendations.push("Consumo de energia dentro dos parâmetros esperados".into());
        }
    }

    /// Exporta relatório como JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

/// Helper para serialização de Duration
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_nanos().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let nanos = u128::deserialize(deserializer)?;
        Ok(Duration::from_nanos(nanos as u64))
    }
}

/// Timestamp simples sem dependência externa
fn chrono_lite_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO);
    format!("{}", duration.as_secs())
}

/// Acumulador de energia em tempo real
#[derive(Debug)]
pub struct EnergyAccumulator {
    /// Início da acumulação
    start: Instant,
    /// Energia acumulada (J)
    accumulated_joules: f64,
    /// Operações acumuladas
    accumulated_ops: u64,
    /// Janela de medição (para taxa)
    window_start: Instant,
    /// Energia na janela atual
    window_joules: f64,
    /// Tamanho da janela
    window_size: Duration,
}

impl EnergyAccumulator {
    /// Cria novo acumulador
    pub fn new(window_size: Duration) -> Self {
        let now = Instant::now();
        Self {
            start: now,
            accumulated_joules: 0.0,
            accumulated_ops: 0,
            window_start: now,
            window_joules: 0.0,
            window_size,
        }
    }

    /// Adiciona medição
    pub fn add(&mut self, joules: f64, operations: u64) {
        self.accumulated_joules += joules;
        self.accumulated_ops += operations;
        self.window_joules += joules;

        // Verifica se a janela expirou
        if self.window_start.elapsed() > self.window_size {
            self.window_start = Instant::now();
            self.window_joules = 0.0;
        }
    }

    /// Retorna energia total acumulada (J)
    pub fn total_joules(&self) -> f64 {
        self.accumulated_joules
    }

    /// Retorna potência média na janela (W)
    pub fn window_watts(&self) -> f64 {
        let elapsed = self.window_start.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.window_joules / elapsed
        } else {
            0.0
        }
    }

    /// Retorna potência média total (W)
    pub fn average_watts(&self) -> f64 {
        let elapsed = self.start.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.accumulated_joules / elapsed
        } else {
            0.0
        }
    }

    /// Retorna eficiência (ops/J)
    pub fn efficiency(&self) -> f64 {
        if self.accumulated_joules > 0.0 {
            self.accumulated_ops as f64 / self.accumulated_joules
        } else {
            f64::INFINITY
        }
    }

    /// Reseta o acumulador
    pub fn reset(&mut self) {
        let now = Instant::now();
        self.start = now;
        self.accumulated_joules = 0.0;
        self.accumulated_ops = 0;
        self.window_start = now;
        self.window_joules = 0.0;
    }
}

impl Default for EnergyAccumulator {
    fn default() -> Self {
        Self::new(Duration::from_secs(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_creation() {
        let snapshot = EnergySnapshot::new(
            Duration::from_millis(100),
            0.001, // 1 mJ
            1000,
            0.5,
        );

        assert_eq!(snapshot.joules, 0.001);
        assert!((snapshot.watts - 0.01).abs() < 0.001); // 1 mJ / 100ms = 10 mW
        assert_eq!(snapshot.joules_per_op, 0.000001); // 1 µJ/op
    }

    #[test]
    fn test_snapshot_efficiency() {
        let snapshot = EnergySnapshot::new(
            Duration::from_millis(100),
            0.001,
            1_000_000,
            0.5,
        );

        // 1M ops / 1mJ = 1e9 ops/J
        assert!((snapshot.efficiency() - 1e9).abs() < 1e6);
    }

    #[test]
    fn test_stats_update() {
        let mut stats = EnergyStats::new();

        stats.update(&EnergySnapshot::new(Duration::from_millis(100), 0.001, 1000, 0.5));
        stats.update(&EnergySnapshot::new(Duration::from_millis(100), 0.002, 2000, 0.7));

        assert_eq!(stats.count, 2);
        assert_eq!(stats.total_joules, 0.003);
        assert_eq!(stats.total_operations, 3000);
        assert_eq!(stats.min_joules, 0.001);
        assert_eq!(stats.max_joules, 0.002);
    }

    #[test]
    fn test_report_generation() {
        let mut stats = EnergyStats::new();
        stats.update(&EnergySnapshot::new(Duration::from_millis(100), 0.001, 1000, 0.5));

        let mut report = EnergyReport::from_stats("Test", stats).with_budget(0.01);
        report.generate_recommendations();

        assert!(!report.recommendations.is_empty());
        assert_eq!(report.budget_used_percent, Some(10.0));
    }

    #[test]
    fn test_accumulator() {
        let mut acc = EnergyAccumulator::new(Duration::from_millis(100));

        acc.add(0.001, 1000);
        acc.add(0.002, 2000);

        assert_eq!(acc.total_joules(), 0.003);
        assert!(acc.efficiency() > 0.0);
    }
}
