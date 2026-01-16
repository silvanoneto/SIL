//! # PID Controller — Controle Proporcional-Integral-Derivativo
//!
//! Implementação clássica de PID com anti-windup e filtro derivativo.
//!
//! ## Teoria
//!
//! ```text
//! u(t) = Kp * e(t) + Ki * ∫e(τ)dτ + Kd * de(t)/dt
//! ```
//!
//! Onde:
//! - e(t) = setpoint - process_variable (erro)
//! - Kp = ganho proporcional
//! - Ki = ganho integral
//! - Kd = ganho derivativo

use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Configuração do PID
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PidConfig {
    /// Ganho proporcional
    pub kp: f64,
    /// Ganho integral
    pub ki: f64,
    /// Ganho derivativo
    pub kd: f64,
    /// Limite mínimo de saída
    pub output_min: f64,
    /// Limite máximo de saída
    pub output_max: f64,
    /// Coeficiente do filtro derivativo (0.0 = sem filtro)
    pub derivative_filter: f64,
    /// Anti-windup: limite do termo integral
    pub integral_limit: f64,
}

impl Default for PidConfig {
    fn default() -> Self {
        Self {
            kp: 1.0,
            ki: 0.1,
            kd: 0.01,
            output_min: -1.0,
            output_max: 1.0,
            derivative_filter: 0.1,
            integral_limit: 10.0,
        }
    }
}

impl PidConfig {
    /// Cria configuração com ganhos específicos
    pub fn new(kp: f64, ki: f64, kd: f64) -> Self {
        Self {
            kp,
            ki,
            kd,
            ..Default::default()
        }
    }

    /// Define limites de saída
    pub fn with_output_limits(mut self, min: f64, max: f64) -> Self {
        self.output_min = min;
        self.output_max = max;
        self
    }

    /// Configuração agressiva (resposta rápida)
    pub fn aggressive() -> Self {
        Self {
            kp: 2.0,
            ki: 0.5,
            kd: 0.1,
            ..Default::default()
        }
    }

    /// Configuração conservadora (resposta suave)
    pub fn conservative() -> Self {
        Self {
            kp: 0.5,
            ki: 0.05,
            kd: 0.02,
            ..Default::default()
        }
    }

    /// Configuração para controle de posição
    pub fn position() -> Self {
        Self {
            kp: 1.0,
            ki: 0.0, // Sem integral para evitar overshoot
            kd: 0.5,
            ..Default::default()
        }
    }

    /// Configuração para controle de velocidade
    pub fn velocity() -> Self {
        Self {
            kp: 0.8,
            ki: 0.2,
            kd: 0.0, // Sem derivativo
            ..Default::default()
        }
    }
}

/// Controlador PID
#[derive(Debug)]
pub struct PidController {
    /// Configuração
    config: PidConfig,
    /// Setpoint (valor desejado)
    setpoint: f64,
    /// Termo integral acumulado
    integral: f64,
    /// Erro anterior (para derivativo)
    prev_error: f64,
    /// Derivativo filtrado
    filtered_derivative: f64,
    /// Último timestamp
    last_time: Option<Instant>,
    /// Última saída
    last_output: f64,
}

impl PidController {
    /// Cria novo controlador PID
    pub fn new(config: PidConfig) -> Self {
        Self {
            config,
            setpoint: 0.0,
            integral: 0.0,
            prev_error: 0.0,
            filtered_derivative: 0.0,
            last_time: None,
            last_output: 0.0,
        }
    }

    /// Cria com configuração padrão
    pub fn default_config() -> Self {
        Self::new(PidConfig::default())
    }

    /// Define setpoint
    pub fn set_setpoint(&mut self, setpoint: f64) {
        self.setpoint = setpoint;
    }

    /// Retorna setpoint atual
    pub fn setpoint(&self) -> f64 {
        self.setpoint
    }

    /// Retorna configuração
    pub fn config(&self) -> &PidConfig {
        &self.config
    }

    /// Atualiza ganhos
    pub fn set_gains(&mut self, kp: f64, ki: f64, kd: f64) {
        self.config.kp = kp;
        self.config.ki = ki;
        self.config.kd = kd;
    }

    /// Reseta estado interno
    pub fn reset(&mut self) {
        self.integral = 0.0;
        self.prev_error = 0.0;
        self.filtered_derivative = 0.0;
        self.last_time = None;
        self.last_output = 0.0;
    }

    /// Calcula saída do controlador
    ///
    /// # Arguments
    /// * `process_variable` - Valor atual do processo
    ///
    /// # Returns
    /// Sinal de controle
    pub fn update(&mut self, process_variable: f64) -> f64 {
        let now = Instant::now();

        // Calcula dt
        let dt = match self.last_time {
            Some(last) => now.duration_since(last).as_secs_f64(),
            None => {
                self.last_time = Some(now);
                self.prev_error = self.setpoint - process_variable;
                return 0.0;
            }
        };

        // Evita dt muito pequeno
        if dt < 1e-6 {
            return self.last_output;
        }

        // Erro atual
        let error = self.setpoint - process_variable;

        // Termo proporcional
        let p_term = self.config.kp * error;

        // Termo integral com anti-windup
        self.integral += error * dt;
        self.integral = self.integral.clamp(
            -self.config.integral_limit,
            self.config.integral_limit,
        );
        let i_term = self.config.ki * self.integral;

        // Termo derivativo com filtro
        let raw_derivative = (error - self.prev_error) / dt;
        let alpha = self.config.derivative_filter;
        self.filtered_derivative = alpha * raw_derivative
            + (1.0 - alpha) * self.filtered_derivative;
        let d_term = self.config.kd * self.filtered_derivative;

        // Soma e limita
        let output = (p_term + i_term + d_term).clamp(
            self.config.output_min,
            self.config.output_max,
        );

        // Atualiza estado
        self.prev_error = error;
        self.last_time = Some(now);
        self.last_output = output;

        output
    }

    /// Calcula saída com dt explícito (sem usar clock)
    pub fn update_with_dt(&mut self, process_variable: f64, dt: f64) -> f64 {
        if dt < 1e-6 {
            return self.last_output;
        }

        let error = self.setpoint - process_variable;

        // P
        let p_term = self.config.kp * error;

        // I com anti-windup
        self.integral += error * dt;
        self.integral = self.integral.clamp(
            -self.config.integral_limit,
            self.config.integral_limit,
        );
        let i_term = self.config.ki * self.integral;

        // D com filtro
        let raw_derivative = (error - self.prev_error) / dt;
        let alpha = self.config.derivative_filter;
        self.filtered_derivative = alpha * raw_derivative
            + (1.0 - alpha) * self.filtered_derivative;
        let d_term = self.config.kd * self.filtered_derivative;

        let output = (p_term + i_term + d_term).clamp(
            self.config.output_min,
            self.config.output_max,
        );

        self.prev_error = error;
        self.last_output = output;

        output
    }

    /// Retorna componentes individuais para debugging
    pub fn components(&self) -> (f64, f64, f64) {
        let p = self.config.kp * self.prev_error;
        let i = self.config.ki * self.integral;
        let d = self.config.kd * self.filtered_derivative;
        (p, i, d)
    }

    /// Retorna última saída
    pub fn output(&self) -> f64 {
        self.last_output
    }

    /// Retorna erro atual
    pub fn error(&self) -> f64 {
        self.prev_error
    }

    /// Retorna termo integral
    pub fn integral(&self) -> f64 {
        self.integral
    }
}

/// FeedbackLoop trait para controle genérico
pub trait FeedbackLoop {
    /// Tipo do sinal de entrada
    type Input;
    /// Tipo do sinal de saída
    type Output;
    /// Tipo de erro
    type Error;

    /// Define setpoint
    fn set_target(&mut self, target: Self::Input);

    /// Processa feedback e retorna sinal de controle
    fn process(&mut self, feedback: Self::Input) -> Result<Self::Output, Self::Error>;

    /// Reseta estado do loop
    fn reset(&mut self);
}

impl FeedbackLoop for PidController {
    type Input = f64;
    type Output = f64;
    type Error = std::convert::Infallible;

    fn set_target(&mut self, target: f64) {
        self.set_setpoint(target);
    }

    fn process(&mut self, feedback: f64) -> Result<f64, Self::Error> {
        Ok(self.update(feedback))
    }

    fn reset(&mut self) {
        PidController::reset(self);
    }
}

// =============================================================================
// Testes
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pid_creation() {
        let pid = PidController::new(PidConfig::new(1.0, 0.1, 0.01));
        assert_eq!(pid.setpoint(), 0.0);
    }

    #[test]
    fn test_pid_setpoint() {
        let mut pid = PidController::default_config();
        pid.set_setpoint(10.0);
        assert_eq!(pid.setpoint(), 10.0);
    }

    #[test]
    fn test_pid_update_with_dt() {
        let mut pid = PidController::new(PidConfig::new(1.0, 0.0, 0.0)); // P only
        pid.set_setpoint(10.0);

        // Com erro de 10, Kp=1, saída deve ser 10 (mas limitada a 1.0)
        let output = pid.update_with_dt(0.0, 0.1);
        assert!((output - 1.0).abs() < 0.001); // Saturado em output_max
    }

    #[test]
    fn test_pid_convergence() {
        let mut pid = PidController::new(
            PidConfig::new(0.5, 0.0, 0.0)
                .with_output_limits(-100.0, 100.0)
        );
        pid.set_setpoint(10.0);

        let mut pv = 0.0;
        for _ in 0..100 {
            let output = pid.update_with_dt(pv, 0.1);
            pv += output * 0.1; // Simula integração
        }

        // Deve convergir para setpoint
        assert!((pv - 10.0).abs() < 1.0);
    }

    #[test]
    fn test_pid_reset() {
        let mut pid = PidController::default_config();
        pid.set_setpoint(10.0);
        pid.update_with_dt(0.0, 0.1);
        pid.update_with_dt(5.0, 0.1);

        pid.reset();
        assert_eq!(pid.integral(), 0.0);
        assert_eq!(pid.error(), 0.0);
    }

    #[test]
    fn test_anti_windup() {
        let mut pid = PidController::new(
            PidConfig {
                ki: 1.0,
                integral_limit: 5.0,
                ..Default::default()
            }
        );
        pid.set_setpoint(100.0);

        // Muitas iterações com erro grande
        for _ in 0..1000 {
            pid.update_with_dt(0.0, 0.1);
        }

        // Integral deve estar limitada
        assert!(pid.integral().abs() <= 5.0);
    }

    #[test]
    fn test_feedback_loop_trait() {
        let mut pid = PidController::default_config();
        pid.set_target(5.0);

        let result = pid.process(0.0);
        assert!(result.is_ok());
    }
}
