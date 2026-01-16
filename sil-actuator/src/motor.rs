//! Implementação de motor DC

use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use sil_core::traits::{Actuator, ActuatorStatus, SilComponent, LayerId};
use crate::error::{ActuatorError, ActuatorResult};
use crate::types::{MotorSpeed, MotorDirection};

/// Default status for serde
fn default_status() -> ActuatorStatus {
    ActuatorStatus::Ready
}

/// Estado interno do motor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotorState {
    /// Velocidade atual (%)
    pub current_speed: f32,
    /// Velocidade alvo (%)
    pub target_speed: f32,
    /// Direção atual
    pub direction: MotorDirection,
    /// Status atual (não serializado, use status_str)
    #[serde(skip, default = "default_status")]
    pub status: ActuatorStatus,
    /// Status como string (para serialização)
    #[serde(default)]
    pub status_str: String,
    /// Tempo da última atualização (µs)
    pub last_update_us: u64,
    /// Total de comandos executados
    pub commands: u64,
    /// Tempo total de operação (ms)
    pub runtime_ms: u64,
    /// Calibrado?
    pub calibrated: bool,
    /// Limite de corrente (A)
    pub current_limit_a: f32,
    /// Corrente atual (A) - simulada
    pub current_a: f32,
}

impl MotorState {
    /// Cria novo estado
    pub fn new() -> Self {
        Self {
            current_speed: 0.0,
            target_speed: 0.0,
            direction: MotorDirection::Stopped,
            status: ActuatorStatus::Ready,
            status_str: "Ready".to_string(),
            last_update_us: Self::now_us(),
            commands: 0,
            runtime_ms: 0,
            calibrated: false,
            current_limit_a: 10.0, // 10A default
            current_a: 0.0,
        }
    }

    /// Timestamp atual em microsegundos
    fn now_us() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_micros() as u64)
            .unwrap_or(0)
    }

    /// Reseta o estado
    pub fn reset(&mut self) {
        self.current_speed = 0.0;
        self.target_speed = 0.0;
        self.direction = MotorDirection::Stopped;
        self.status = ActuatorStatus::Ready;
        self.status_str = "Ready".to_string();
        self.commands = 0;
        self.current_a = 0.0;
        self.calibrated = false;
    }

    /// Atualiza velocidade
    pub fn update_speed(&mut self, speed: f32) -> ActuatorResult<()> {
        if speed < -100.0 || speed > 100.0 {
            return Err(ActuatorError::OutOfRange(format!(
                "Speed {}% outside limits (-100% to 100%)",
                speed
            )));
        }

        let now = Self::now_us();
        let delta_us = now - self.last_update_us;

        // Atualizar tempo de operação se estava rodando
        if self.current_speed.abs() > 0.1 {
            self.runtime_ms += (delta_us / 1000) as u64;
        }

        self.target_speed = speed;
        self.current_speed = speed; // Mock: mudança instantânea
        self.direction = if speed > 0.0 {
            MotorDirection::Forward
        } else if speed < 0.0 {
            MotorDirection::Reverse
        } else {
            MotorDirection::Stopped
        };

        // Simular corrente proporcional à velocidade
        self.current_a = (speed.abs() / 100.0) * 5.0; // Max 5A em full speed

        // Verificar limite de corrente
        if self.current_a > self.current_limit_a {
            self.status = ActuatorStatus::Fault;
            return Err(ActuatorError::Fault(format!(
                "Current limit exceeded: {:.2}A > {:.2}A",
                self.current_a, self.current_limit_a
            )));
        }

        self.last_update_us = now;
        self.commands += 1;
        Ok(())
    }

    /// Verifica se está acelerando
    pub fn is_accelerating(&self) -> bool {
        (self.current_speed.abs() - self.target_speed.abs()).abs() > 0.1
    }

    /// Para o motor
    pub fn stop(&mut self) -> ActuatorResult<()> {
        self.update_speed(0.0)
    }
}

impl Default for MotorState {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuração do motor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotorConfig {
    /// Nome do motor
    pub name: String,
    /// ID do motor (para hardware)
    pub motor_id: u8,
    /// Limite de corrente (A)
    pub current_limit_a: f32,
    /// Taxa de aceleração (% por segundo)
    pub acceleration_rate: f32,
    /// Inverter direção?
    pub invert_direction: bool,
}

impl Default for MotorConfig {
    fn default() -> Self {
        Self {
            name: "motor".to_string(),
            motor_id: 0,
            current_limit_a: 10.0,
            acceleration_rate: 100.0, // 100%/s (instantâneo em mock)
            invert_direction: false,
        }
    }
}

/// Motor DC concreto (L6)
#[derive(Clone)]
pub struct MotorActuator {
    /// Estado interno
    state: Arc<Mutex<MotorState>>,
    /// Configuração
    config: MotorConfig,
    /// ID único
    id: u128,
}

impl std::fmt::Debug for MotorActuator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MotorActuator")
            .field("id", &self.id)
            .field("config", &self.config)
            .finish()
    }
}

impl MotorActuator {
    /// Cria novo motor
    pub fn new() -> ActuatorResult<Self> {
        Self::with_config(MotorConfig::default())
    }

    /// Cria com configuração específica
    pub fn with_config(config: MotorConfig) -> ActuatorResult<Self> {
        if config.current_limit_a <= 0.0 {
            return Err(ActuatorError::InvalidConfig(
                "current_limit_a must be positive".into(),
            ));
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let id = (timestamp as u128) ^ ((config.motor_id as u128) << 32) ^ 0xFF;

        let mut state = MotorState::new();
        state.current_limit_a = config.current_limit_a;

        Ok(Self {
            state: Arc::new(Mutex::new(state)),
            config,
            id,
        })
    }

    /// Cria motor com nome e ID
    pub fn named(name: &str, motor_id: u8) -> ActuatorResult<Self> {
        let mut config = MotorConfig::default();
        config.name = name.to_string();
        config.motor_id = motor_id;
        Self::with_config(config)
    }

    /// Retorna estado interno
    pub fn state(&self) -> ActuatorResult<MotorState> {
        let state = self.state.lock().unwrap();
        Ok(state.clone())
    }

    /// Retorna velocidade atual
    pub fn speed(&self) -> ActuatorResult<f32> {
        let state = self.state.lock().unwrap();
        Ok(state.current_speed)
    }

    /// Retorna direção atual
    pub fn direction(&self) -> MotorDirection {
        let state = self.state.lock().unwrap();
        state.direction
    }

    /// Define velocidade
    pub fn set_speed(&mut self, speed: f32) -> ActuatorResult<()> {
        let mut state = self.state.lock().unwrap();

        if state.status == ActuatorStatus::Fault {
            return Err(ActuatorError::Fault("Motor in fault state".into()));
        }

        if state.status == ActuatorStatus::Busy {
            return Err(ActuatorError::Busy);
        }

        let speed = if self.config.invert_direction {
            -speed
        } else {
            speed
        };

        state.status = ActuatorStatus::Busy;
        let result = state.update_speed(speed);
        state.status = if result.is_ok() {
            ActuatorStatus::Ready
        } else {
            ActuatorStatus::Fault
        };

        result
    }

    /// Define velocidade usando MotorSpeed
    pub fn set_motor_speed(&mut self, speed: MotorSpeed) -> ActuatorResult<()> {
        speed.validate()?;
        self.set_speed(speed.percent)
    }

    /// Para o motor
    pub fn stop(&mut self) -> ActuatorResult<()> {
        let mut state = self.state.lock().unwrap();
        state.stop()
    }

    /// Retorna configuração
    pub fn config(&self) -> &MotorConfig {
        &self.config
    }

    /// Verifica se está calibrado
    pub fn is_calibrated(&self) -> bool {
        let state = self.state.lock().unwrap();
        state.calibrated
    }

    /// Número de comandos executados
    pub fn command_count(&self) -> u64 {
        let state = self.state.lock().unwrap();
        state.commands
    }

    /// Tempo total de operação (ms)
    pub fn runtime_ms(&self) -> u64 {
        let state = self.state.lock().unwrap();
        state.runtime_ms
    }

    /// Corrente atual (A)
    pub fn current_a(&self) -> f32 {
        let state = self.state.lock().unwrap();
        state.current_a
    }
}

impl Default for MotorActuator {
    fn default() -> Self {
        Self::new().expect("Failed to create MotorActuator")
    }
}

/// Implementação do trait Actuator
impl Actuator for MotorActuator {
    type Command = MotorSpeed;

    fn send(&mut self, cmd: Self::Command) -> Result<(), sil_core::traits::ActuatorError> {
        self.set_motor_speed(cmd)
            .map_err(|e| e.into())
    }

    fn status(&self) -> ActuatorStatus {
        let state = self.state.lock().unwrap();
        state.status
    }

    fn emergency_stop(&mut self) -> Result<(), sil_core::traits::ActuatorError> {
        let mut state = self.state.lock().unwrap();
        let _ = state.stop();  // Ignore error, just stop
        state.status = ActuatorStatus::Off;
        Ok(())
    }

    fn reset(&mut self) -> Result<(), sil_core::traits::ActuatorError> {
        let mut state = self.state.lock().unwrap();
        state.reset();
        Ok(())
    }
}

/// Implementação do trait SilComponent
impl SilComponent for MotorActuator {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn layers(&self) -> &[LayerId] {
        &[6] // L6 - Psicomotor/Atuador
    }

    fn version(&self) -> &str {
        "2026.1.12"
    }

    fn is_ready(&self) -> bool {
        let state = self.state.lock().unwrap();
        state.status == ActuatorStatus::Ready
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_motor_state_new() {
        let state = MotorState::new();
        assert_eq!(state.current_speed, 0.0);
        assert_eq!(state.direction, MotorDirection::Stopped);
        assert_eq!(state.status, ActuatorStatus::Ready);
    }

    #[test]
    fn test_motor_state_update() {
        let mut state = MotorState::new();
        assert!(state.update_speed(50.0).is_ok());
        assert_eq!(state.current_speed, 50.0);
        assert_eq!(state.direction, MotorDirection::Forward);
        assert_eq!(state.commands, 1);
    }

    #[test]
    fn test_motor_state_reverse() {
        let mut state = MotorState::new();
        assert!(state.update_speed(-75.0).is_ok());
        assert_eq!(state.current_speed, -75.0);
        assert_eq!(state.direction, MotorDirection::Reverse);
    }

    #[test]
    fn test_motor_state_out_of_range() {
        let mut state = MotorState::new();
        assert!(state.update_speed(-150.0).is_err());
        assert!(state.update_speed(150.0).is_err());
    }

    #[test]
    fn test_motor_state_stop() {
        let mut state = MotorState::new();
        state.update_speed(50.0).unwrap();
        assert!(state.stop().is_ok());
        assert_eq!(state.current_speed, 0.0);
        assert_eq!(state.direction, MotorDirection::Stopped);
    }

    #[test]
    fn test_motor_actuator_new() {
        let motor = MotorActuator::new();
        assert!(motor.is_ok());
    }

    #[test]
    fn test_motor_actuator_named() {
        let motor = MotorActuator::named("test-motor", 2).unwrap();
        assert_eq!(motor.name(), "test-motor");
        assert_eq!(motor.config().motor_id, 2);
    }

    #[test]
    fn test_motor_actuator_set_speed() {
        let mut motor = MotorActuator::new().unwrap();
        assert!(motor.set_speed(75.0).is_ok());
        assert_eq!(motor.speed().unwrap(), 75.0);
        assert_eq!(motor.direction(), MotorDirection::Forward);
    }

    #[test]
    fn test_motor_actuator_set_motor_speed() {
        let mut motor = MotorActuator::new().unwrap();
        let speed = MotorSpeed::new(-50.0).unwrap();
        assert!(motor.set_motor_speed(speed).is_ok());
        assert_eq!(motor.speed().unwrap(), -50.0);
        assert_eq!(motor.direction(), MotorDirection::Reverse);
    }

    #[test]
    fn test_motor_actuator_stop() {
        let mut motor = MotorActuator::new().unwrap();
        motor.set_speed(100.0).unwrap();
        assert!(motor.stop().is_ok());
        assert_eq!(motor.speed().unwrap(), 0.0);
    }

    #[test]
    fn test_motor_actuator_trait() {
        let mut motor = MotorActuator::new().unwrap();
        let speed = MotorSpeed::new(60.0).unwrap();

        assert!(motor.send(speed).is_ok());
        assert_eq!(motor.status(), ActuatorStatus::Ready);
    }

    #[test]
    fn test_motor_emergency_stop() {
        let mut motor = MotorActuator::new().unwrap();
        motor.set_speed(100.0).unwrap();
        assert!(motor.emergency_stop().is_ok());
        assert_eq!(motor.speed().unwrap(), 0.0);
        assert_eq!(motor.status(), ActuatorStatus::Off);
    }

    #[test]
    fn test_motor_reset() {
        let mut motor = MotorActuator::new().unwrap();
        motor.set_speed(50.0).unwrap();
        assert!(motor.reset().is_ok());
        assert_eq!(motor.status(), ActuatorStatus::Ready);
        assert_eq!(motor.speed().unwrap(), 0.0);
    }

    #[test]
    fn test_motor_component_trait() {
        let motor = MotorActuator::named("my-motor", 7).unwrap();
        assert_eq!(motor.name(), "my-motor");
        assert_eq!(motor.layers(), &[6]);
        assert_eq!(motor.version(), "2026.1.12");
        assert!(motor.is_ready());
    }

    #[test]
    fn test_motor_command_count() {
        let mut motor = MotorActuator::new().unwrap();
        assert_eq!(motor.command_count(), 0);

        motor.set_speed(50.0).unwrap();
        assert_eq!(motor.command_count(), 1);

        motor.set_speed(-75.0).unwrap();
        assert_eq!(motor.command_count(), 2);
    }

    #[test]
    fn test_motor_invert_direction() {
        let config = MotorConfig {
            invert_direction: true,
            ..Default::default()
        };
        let mut motor = MotorActuator::with_config(config).unwrap();

        motor.set_speed(50.0).unwrap();
        assert_eq!(motor.speed().unwrap(), -50.0); // Invertido
    }

    #[test]
    fn test_motor_current_simulation() {
        let mut motor = MotorActuator::new().unwrap();

        motor.set_speed(0.0).unwrap();
        assert_eq!(motor.current_a(), 0.0);

        motor.set_speed(100.0).unwrap();
        assert!(motor.current_a() > 0.0);

        motor.set_speed(-100.0).unwrap();
        assert!(motor.current_a() > 0.0); // Corrente é sempre positiva
    }

    #[test]
    fn test_motor_current_limit() {
        let config = MotorConfig {
            current_limit_a: 2.0, // Limite baixo
            ..Default::default()
        };
        let mut motor = MotorActuator::with_config(config).unwrap();

        // Full speed deve exceder limite de 2A (simulado: 5A em 100%)
        let result = motor.set_speed(100.0);
        assert!(result.is_err());
        assert_eq!(motor.status(), ActuatorStatus::Fault);
    }

    #[test]
    fn test_motor_config_invalid() {
        let config = MotorConfig {
            current_limit_a: -1.0, // Invalid: negative current
            ..Default::default()
        };
        assert!(MotorActuator::with_config(config).is_err());
    }
}
