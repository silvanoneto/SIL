//! Medidor de energia (EnergyMeter)
//!
//! Componente principal para medição de consumo energético em tempo real.

use crate::{
    EnergyModel, EnergyResult, EnergyError,
    PipelineStage, ProcessorType,
    snapshot::{EnergySnapshot, EnergyStats, EnergyAccumulator},
};
use std::time::{Duration, Instant};
use std::collections::VecDeque;

/// Configuração do medidor de energia
#[derive(Debug, Clone)]
pub struct MeterConfig {
    /// Tamanho do histórico de snapshots
    pub history_size: usize,
    /// Intervalo mínimo entre medições (para evitar overhead)
    pub min_interval: Duration,
    /// Habilitar acumulação contínua
    pub enable_accumulation: bool,
    /// Janela para cálculo de taxa (Watts)
    pub rate_window: Duration,
    /// Habilitar detecção automática de modelo
    pub auto_detect_model: bool,
}

impl Default for MeterConfig {
    fn default() -> Self {
        Self {
            history_size: 1000,
            min_interval: Duration::from_micros(100),
            enable_accumulation: true,
            rate_window: Duration::from_secs(1),
            auto_detect_model: true,
        }
    }
}

impl MeterConfig {
    /// Configuração para alta precisão (mais overhead)
    pub fn high_precision() -> Self {
        Self {
            history_size: 10000,
            min_interval: Duration::from_micros(10),
            enable_accumulation: true,
            rate_window: Duration::from_millis(100),
            auto_detect_model: true,
        }
    }

    /// Configuração para baixo overhead
    pub fn low_overhead() -> Self {
        Self {
            history_size: 100,
            min_interval: Duration::from_millis(1),
            enable_accumulation: false,
            rate_window: Duration::from_secs(5),
            auto_detect_model: false,
        }
    }
}

/// Estado de uma medição em progresso
#[derive(Debug)]
struct MeasurementState {
    /// Início da medição
    start: Instant,
    /// Estágio do pipeline (se aplicável)
    stage: Option<PipelineStage>,
    /// Operações no início
    start_operations: u64,
    /// Utilização estimada
    utilization: f32,
}

/// Medidor de energia
pub struct EnergyMeter {
    /// Modelo de energia
    model: Box<dyn EnergyModel>,
    /// Configuração
    config: MeterConfig,
    /// Histórico de snapshots
    history: VecDeque<EnergySnapshot>,
    /// Estatísticas agregadas
    stats: EnergyStats,
    /// Acumulador contínuo
    accumulator: EnergyAccumulator,
    /// Medição em progresso
    measurement: Option<MeasurementState>,
    /// Contador de operações globais
    total_operations: u64,
    /// Última medição (para min_interval)
    last_measurement: Option<Instant>,
}

impl EnergyMeter {
    /// Cria novo medidor com modelo específico
    pub fn new(model: Box<dyn EnergyModel>) -> Self {
        Self::with_config(model, MeterConfig::default())
    }

    /// Cria medidor com configuração customizada
    pub fn with_config(model: Box<dyn EnergyModel>, config: MeterConfig) -> Self {
        Self {
            model,
            accumulator: EnergyAccumulator::new(config.rate_window),
            config,
            history: VecDeque::new(),
            stats: EnergyStats::new(),
            measurement: None,
            total_operations: 0,
            last_measurement: None,
        }
    }

    /// Cria medidor com detecção automática de modelo
    pub fn auto_detect() -> Self {
        let model = ProcessorType::Cpu.default_model();
        Self::new(model)
    }

    /// Cria medidor para tipo de processador específico
    pub fn for_processor(processor: ProcessorType) -> Self {
        Self::new(processor.default_model())
    }

    /// Retorna referência ao modelo de energia
    pub fn model(&self) -> &dyn EnergyModel {
        self.model.as_ref()
    }

    /// Define novo modelo de energia
    pub fn set_model(&mut self, model: Box<dyn EnergyModel>) {
        self.model = model;
    }

    /// Inicia uma medição
    ///
    /// # Erros
    ///
    /// Retorna erro se já houver uma medição em progresso.
    pub fn begin_measurement(&mut self) -> EnergyResult<()> {
        if self.measurement.is_some() {
            return Err(EnergyError::MeasurementInProgress);
        }

        self.measurement = Some(MeasurementState {
            start: Instant::now(),
            stage: None,
            start_operations: self.total_operations,
            utilization: 0.5, // Default
        });

        Ok(())
    }

    /// Inicia medição para um estágio específico
    pub fn begin_stage_measurement(&mut self, stage: PipelineStage) -> EnergyResult<()> {
        self.begin_measurement()?;
        if let Some(ref mut state) = self.measurement {
            state.stage = Some(stage);
        }
        Ok(())
    }

    /// Define utilização estimada para medição em progresso
    pub fn set_utilization(&mut self, utilization: f32) {
        if let Some(ref mut state) = self.measurement {
            state.utilization = utilization.clamp(0.0, 1.0);
        }
    }

    /// Finaliza a medição e retorna snapshot
    ///
    /// # Argumentos
    ///
    /// * `operations` - Número de operações executadas durante a medição
    ///
    /// # Erros
    ///
    /// Retorna erro se não houver medição em progresso.
    pub fn end_measurement(&mut self, operations: u64) -> EnergyResult<EnergySnapshot> {
        let state = self.measurement.take()
            .ok_or(EnergyError::MeasurementNotStarted)?;

        let duration = state.start.elapsed();

        // Verifica intervalo mínimo
        if let Some(last) = self.last_measurement {
            if last.elapsed() < self.config.min_interval {
                // Retorna snapshot vazio para evitar overhead
                return Ok(EnergySnapshot::default());
            }
        }

        // Calcula energia
        let joules = self.model.estimate_joules(duration, operations, state.utilization);

        // Atualiza contadores
        self.total_operations += operations;
        self.last_measurement = Some(Instant::now());

        // Cria snapshot
        let mut snapshot = EnergySnapshot::new(duration, joules, operations, state.utilization);
        if let Some(stage) = state.stage {
            snapshot = snapshot.with_stage(stage);
        }

        // Atualiza histórico e estatísticas
        self.record_snapshot(&snapshot);

        Ok(snapshot)
    }

    /// Mede uma função/closure e retorna o resultado junto com o snapshot
    pub fn measure<F, T>(&mut self, operations: u64, f: F) -> EnergyResult<(T, EnergySnapshot)>
    where
        F: FnOnce() -> T,
    {
        self.begin_measurement()?;
        let result = f();
        let snapshot = self.end_measurement(operations)?;
        Ok((result, snapshot))
    }

    /// Mede uma função com utilização estimada
    pub fn measure_with_utilization<F, T>(
        &mut self,
        operations: u64,
        utilization: f32,
        f: F,
    ) -> EnergyResult<(T, EnergySnapshot)>
    where
        F: FnOnce() -> T,
    {
        self.begin_measurement()?;
        self.set_utilization(utilization);
        let result = f();
        let snapshot = self.end_measurement(operations)?;
        Ok((result, snapshot))
    }

    /// Registra snapshot (chamado internamente ou para snapshots externos)
    pub fn record_snapshot(&mut self, snapshot: &EnergySnapshot) {
        // Adiciona ao histórico
        self.history.push_back(snapshot.clone());
        while self.history.len() > self.config.history_size {
            self.history.pop_front();
        }

        // Atualiza estatísticas
        self.stats.update(snapshot);

        // Atualiza acumulador
        if self.config.enable_accumulation {
            self.accumulator.add(snapshot.joules, snapshot.operations);
        }
    }

    /// Retorna energia total acumulada (J)
    pub fn total_joules(&self) -> f64 {
        self.stats.total_joules
    }

    /// Retorna potência média atual (W)
    pub fn current_watts(&self) -> f64 {
        self.accumulator.window_watts()
    }

    /// Retorna potência média total (W)
    pub fn average_watts(&self) -> f64 {
        self.accumulator.average_watts()
    }

    /// Retorna eficiência (ops/J)
    pub fn efficiency(&self) -> f64 {
        self.stats.avg_efficiency
    }

    /// Retorna estatísticas agregadas
    pub fn stats(&self) -> &EnergyStats {
        &self.stats
    }

    /// Retorna histórico de snapshots
    pub fn history(&self) -> &VecDeque<EnergySnapshot> {
        &self.history
    }

    /// Retorna os últimos N snapshots
    pub fn recent_snapshots(&self, n: usize) -> Vec<EnergySnapshot> {
        self.history.iter()
            .rev()
            .take(n)
            .cloned()
            .collect()
    }

    /// Reseta medidor (mantém modelo)
    pub fn reset(&mut self) {
        self.history.clear();
        self.stats.reset();
        self.accumulator.reset();
        self.measurement = None;
        self.total_operations = 0;
        self.last_measurement = None;
    }

    /// Verifica se está em medição
    pub fn is_measuring(&self) -> bool {
        self.measurement.is_some()
    }

    /// Estima energia para uma duração futura
    pub fn estimate_future_joules(&self, duration: Duration, operations: u64) -> f64 {
        let utilization = if self.stats.count > 0 {
            // Usa média histórica
            self.stats.total_operations as f64 / self.stats.total_duration.as_secs_f64() / 1e6
        } else {
            0.5
        };

        self.model.estimate_joules(duration, operations, utilization as f32)
    }
}

impl std::fmt::Debug for EnergyMeter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EnergyMeter")
            .field("model", &self.model.name())
            .field("total_joules", &self.total_joules())
            .field("current_watts", &self.current_watts())
            .field("efficiency", &self.efficiency())
            .field("measurements", &self.stats.count)
            .finish()
    }
}

/// Medidor compartilhado thread-safe
pub struct SharedEnergyMeter {
    inner: std::sync::Arc<std::sync::Mutex<EnergyMeter>>,
}

impl SharedEnergyMeter {
    /// Cria novo medidor compartilhado
    pub fn new(meter: EnergyMeter) -> Self {
        Self {
            inner: std::sync::Arc::new(std::sync::Mutex::new(meter)),
        }
    }

    /// Executa operação com lock
    pub fn with_lock<F, T>(&self, f: F) -> EnergyResult<T>
    where
        F: FnOnce(&mut EnergyMeter) -> EnergyResult<T>,
    {
        let mut guard = self.inner.lock()
            .map_err(|_| EnergyError::SystemError("Lock poisoned".into()))?;
        f(&mut guard)
    }

    /// Mede uma função
    pub fn measure<F, T>(&self, operations: u64, f: F) -> EnergyResult<(T, EnergySnapshot)>
    where
        F: FnOnce() -> T,
    {
        self.with_lock(|meter| meter.measure(operations, f))
    }

    /// Retorna energia total
    pub fn total_joules(&self) -> f64 {
        self.inner.lock()
            .map(|m| m.total_joules())
            .unwrap_or(0.0)
    }

    /// Retorna potência atual
    pub fn current_watts(&self) -> f64 {
        self.inner.lock()
            .map(|m| m.current_watts())
            .unwrap_or(0.0)
    }

    /// Clona o Arc interno
    pub fn clone_inner(&self) -> std::sync::Arc<std::sync::Mutex<EnergyMeter>> {
        self.inner.clone()
    }
}

impl Clone for SharedEnergyMeter {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::CpuEnergyModel;

    #[test]
    fn test_meter_creation() {
        let meter = EnergyMeter::auto_detect();
        assert!(!meter.model().name().is_empty());
    }

    #[test]
    fn test_measurement_cycle() {
        let mut meter = EnergyMeter::new(Box::new(CpuEnergyModel::detect()));

        meter.begin_measurement().unwrap();
        std::thread::sleep(Duration::from_millis(10));
        let snapshot = meter.end_measurement(1000).unwrap();

        assert!(snapshot.joules > 0.0);
        assert!(snapshot.duration >= Duration::from_millis(10));
    }

    #[test]
    fn test_measure_closure() {
        let mut meter = EnergyMeter::auto_detect();

        let (result, snapshot) = meter.measure(100, || {
            let mut sum = 0;
            for i in 0..100 {
                sum += i;
            }
            sum
        }).unwrap();

        assert_eq!(result, 4950);
        assert!(snapshot.operations == 100);
    }

    #[test]
    fn test_measurement_not_started_error() {
        let mut meter = EnergyMeter::auto_detect();
        let result = meter.end_measurement(100);
        assert!(matches!(result, Err(EnergyError::MeasurementNotStarted)));
    }

    #[test]
    fn test_measurement_in_progress_error() {
        let mut meter = EnergyMeter::auto_detect();
        meter.begin_measurement().unwrap();
        let result = meter.begin_measurement();
        assert!(matches!(result, Err(EnergyError::MeasurementInProgress)));
    }

    #[test]
    fn test_stats_accumulation() {
        let mut meter = EnergyMeter::auto_detect();

        for _ in 0..10 {
            meter.begin_measurement().unwrap();
            std::thread::sleep(Duration::from_micros(100));
            meter.end_measurement(100).unwrap();
        }

        assert_eq!(meter.stats().count, 10);
        assert!(meter.total_joules() > 0.0);
    }

    #[test]
    fn test_shared_meter() {
        let meter = SharedEnergyMeter::new(EnergyMeter::auto_detect());

        let (_, snapshot) = meter.measure(100, || 42).unwrap();
        assert!(snapshot.operations == 100);
    }

    #[test]
    fn test_stage_measurement() {
        let mut meter = EnergyMeter::auto_detect();

        meter.begin_stage_measurement(PipelineStage::Process).unwrap();
        std::thread::sleep(Duration::from_micros(100));
        let snapshot = meter.end_measurement(500).unwrap();

        assert_eq!(snapshot.stage, Some(PipelineStage::Process));
    }
}
