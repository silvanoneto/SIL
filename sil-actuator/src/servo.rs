//! Implementação de servo motor

use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use sil_core::traits::{Actuator, ActuatorStatus, SilComponent, LayerId};
use crate::error::{ActuatorError, ActuatorResult};
use crate::types::ServoPosition;

/// Default status for serde
fn default_status() -> ActuatorStatus {
    ActuatorStatus::Ready
}

/// Estado interno do servo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServoState {
    /// Posição atual (graus)
    pub current_position: f32,
    /// Posição alvo (graus)
    pub target_position: f32,
    /// Status atual (não serializado, use status_str)
    #[serde(skip, default = "default_status")]
    pub status: ActuatorStatus,
    /// Status como string (para serialização)
    #[serde(default)]
    pub status_str: String,
    /// Tempo da última atualização (µs)
    pub last_update_us: u64,
    /// Total de movimentos realizados
    pub movements: u64,
    /// Calibrado?
    pub calibrated: bool,
    /// Limite mínimo (graus)
    pub min_angle: f32,
    /// Limite máximo (graus)
    pub max_angle: f32,
}

impl ServoState {
    /// Cria novo estado
    pub fn new() -> Self {
        Self {
            current_position: 90.0, // Centro
            target_position: 90.0,
            status: ActuatorStatus::Ready,
            status_str: "Ready".to_string(),
            last_update_us: Self::now_us(),
            movements: 0,
            calibrated: false,
            min_angle: 0.0,
            max_angle: 180.0,
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
        self.current_position = 90.0;
        self.target_position = 90.0;
        self.status = ActuatorStatus::Ready;
        self.status_str = "Ready".to_string();
        self.movements = 0;
        self.calibrated = false;
    }

    /// Atualiza posição
    pub fn update_position(&mut self, position: f32) -> ActuatorResult<()> {
        if position < self.min_angle || position > self.max_angle {
            return Err(ActuatorError::OutOfRange(format!(
                "Position {}° outside limits ({}-{}°)",
                position, self.min_angle, self.max_angle
            )));
        }

        self.target_position = position;
        self.current_position = position; // Mock: movimento instantâneo
        self.last_update_us = Self::now_us();
        self.movements += 1;
        Ok(())
    }

    /// Verifica se está em movimento
    pub fn is_moving(&self) -> bool {
        (self.current_position - self.target_position).abs() > 0.1
    }
}

impl Default for ServoState {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuração do servo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServoConfig {
    /// Nome do servo
    pub name: String,
    /// ID do servo (para hardware)
    pub servo_id: u8,
    /// Limite mínimo (graus)
    pub min_angle: f32,
    /// Limite máximo (graus)
    pub max_angle: f32,
    /// Velocidade de movimento (graus/s)
    pub speed_deg_per_sec: f32,
}

impl Default for ServoConfig {
    fn default() -> Self {
        Self {
            name: "servo".to_string(),
            servo_id: 0,
            min_angle: 0.0,
            max_angle: 180.0,
            speed_deg_per_sec: 60.0, // 60°/s
        }
    }
}

/// Servo motor concreto (L6)
#[derive(Clone)]
pub struct ServoActuator {
    /// Estado interno
    state: Arc<Mutex<ServoState>>,
    /// Configuração
    config: ServoConfig,
    /// ID único
    id: u128,
}

impl std::fmt::Debug for ServoActuator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServoActuator")
            .field("id", &self.id)
            .field("config", &self.config)
            .finish()
    }
}

impl ServoActuator {
    /// Cria novo servo
    pub fn new() -> ActuatorResult<Self> {
        Self::with_config(ServoConfig::default())
    }

    /// Cria com configuração específica
    pub fn with_config(config: ServoConfig) -> ActuatorResult<Self> {
        if config.min_angle >= config.max_angle {
            return Err(ActuatorError::InvalidConfig(
                "min_angle must be less than max_angle".into(),
            ));
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let id = (timestamp as u128) ^ ((config.servo_id as u128) << 32);

        let mut state = ServoState::new();
        state.min_angle = config.min_angle;
        state.max_angle = config.max_angle;

        Ok(Self {
            state: Arc::new(Mutex::new(state)),
            config,
            id,
        })
    }

    /// Cria servo com nome e ID
    pub fn named(name: &str, servo_id: u8) -> ActuatorResult<Self> {
        let mut config = ServoConfig::default();
        config.name = name.to_string();
        config.servo_id = servo_id;
        Self::with_config(config)
    }

    /// Retorna estado interno
    pub fn state(&self) -> ActuatorResult<ServoState> {
        let state = self.state.lock().unwrap();
        Ok(state.clone())
    }

    /// Retorna posição atual
    pub fn position(&self) -> ActuatorResult<f32> {
        let state = self.state.lock().unwrap();
        Ok(state.current_position)
    }

    /// Move para posição específica
    pub fn move_to(&mut self, position: f32) -> ActuatorResult<()> {
        let mut state = self.state.lock().unwrap();

        if state.status == ActuatorStatus::Fault {
            return Err(ActuatorError::Fault("Servo in fault state".into()));
        }

        if state.status == ActuatorStatus::Busy {
            return Err(ActuatorError::Busy);
        }

        state.status = ActuatorStatus::Busy;
        let result = state.update_position(position);
        state.status = if result.is_ok() {
            ActuatorStatus::Ready
        } else {
            ActuatorStatus::Fault
        };

        result
    }

    /// Move para posição usando ServoPosition
    pub fn move_to_position(&mut self, position: ServoPosition) -> ActuatorResult<()> {
        position.validate()?;
        self.move_to(position.angle)
    }

    /// Retorna configuração
    pub fn config(&self) -> &ServoConfig {
        &self.config
    }

    /// Verifica se está calibrado
    pub fn is_calibrated(&self) -> bool {
        let state = self.state.lock().unwrap();
        state.calibrated
    }

    /// Número de movimentos realizados
    pub fn movement_count(&self) -> u64 {
        let state = self.state.lock().unwrap();
        state.movements
    }
}

impl Default for ServoActuator {
    fn default() -> Self {
        Self::new().expect("Failed to create ServoActuator")
    }
}

/// Implementação do trait Actuator
impl Actuator for ServoActuator {
    type Command = ServoPosition;

    fn send(&mut self, cmd: Self::Command) -> Result<(), sil_core::traits::ActuatorError> {
        self.move_to_position(cmd)
            .map_err(|e| e.into())
    }

    fn status(&self) -> ActuatorStatus {
        let state = self.state.lock().unwrap();
        state.status
    }

    fn emergency_stop(&mut self) -> Result<(), sil_core::traits::ActuatorError> {
        let mut state = self.state.lock().unwrap();
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
impl SilComponent for ServoActuator {
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
    fn test_servo_state_new() {
        let state = ServoState::new();
        assert_eq!(state.current_position, 90.0);
        assert_eq!(state.target_position, 90.0);
        assert_eq!(state.status, ActuatorStatus::Ready);
    }

    #[test]
    fn test_servo_state_update() {
        let mut state = ServoState::new();
        assert!(state.update_position(45.0).is_ok());
        assert_eq!(state.current_position, 45.0);
        assert_eq!(state.movements, 1);
    }

    #[test]
    fn test_servo_state_out_of_range() {
        let mut state = ServoState::new();
        assert!(state.update_position(-10.0).is_err());
        assert!(state.update_position(200.0).is_err());
    }

    #[test]
    fn test_servo_actuator_new() {
        let servo = ServoActuator::new();
        assert!(servo.is_ok());
    }

    #[test]
    fn test_servo_actuator_named() {
        let servo = ServoActuator::named("test-servo", 1).unwrap();
        assert_eq!(servo.name(), "test-servo");
        assert_eq!(servo.config().servo_id, 1);
    }

    #[test]
    fn test_servo_actuator_move_to() {
        let mut servo = ServoActuator::new().unwrap();
        assert!(servo.move_to(45.0).is_ok());
        assert_eq!(servo.position().unwrap(), 45.0);
    }

    #[test]
    fn test_servo_actuator_move_to_position() {
        let mut servo = ServoActuator::new().unwrap();
        let pos = ServoPosition::new(135.0).unwrap();
        assert!(servo.move_to_position(pos).is_ok());
        assert_eq!(servo.position().unwrap(), 135.0);
    }

    #[test]
    fn test_servo_actuator_trait() {
        let mut servo = ServoActuator::new().unwrap();
        let pos = ServoPosition::new(60.0).unwrap();

        assert!(servo.send(pos).is_ok());
        assert_eq!(servo.status(), ActuatorStatus::Ready);
    }

    #[test]
    fn test_servo_emergency_stop() {
        let mut servo = ServoActuator::new().unwrap();
        assert!(servo.emergency_stop().is_ok());
        assert_eq!(servo.status(), ActuatorStatus::Off);
    }

    #[test]
    fn test_servo_reset() {
        let mut servo = ServoActuator::new().unwrap();
        servo.move_to(45.0).unwrap();
        assert!(servo.reset().is_ok());
        assert_eq!(servo.status(), ActuatorStatus::Ready);
        assert_eq!(servo.position().unwrap(), 90.0);
    }

    #[test]
    fn test_servo_component_trait() {
        let servo = ServoActuator::named("my-servo", 5).unwrap();
        assert_eq!(servo.name(), "my-servo");
        assert_eq!(servo.layers(), &[6]);
        assert_eq!(servo.version(), "2026.1.12");
        assert!(servo.is_ready());
    }

    #[test]
    fn test_servo_movement_count() {
        let mut servo = ServoActuator::new().unwrap();
        assert_eq!(servo.movement_count(), 0);

        servo.move_to(45.0).unwrap();
        assert_eq!(servo.movement_count(), 1);

        servo.move_to(135.0).unwrap();
        assert_eq!(servo.movement_count(), 2);
    }

    #[test]
    fn test_servo_config_invalid() {
        let config = ServoConfig {
            min_angle: 180.0,
            max_angle: 0.0, // Invalid: min > max
            ..Default::default()
        };
        assert!(ServoActuator::with_config(config).is_err());
    }

    #[test]
    fn test_servo_custom_limits() {
        let config = ServoConfig {
            min_angle: 30.0,
            max_angle: 150.0,
            ..Default::default()
        };
        let mut servo = ServoActuator::with_config(config).unwrap();

        // Within limits
        assert!(servo.move_to(90.0).is_ok());

        // Outside limits
        assert!(servo.move_to(20.0).is_err());
        assert!(servo.move_to(160.0).is_err());
    }
}
