//! Amostragem de energia
//!
//! Estratégias de amostragem para medição contínua de energia.

use crate::{
    EnergyModel, EnergyResult, EnergyError,
    snapshot::{EnergySnapshot, EnergyAccumulator},
};
use std::time::{Duration, Instant};

/// Modo de amostragem
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SamplingMode {
    /// Amostragem em intervalo fixo
    FixedInterval(u64), // microseconds
    /// Amostragem adaptativa (ajusta baseado na carga)
    Adaptive,
    /// Amostragem por operação (a cada N operações)
    PerOperation(u64),
    /// Amostragem por energia (a cada N Joules)
    PerEnergy(u64), // nanojoules
    /// Sem amostragem (apenas manual)
    Manual,
}

impl Default for SamplingMode {
    fn default() -> Self {
        SamplingMode::FixedInterval(1000) // 1ms
    }
}

/// Sampler de energia
pub struct EnergySampler {
    /// Modelo de energia
    model: Box<dyn EnergyModel>,
    /// Modo de amostragem
    mode: SamplingMode,
    /// Acumulador
    accumulator: EnergyAccumulator,
    /// Última amostra
    last_sample: Instant,
    /// Operações desde última amostra
    ops_since_sample: u64,
    /// Energia desde última amostra (nJ)
    energy_since_sample_nj: u64,
    /// Snapshots coletados
    samples: Vec<EnergySnapshot>,
    /// Capacidade máxima de samples
    max_samples: usize,
    /// Utilização atual estimada
    current_utilization: f32,
}

impl EnergySampler {
    /// Cria novo sampler
    pub fn new(model: Box<dyn EnergyModel>, mode: SamplingMode) -> Self {
        Self {
            model,
            mode,
            accumulator: EnergyAccumulator::default(),
            last_sample: Instant::now(),
            ops_since_sample: 0,
            energy_since_sample_nj: 0,
            samples: Vec::new(),
            max_samples: 10000,
            current_utilization: 0.5,
        }
    }

    /// Define capacidade máxima de samples
    pub fn with_max_samples(mut self, max: usize) -> Self {
        self.max_samples = max;
        self
    }

    /// Atualiza utilização estimada
    pub fn set_utilization(&mut self, utilization: f32) {
        self.current_utilization = utilization.clamp(0.0, 1.0);
    }

    /// Registra operações e verifica se deve amostrar
    ///
    /// Retorna Some(snapshot) se uma amostra foi coletada.
    pub fn record_operations(&mut self, operations: u64) -> Option<EnergySnapshot> {
        self.ops_since_sample += operations;

        // Calcula energia estimada
        let elapsed = self.last_sample.elapsed();
        let joules = self.model.estimate_joules(elapsed, operations, self.current_utilization);
        self.energy_since_sample_nj += (joules * 1e9) as u64;

        // Verifica se deve amostrar
        if self.should_sample() {
            Some(self.take_sample())
        } else {
            None
        }
    }

    /// Verifica se deve coletar amostra
    fn should_sample(&self) -> bool {
        match self.mode {
            SamplingMode::FixedInterval(us) => {
                self.last_sample.elapsed() >= Duration::from_micros(us)
            }
            SamplingMode::Adaptive => {
                // Amostra mais frequentemente com alta utilização
                let base_interval = Duration::from_millis(1);
                let adjusted = base_interval.mul_f32(1.0 - self.current_utilization * 0.9);
                self.last_sample.elapsed() >= adjusted
            }
            SamplingMode::PerOperation(n) => {
                self.ops_since_sample >= n
            }
            SamplingMode::PerEnergy(nj) => {
                self.energy_since_sample_nj >= nj
            }
            SamplingMode::Manual => false,
        }
    }

    /// Coleta amostra
    fn take_sample(&mut self) -> EnergySnapshot {
        let duration = self.last_sample.elapsed();
        let joules = self.energy_since_sample_nj as f64 / 1e9;
        let operations = self.ops_since_sample;

        let snapshot = EnergySnapshot::new(
            duration,
            joules,
            operations,
            self.current_utilization,
        );

        // Atualiza acumulador
        self.accumulator.add(joules, operations);

        // Guarda sample
        if self.samples.len() < self.max_samples {
            self.samples.push(snapshot.clone());
        } else {
            // Circular buffer
            self.samples.remove(0);
            self.samples.push(snapshot.clone());
        }

        // Reset contadores
        self.last_sample = Instant::now();
        self.ops_since_sample = 0;
        self.energy_since_sample_nj = 0;

        snapshot
    }

    /// Força coleta de amostra manual
    pub fn force_sample(&mut self) -> EnergySnapshot {
        self.take_sample()
    }

    /// Retorna energia total acumulada (J)
    pub fn total_joules(&self) -> f64 {
        self.accumulator.total_joules()
    }

    /// Retorna potência média (W)
    pub fn average_watts(&self) -> f64 {
        self.accumulator.average_watts()
    }

    /// Retorna potência na janela atual (W)
    pub fn current_watts(&self) -> f64 {
        self.accumulator.window_watts()
    }

    /// Retorna eficiência (ops/J)
    pub fn efficiency(&self) -> f64 {
        self.accumulator.efficiency()
    }

    /// Retorna samples coletados
    pub fn samples(&self) -> &[EnergySnapshot] {
        &self.samples
    }

    /// Retorna número de samples
    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }

    /// Limpa samples mas mantém acumulador
    pub fn clear_samples(&mut self) {
        self.samples.clear();
    }

    /// Reseta completamente
    pub fn reset(&mut self) {
        self.accumulator.reset();
        self.samples.clear();
        self.last_sample = Instant::now();
        self.ops_since_sample = 0;
        self.energy_since_sample_nj = 0;
    }

    /// Retorna estatísticas resumidas dos samples
    pub fn sample_stats(&self) -> SampleStats {
        if self.samples.is_empty() {
            return SampleStats::default();
        }

        let total_joules: f64 = self.samples.iter().map(|s| s.joules).sum();
        let total_ops: u64 = self.samples.iter().map(|s| s.operations).sum();
        let min_joules = self.samples.iter().map(|s| s.joules).fold(f64::INFINITY, f64::min);
        let max_joules = self.samples.iter().map(|s| s.joules).fold(f64::NEG_INFINITY, f64::max);

        SampleStats {
            count: self.samples.len(),
            total_joules,
            avg_joules: total_joules / self.samples.len() as f64,
            min_joules,
            max_joules,
            total_operations: total_ops,
            efficiency: if total_joules > 0.0 { total_ops as f64 / total_joules } else { 0.0 },
        }
    }
}

/// Estatísticas de samples
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SampleStats {
    /// Número de samples
    pub count: usize,
    /// Energia total (J)
    pub total_joules: f64,
    /// Energia média por sample (J)
    pub avg_joules: f64,
    /// Energia mínima (J)
    pub min_joules: f64,
    /// Energia máxima (J)
    pub max_joules: f64,
    /// Total de operações
    pub total_operations: u64,
    /// Eficiência (ops/J)
    pub efficiency: f64,
}

/// Sampler de energia em tempo real com callback
pub struct RealtimeSampler<F>
where
    F: FnMut(&EnergySnapshot),
{
    /// Sampler interno
    sampler: EnergySampler,
    /// Callback para cada sample
    on_sample: F,
    /// Threshold de potência para alerta (W)
    power_threshold: Option<f64>,
    /// Callback de alerta de potência
    on_power_alert: Option<Box<dyn FnMut(f64)>>,
}

impl<F> RealtimeSampler<F>
where
    F: FnMut(&EnergySnapshot),
{
    /// Cria sampler com callback
    pub fn new(model: Box<dyn EnergyModel>, mode: SamplingMode, on_sample: F) -> Self {
        Self {
            sampler: EnergySampler::new(model, mode),
            on_sample,
            power_threshold: None,
            on_power_alert: None,
        }
    }

    /// Define threshold de potência para alertas
    pub fn with_power_threshold(mut self, watts: f64) -> Self {
        self.power_threshold = Some(watts);
        self
    }

    /// Define callback de alerta de potência
    pub fn with_power_alert<G>(mut self, callback: G) -> Self
    where
        G: FnMut(f64) + 'static,
    {
        self.on_power_alert = Some(Box::new(callback));
        self
    }

    /// Registra operações
    pub fn record(&mut self, operations: u64) {
        if let Some(snapshot) = self.sampler.record_operations(operations) {
            (self.on_sample)(&snapshot);

            // Verifica threshold de potência
            if let Some(threshold) = self.power_threshold {
                if snapshot.watts > threshold {
                    if let Some(ref mut alert) = self.on_power_alert {
                        alert(snapshot.watts);
                    }
                }
            }
        }
    }

    /// Acesso ao sampler interno
    pub fn inner(&self) -> &EnergySampler {
        &self.sampler
    }

    /// Acesso mutável ao sampler interno
    pub fn inner_mut(&mut self) -> &mut EnergySampler {
        &mut self.sampler
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::CpuEnergyModel;

    #[test]
    fn test_sampler_fixed_interval() {
        let model = Box::new(CpuEnergyModel::detect());
        let mut sampler = EnergySampler::new(model, SamplingMode::FixedInterval(100)); // 100µs

        // Primeiro registro não deve gerar sample
        let sample = sampler.record_operations(100);
        assert!(sample.is_none());

        // Espera e tenta novamente
        std::thread::sleep(Duration::from_micros(150));
        let sample = sampler.record_operations(100);
        assert!(sample.is_some());
    }

    #[test]
    fn test_sampler_per_operation() {
        let model = Box::new(CpuEnergyModel::detect());
        let mut sampler = EnergySampler::new(model, SamplingMode::PerOperation(1000));

        // Registra 500 - não deve amostrar
        assert!(sampler.record_operations(500).is_none());

        // Registra mais 600 - deve amostrar
        assert!(sampler.record_operations(600).is_some());
    }

    #[test]
    fn test_sampler_stats() {
        let model = Box::new(CpuEnergyModel::detect());
        let mut sampler = EnergySampler::new(model, SamplingMode::Manual);

        // Força alguns samples
        sampler.record_operations(100);
        sampler.force_sample();

        sampler.record_operations(200);
        sampler.force_sample();

        let stats = sampler.sample_stats();
        assert_eq!(stats.count, 2);
        assert!(stats.total_operations > 0);
    }

    #[test]
    fn test_realtime_sampler() {
        let model = Box::new(CpuEnergyModel::detect());
        let samples = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let samples_clone = samples.clone();

        let mut sampler = RealtimeSampler::new(
            model,
            SamplingMode::PerOperation(100),
            move |snapshot| {
                samples_clone.lock().unwrap().push(snapshot.clone());
            },
        );

        // Registra operações
        for _ in 0..10 {
            sampler.record(50);
        }

        assert!(samples.lock().unwrap().len() >= 4); // 500 ops / 100 = 5 samples (aprox)
    }
}
