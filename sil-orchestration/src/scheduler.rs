//! Scheduler para controle de execução do pipeline

use std::time::{Duration, Instant};
use crate::error::OrchestrationResult;

/// Configuração do scheduler
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SchedulerConfig {
    /// Taxa de execução (ticks por segundo)
    pub target_rate_hz: f64,
    /// Modo de execução
    pub mode: SchedulerMode,
    /// Permitir burst (executar múltiplos ticks se atrasado)
    pub allow_burst: bool,
    /// Número máximo de ticks em burst
    pub max_burst_ticks: usize,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            target_rate_hz: 100.0, // 100 Hz = 10ms por tick
            mode: SchedulerMode::FixedRate,
            allow_burst: false,
            max_burst_ticks: 5,
        }
    }
}

/// Modo de execução do scheduler
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SchedulerMode {
    /// Taxa fixa (fixed rate) - mantém intervalo constante
    FixedRate,
    /// Delay fixo (fixed delay) - espera após cada execução
    FixedDelay,
    /// Best effort - executa o mais rápido possível
    BestEffort,
}

/// Scheduler de pipeline
#[derive(Debug)]
pub struct Scheduler {
    config: SchedulerConfig,
    last_tick: Option<Instant>,
    tick_count: u64,
    missed_ticks: u64,
    total_execution_time: Duration,
    min_execution_time: Option<Duration>,
    max_execution_time: Option<Duration>,
}

impl Scheduler {
    /// Cria novo scheduler
    pub fn new(config: SchedulerConfig) -> Self {
        Self {
            config,
            last_tick: None,
            tick_count: 0,
            missed_ticks: 0,
            total_execution_time: Duration::ZERO,
            min_execution_time: None,
            max_execution_time: None,
        }
    }

    /// Cria scheduler com taxa padrão
    pub fn with_rate_hz(rate_hz: f64) -> Self {
        let mut config = SchedulerConfig::default();
        config.target_rate_hz = rate_hz;
        Self::new(config)
    }

    /// Retorna intervalo alvo entre ticks
    pub fn target_interval(&self) -> Duration {
        Duration::from_secs_f64(1.0 / self.config.target_rate_hz)
    }

    /// Aguarda até o próximo tick
    pub fn wait_for_next_tick(&mut self) -> OrchestrationResult<TickInfo> {
        let now = Instant::now();
        let target_interval = self.target_interval();

        match self.config.mode {
            SchedulerMode::FixedRate => {
                if let Some(last) = self.last_tick {
                    let elapsed = now.duration_since(last);

                    if elapsed < target_interval {
                        // Ainda não chegou no próximo tick
                        let sleep_duration = target_interval - elapsed;
                        std::thread::sleep(sleep_duration);
                    } else if elapsed > target_interval * 2 {
                        // Múltiplos ticks foram perdidos
                        let missed = (elapsed.as_secs_f64() / target_interval.as_secs_f64()) as u64 - 1;
                        self.missed_ticks += missed;
                    }
                }

                self.last_tick = Some(Instant::now());
                self.tick_count += 1;

                Ok(TickInfo {
                    tick_number: self.tick_count,
                    elapsed: self.last_tick.unwrap().duration_since(now),
                    on_time: true,
                })
            }
            SchedulerMode::FixedDelay => {
                if let Some(last) = self.last_tick {
                    let elapsed = now.duration_since(last);
                    if elapsed < target_interval {
                        std::thread::sleep(target_interval - elapsed);
                    }
                }

                self.last_tick = Some(Instant::now());
                self.tick_count += 1;

                Ok(TickInfo {
                    tick_number: self.tick_count,
                    elapsed: Duration::ZERO,
                    on_time: true,
                })
            }
            SchedulerMode::BestEffort => {
                self.last_tick = Some(now);
                self.tick_count += 1;

                Ok(TickInfo {
                    tick_number: self.tick_count,
                    elapsed: Duration::ZERO,
                    on_time: true,
                })
            }
        }
    }

    /// Registra tempo de execução de um tick
    pub fn record_execution_time(&mut self, duration: Duration) {
        self.total_execution_time += duration;

        if self.min_execution_time.is_none() || duration < self.min_execution_time.unwrap() {
            self.min_execution_time = Some(duration);
        }

        if self.max_execution_time.is_none() || duration > self.max_execution_time.unwrap() {
            self.max_execution_time = Some(duration);
        }
    }

    /// Retorna estatísticas do scheduler
    pub fn stats(&self) -> SchedulerStats {
        let avg_execution_time = if self.tick_count > 0 {
            self.total_execution_time / self.tick_count as u32
        } else {
            Duration::ZERO
        };

        SchedulerStats {
            tick_count: self.tick_count,
            missed_ticks: self.missed_ticks,
            target_rate_hz: self.config.target_rate_hz,
            actual_rate_hz: if let Some(last) = self.last_tick {
                1.0 / last.elapsed().as_secs_f64()
            } else {
                0.0
            },
            avg_execution_time,
            min_execution_time: self.min_execution_time.unwrap_or(Duration::ZERO),
            max_execution_time: self.max_execution_time.unwrap_or(Duration::ZERO),
        }
    }

    /// Reseta estatísticas
    pub fn reset(&mut self) {
        self.last_tick = None;
        self.tick_count = 0;
        self.missed_ticks = 0;
        self.total_execution_time = Duration::ZERO;
        self.min_execution_time = None;
        self.max_execution_time = None;
    }

    /// Retorna número de ticks executados
    pub fn tick_count(&self) -> u64 {
        self.tick_count
    }

    /// Retorna número de ticks perdidos
    pub fn missed_ticks(&self) -> u64 {
        self.missed_ticks
    }

    /// Verifica se está mantendo a taxa alvo
    pub fn is_on_time(&self) -> bool {
        if self.tick_count == 0 {
            return true;
        }

        let miss_rate = self.missed_ticks as f64 / self.tick_count as f64;
        miss_rate < 0.01 // Menos de 1% de miss rate
    }
}

/// Informações sobre um tick
#[derive(Debug, Clone)]
pub struct TickInfo {
    /// Número do tick
    pub tick_number: u64,
    /// Tempo decorrido desde o último tick
    pub elapsed: Duration,
    /// Se o tick ocorreu no tempo esperado
    pub on_time: bool,
}

/// Estatísticas do scheduler
#[derive(Debug, Clone)]
pub struct SchedulerStats {
    /// Número de ticks executados
    pub tick_count: u64,
    /// Número de ticks perdidos
    pub missed_ticks: u64,
    /// Taxa alvo (Hz)
    pub target_rate_hz: f64,
    /// Taxa real (Hz)
    pub actual_rate_hz: f64,
    /// Tempo médio de execução
    pub avg_execution_time: Duration,
    /// Tempo mínimo de execução
    pub min_execution_time: Duration,
    /// Tempo máximo de execução
    pub max_execution_time: Duration,
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new(SchedulerConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduler_creation() {
        let scheduler = Scheduler::default();
        assert_eq!(scheduler.tick_count(), 0);
        assert_eq!(scheduler.missed_ticks(), 0);
    }

    #[test]
    fn test_scheduler_with_rate() {
        let scheduler = Scheduler::with_rate_hz(50.0);
        assert_eq!(scheduler.config.target_rate_hz, 50.0);
        assert_eq!(scheduler.target_interval(), Duration::from_millis(20));
    }

    #[test]
    fn test_tick_count() {
        let mut scheduler = Scheduler::with_rate_hz(1000.0); // 1kHz para testes rápidos

        scheduler.wait_for_next_tick().unwrap();
        assert_eq!(scheduler.tick_count(), 1);

        scheduler.wait_for_next_tick().unwrap();
        assert_eq!(scheduler.tick_count(), 2);
    }

    #[test]
    fn test_execution_time_recording() {
        let mut scheduler = Scheduler::default();

        scheduler.record_execution_time(Duration::from_millis(5));
        scheduler.record_execution_time(Duration::from_millis(10));
        scheduler.record_execution_time(Duration::from_millis(3));

        let stats = scheduler.stats();
        assert_eq!(stats.min_execution_time, Duration::from_millis(3));
        assert_eq!(stats.max_execution_time, Duration::from_millis(10));
    }

    #[test]
    fn test_scheduler_reset() {
        let mut scheduler = Scheduler::default();

        scheduler.wait_for_next_tick().unwrap();
        assert_eq!(scheduler.tick_count(), 1);

        scheduler.reset();
        assert_eq!(scheduler.tick_count(), 0);
    }

    #[test]
    fn test_best_effort_mode() {
        let config = SchedulerConfig {
            target_rate_hz: 100.0,
            mode: SchedulerMode::BestEffort,
            allow_burst: false,
            max_burst_ticks: 5,
        };

        let mut scheduler = Scheduler::new(config);

        // Best effort não deveria esperar
        let start = Instant::now();
        for _ in 0..10 {
            scheduler.wait_for_next_tick().unwrap();
        }
        let elapsed = start.elapsed();

        // Deveria ter executado muito rápido
        assert!(elapsed < Duration::from_millis(50));
    }

    #[test]
    fn test_stats() {
        let mut scheduler = Scheduler::with_rate_hz(100.0);

        scheduler.wait_for_next_tick().unwrap();
        scheduler.record_execution_time(Duration::from_micros(100));

        let stats = scheduler.stats();
        assert_eq!(stats.tick_count, 1);
        assert_eq!(stats.target_rate_hz, 100.0);
    }
}
