//! # 🔄 sil_loop — Loop Principal do Ciclo SIL
//!
//! Implementação do ciclo fechado L(F) → L(0).

use crate::state::{ByteSil, SilState, layers};
use crate::transforms::SilTransform;

/// Resultado de um ciclo SIL
#[derive(Debug, Clone)]
pub struct CycleResult {
    /// Estado final
    pub state: SilState,
    /// Número de ciclos executados
    pub cycles: usize,
    /// Motivo da parada
    pub stop_reason: StopReason,
    /// Histórico de estados (se habilitado)
    pub history: Option<Vec<SilState>>,
    /// Energia total consumida em Joules (quando medição está habilitada)
    pub energy_joules: Option<f64>,
    /// Tempo total de execução
    pub execution_time: Option<std::time::Duration>,
}

impl CycleResult {
    /// Retorna eficiência energética (ciclos/Joule)
    pub fn energy_efficiency(&self) -> Option<f64> {
        self.energy_joules.map(|j| if j > 0.0 {
            self.cycles as f64 / j
        } else {
            f64::INFINITY
        })
    }

    /// Retorna potência média (Watts)
    pub fn average_watts(&self) -> Option<f64> {
        match (self.energy_joules, self.execution_time) {
            (Some(j), Some(t)) if t.as_secs_f64() > 0.0 => {
                Some(j / t.as_secs_f64())
            }
            _ => None,
        }
    }

    /// Retorna energia por ciclo (Joules/ciclo)
    pub fn joules_per_cycle(&self) -> Option<f64> {
        self.energy_joules.map(|j| if self.cycles > 0 {
            j / self.cycles as f64
        } else {
            0.0
        })
    }
}

/// Motivo de parada do ciclo
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopReason {
    /// Atingiu máximo de ciclos
    MaxCycles,
    /// Colapso natural (LF é null)
    Collapse,
    /// Estado estável (não mudou)
    Stable,
    /// Interrompido externamente
    Interrupted,
}

/// Configuração do ciclo SIL
#[derive(Debug, Clone)]
pub struct CycleConfig {
    /// Máximo de ciclos (0 = infinito)
    pub max_cycles: usize,
    /// Detectar estado estável
    pub detect_stable: bool,
    /// Manter histórico
    pub keep_history: bool,
    /// Fator de feedback L(F) → L(0)
    pub feedback_factor: f64,
}

impl Default for CycleConfig {
    fn default() -> Self {
        Self {
            max_cycles: 1000,
            detect_stable: true,
            keep_history: false,
            feedback_factor: 0.5,
        }
    }
}

impl CycleConfig {
    /// Configuração para debug (com histórico)
    pub fn debug() -> Self {
        Self {
            max_cycles: 100,
            detect_stable: true,
            keep_history: true,
            feedback_factor: 0.5,
        }
    }
    
    /// Configuração para produção (sem histórico, mais ciclos)
    pub fn production() -> Self {
        Self {
            max_cycles: 10000,
            detect_stable: true,
            keep_history: false,
            feedback_factor: 0.5,
        }
    }
}

/// Executa o loop principal SIL
///
/// # Argumentos
///
/// - `initial`: Estado inicial
/// - `transform`: Transformação a aplicar em cada ciclo
/// - `max_cycles`: Máximo de ciclos
///
/// # Retorna
///
/// Estado final após ciclos
///
/// # Exemplo
///
/// ```
/// use sil_core::cycle::sil_loop;
/// use sil_core::transforms::{Pipeline, PhaseShift};
/// use sil_core::state::SilState;
///
/// let pipeline = Pipeline::new(vec![
///     Box::new(PhaseShift(1)),
/// ]);
///
/// let result = sil_loop(SilState::neutral(), &pipeline, 100);
/// ```
pub fn sil_loop<T: SilTransform>(
    initial: SilState,
    transform: &T,
    max_cycles: usize,
) -> SilState {
    let config = CycleConfig {
        max_cycles,
        ..Default::default()
    };
    
    sil_loop_with_config(initial, transform, &config).state
}

/// Executa o loop principal SIL com configuração completa
pub fn sil_loop_with_config<T: SilTransform>(
    initial: SilState,
    transform: &T,
    config: &CycleConfig,
) -> CycleResult {
    let start_time = std::time::Instant::now();
    let mut state = initial;
    let mut history = if config.keep_history {
        Some(vec![state])
    } else {
        None
    };

    let mut cycles = 0;

    let stop_reason = loop {
        // Verificar limite de ciclos
        if config.max_cycles > 0 && cycles >= config.max_cycles {
            break StopReason::MaxCycles;
        }

        // Aplicar transformação
        let new_state = transform.transform(&state);

        // Verificar colapso (LF == null)
        if new_state.layers[layers::COLLAPSE].is_null() {
            state = new_state;
            break StopReason::Collapse;
        }

        // Verificar estabilidade
        if config.detect_stable && new_state == state {
            break StopReason::Stable;
        }

        // Feedback: L(F) influencia L(0) do próximo ciclo
        let feedback = apply_feedback(&new_state, config.feedback_factor);
        state = feedback;

        // Registrar histórico
        if let Some(ref mut h) = history {
            h.push(state);
        }

        cycles += 1;
    };

    let execution_time = start_time.elapsed();

    CycleResult {
        state,
        cycles,
        stop_reason,
        history,
        energy_joules: None, // Será preenchido pelo medidor externo se disponível
        execution_time: Some(execution_time),
    }
}

/// Aplica feedback L(F) → L(0)
fn apply_feedback(state: &SilState, factor: f64) -> SilState {
    let lf = state.layers[layers::COLLAPSE];
    let l0 = state.layers[layers::PHOTONIC];
    
    // Mix ponderado
    let new_l0 = if factor > 0.0 {
        let rho = (l0.rho as f64 * (1.0 - factor) + lf.rho as f64 * factor)
            .round() as i8;
        let theta = (l0.theta as f64 * (1.0 - factor) + lf.theta as f64 * factor)
            .round() as u8;
        ByteSil::new(rho, theta)
    } else {
        l0
    };
    
    state.with_layer(layers::PHOTONIC, new_l0)
}

/// Runner de ciclo com callbacks
pub struct CycleRunner<T: SilTransform> {
    transform: T,
    config: CycleConfig,
    on_cycle: Option<Box<dyn Fn(usize, &SilState)>>,
    on_stop: Option<Box<dyn Fn(StopReason, &SilState)>>,
}

impl<T: SilTransform> CycleRunner<T> {
    pub fn new(transform: T) -> Self {
        Self {
            transform,
            config: CycleConfig::default(),
            on_cycle: None,
            on_stop: None,
        }
    }
    
    pub fn with_config(mut self, config: CycleConfig) -> Self {
        self.config = config;
        self
    }
    
    pub fn on_cycle<F: Fn(usize, &SilState) + 'static>(mut self, callback: F) -> Self {
        self.on_cycle = Some(Box::new(callback));
        self
    }
    
    pub fn on_stop<F: Fn(StopReason, &SilState) + 'static>(mut self, callback: F) -> Self {
        self.on_stop = Some(Box::new(callback));
        self
    }
    
    pub fn run(&self, initial: SilState) -> CycleResult {
        let start_time = std::time::Instant::now();
        let mut state = initial;
        let mut history = if self.config.keep_history {
            Some(vec![state])
        } else {
            None
        };

        let mut cycles = 0;

        let stop_reason = loop {
            // Callback de ciclo
            if let Some(ref callback) = self.on_cycle {
                callback(cycles, &state);
            }

            // Verificar limite
            if self.config.max_cycles > 0 && cycles >= self.config.max_cycles {
                break StopReason::MaxCycles;
            }

            // Aplicar transformação
            let new_state = self.transform.transform(&state);

            // Verificar colapso
            if new_state.layers[layers::COLLAPSE].is_null() {
                state = new_state;
                break StopReason::Collapse;
            }

            // Verificar estabilidade
            if self.config.detect_stable && new_state == state {
                break StopReason::Stable;
            }

            // Feedback
            state = apply_feedback(&new_state, self.config.feedback_factor);

            if let Some(ref mut h) = history {
                h.push(state);
            }

            cycles += 1;
        };

        // Callback de parada
        if let Some(ref callback) = self.on_stop {
            callback(stop_reason, &state);
        }

        let execution_time = start_time.elapsed();

        CycleResult {
            state,
            cycles,
            stop_reason,
            history,
            energy_joules: None,
            execution_time: Some(execution_time),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transforms::{Identity, PhaseShift};
    
    #[test]
    fn test_sil_loop_identity() {
        let state = SilState::neutral();
        let result = sil_loop(state, &Identity, 10);
        
        // Identity não muda nada, deve estabilizar
        assert_eq!(result, state);
    }
    
    #[test]
    fn test_sil_loop_with_config() {
        let config = CycleConfig {
            max_cycles: 5,
            detect_stable: false,
            keep_history: true,
            feedback_factor: 0.0,
        };
        
        let result = sil_loop_with_config(
            SilState::neutral(),
            &PhaseShift(1),
            &config,
        );
        
        assert_eq!(result.cycles, 5);
        assert_eq!(result.stop_reason, StopReason::MaxCycles);
        assert!(result.history.is_some());
    }
    
    #[test]
    fn test_sil_loop_collapse() {
        // Transformação que força colapso
        struct ForceCollapse;
        
        impl SilTransform for ForceCollapse {
            fn transform(&self, state: &SilState) -> SilState {
                state.with_layer(layers::COLLAPSE, ByteSil::NULL)
            }
            
            fn name(&self) -> &'static str {
                "ForceCollapse"
            }
        }
        
        let result = sil_loop_with_config(
            SilState::neutral(),
            &ForceCollapse,
            &CycleConfig::default(),
        );
        
        assert_eq!(result.stop_reason, StopReason::Collapse);
        assert_eq!(result.cycles, 0);
    }
    
    #[test]
    fn test_cycle_runner() {
        let runner = CycleRunner::new(Identity)
            .with_config(CycleConfig {
                max_cycles: 3,
                detect_stable: true,
                ..Default::default()
            });
        
        let result = runner.run(SilState::neutral());
        
        // Identity estabiliza imediatamente
        assert_eq!(result.stop_reason, StopReason::Stable);
    }
}
