//! Tipos de dados para atuadores

use serde::{Deserialize, Serialize};
use crate::error::{ActuatorError, ActuatorResult};

/// Posição de servo (ângulo em graus)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ServoPosition {
    /// Ângulo em graus (0.0 a 180.0)
    pub angle: f32,
}

impl ServoPosition {
    /// Cria nova posição de servo
    pub fn new(angle: f32) -> ActuatorResult<Self> {
        if angle < 0.0 || angle > 180.0 {
            return Err(ActuatorError::OutOfRange(format!(
                "Servo angle must be 0-180°, got {}°",
                angle
            )));
        }
        Ok(Self { angle })
    }

    /// Posição mínima (0°)
    pub fn min() -> Self {
        Self { angle: 0.0 }
    }

    /// Posição máxima (180°)
    pub fn max() -> Self {
        Self { angle: 180.0 }
    }

    /// Posição central (90°)
    pub fn center() -> Self {
        Self { angle: 90.0 }
    }

    /// Valida se a posição está no range válido
    pub fn validate(&self) -> ActuatorResult<()> {
        if self.angle < 0.0 || self.angle > 180.0 {
            return Err(ActuatorError::OutOfRange(format!(
                "Invalid servo angle: {}°",
                self.angle
            )));
        }
        Ok(())
    }

    /// Converte para largura de pulso (µs) padrão
    /// 0° = 500µs, 180° = 2500µs
    pub fn to_pulse_width_us(&self) -> u16 {
        (500.0 + (self.angle / 180.0) * 2000.0) as u16
    }

    /// Cria a partir de largura de pulso (µs)
    pub fn from_pulse_width_us(pulse_us: u16) -> ActuatorResult<Self> {
        if pulse_us < 500 || pulse_us > 2500 {
            return Err(ActuatorError::OutOfRange(format!(
                "Pulse width must be 500-2500µs, got {}µs",
                pulse_us
            )));
        }
        let angle = ((pulse_us as f32 - 500.0) / 2000.0) * 180.0;
        Ok(Self { angle })
    }
}

/// Velocidade de motor (PWM duty cycle)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MotorSpeed {
    /// Velocidade como porcentagem (-100.0 a 100.0)
    /// Negativo = reverso, positivo = avante, 0 = parado
    pub percent: f32,
}

impl MotorSpeed {
    /// Cria nova velocidade de motor
    pub fn new(percent: f32) -> ActuatorResult<Self> {
        if percent < -100.0 || percent > 100.0 {
            return Err(ActuatorError::OutOfRange(format!(
                "Motor speed must be -100% to 100%, got {}%",
                percent
            )));
        }
        Ok(Self { percent })
    }

    /// Motor parado
    pub fn stop() -> Self {
        Self { percent: 0.0 }
    }

    /// Velocidade máxima avante
    pub fn full_forward() -> Self {
        Self { percent: 100.0 }
    }

    /// Velocidade máxima reverso
    pub fn full_reverse() -> Self {
        Self { percent: -100.0 }
    }

    /// Meia velocidade avante
    pub fn half_forward() -> Self {
        Self { percent: 50.0 }
    }

    /// Meia velocidade reverso
    pub fn half_reverse() -> Self {
        Self { percent: -50.0 }
    }

    /// Valida se a velocidade está no range válido
    pub fn validate(&self) -> ActuatorResult<()> {
        if self.percent < -100.0 || self.percent > 100.0 {
            return Err(ActuatorError::OutOfRange(format!(
                "Invalid motor speed: {}%",
                self.percent
            )));
        }
        Ok(())
    }

    /// Retorna direção do motor
    pub fn direction(&self) -> MotorDirection {
        if self.percent > 0.0 {
            MotorDirection::Forward
        } else if self.percent < 0.0 {
            MotorDirection::Reverse
        } else {
            MotorDirection::Stopped
        }
    }

    /// Retorna velocidade absoluta (0-100%)
    pub fn abs_speed(&self) -> f32 {
        self.percent.abs()
    }

    /// Converte para valor PWM (0-255)
    pub fn to_pwm_u8(&self) -> u8 {
        ((self.percent.abs() / 100.0) * 255.0) as u8
    }

    /// Converte para valor PWM (0-4095, 12-bit)
    pub fn to_pwm_u12(&self) -> u16 {
        ((self.percent.abs() / 100.0) * 4095.0) as u16
    }
}

/// Direção do motor
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MotorDirection {
    Forward,
    Reverse,
    Stopped,
}

/// Comando genérico para atuadores
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ActuatorCommand {
    /// Comando de servo
    Servo(ServoPosition),
    /// Comando de motor
    Motor(MotorSpeed),
    /// Parada de emergência
    EmergencyStop,
    /// Reset do atuador
    Reset,
    /// Calibração
    Calibrate,
}

impl ActuatorCommand {
    /// Cria comando de servo
    pub fn servo(angle: f32) -> ActuatorResult<Self> {
        Ok(ActuatorCommand::Servo(ServoPosition::new(angle)?))
    }

    /// Cria comando de motor
    pub fn motor(percent: f32) -> ActuatorResult<Self> {
        Ok(ActuatorCommand::Motor(MotorSpeed::new(percent)?))
    }

    /// Verifica se é comando de emergência
    pub fn is_emergency(&self) -> bool {
        matches!(self, ActuatorCommand::EmergencyStop)
    }

    /// Verifica se é comando de reset
    pub fn is_reset(&self) -> bool {
        matches!(self, ActuatorCommand::Reset)
    }

    /// Verifica se é comando de calibração
    pub fn is_calibrate(&self) -> bool {
        matches!(self, ActuatorCommand::Calibrate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_servo_position_new() {
        let pos = ServoPosition::new(90.0).unwrap();
        assert_eq!(pos.angle, 90.0);
    }

    #[test]
    fn test_servo_position_out_of_range() {
        assert!(ServoPosition::new(-10.0).is_err());
        assert!(ServoPosition::new(200.0).is_err());
    }

    #[test]
    fn test_servo_position_presets() {
        assert_eq!(ServoPosition::min().angle, 0.0);
        assert_eq!(ServoPosition::max().angle, 180.0);
        assert_eq!(ServoPosition::center().angle, 90.0);
    }

    #[test]
    fn test_servo_position_pulse_width() {
        let pos = ServoPosition::new(0.0).unwrap();
        assert_eq!(pos.to_pulse_width_us(), 500);

        let pos = ServoPosition::new(90.0).unwrap();
        assert_eq!(pos.to_pulse_width_us(), 1500);

        let pos = ServoPosition::new(180.0).unwrap();
        assert_eq!(pos.to_pulse_width_us(), 2500);
    }

    #[test]
    fn test_servo_position_from_pulse_width() {
        let pos = ServoPosition::from_pulse_width_us(500).unwrap();
        assert!((pos.angle - 0.0).abs() < 0.1);

        let pos = ServoPosition::from_pulse_width_us(1500).unwrap();
        assert!((pos.angle - 90.0).abs() < 0.1);

        let pos = ServoPosition::from_pulse_width_us(2500).unwrap();
        assert!((pos.angle - 180.0).abs() < 0.1);
    }

    #[test]
    fn test_motor_speed_new() {
        let speed = MotorSpeed::new(50.0).unwrap();
        assert_eq!(speed.percent, 50.0);
    }

    #[test]
    fn test_motor_speed_out_of_range() {
        assert!(MotorSpeed::new(-150.0).is_err());
        assert!(MotorSpeed::new(150.0).is_err());
    }

    #[test]
    fn test_motor_speed_presets() {
        assert_eq!(MotorSpeed::stop().percent, 0.0);
        assert_eq!(MotorSpeed::full_forward().percent, 100.0);
        assert_eq!(MotorSpeed::full_reverse().percent, -100.0);
        assert_eq!(MotorSpeed::half_forward().percent, 50.0);
        assert_eq!(MotorSpeed::half_reverse().percent, -50.0);
    }

    #[test]
    fn test_motor_speed_direction() {
        assert_eq!(MotorSpeed::full_forward().direction(), MotorDirection::Forward);
        assert_eq!(MotorSpeed::full_reverse().direction(), MotorDirection::Reverse);
        assert_eq!(MotorSpeed::stop().direction(), MotorDirection::Stopped);
    }

    #[test]
    fn test_motor_speed_abs() {
        let speed = MotorSpeed::new(-75.0).unwrap();
        assert_eq!(speed.abs_speed(), 75.0);
    }

    #[test]
    fn test_motor_speed_pwm() {
        let speed = MotorSpeed::new(100.0).unwrap();
        assert_eq!(speed.to_pwm_u8(), 255);

        let speed = MotorSpeed::new(50.0).unwrap();
        assert!((speed.to_pwm_u8() as f32 - 127.5).abs() < 1.0);

        let speed = MotorSpeed::new(0.0).unwrap();
        assert_eq!(speed.to_pwm_u8(), 0);
    }

    #[test]
    fn test_motor_speed_pwm_12bit() {
        let speed = MotorSpeed::new(100.0).unwrap();
        assert_eq!(speed.to_pwm_u12(), 4095);

        let speed = MotorSpeed::new(0.0).unwrap();
        assert_eq!(speed.to_pwm_u12(), 0);
    }

    #[test]
    fn test_actuator_command_servo() {
        let cmd = ActuatorCommand::servo(90.0).unwrap();
        match cmd {
            ActuatorCommand::Servo(pos) => assert_eq!(pos.angle, 90.0),
            _ => panic!("Expected servo command"),
        }
    }

    #[test]
    fn test_actuator_command_motor() {
        let cmd = ActuatorCommand::motor(75.0).unwrap();
        match cmd {
            ActuatorCommand::Motor(speed) => assert_eq!(speed.percent, 75.0),
            _ => panic!("Expected motor command"),
        }
    }

    #[test]
    fn test_actuator_command_checks() {
        assert!(ActuatorCommand::EmergencyStop.is_emergency());
        assert!(ActuatorCommand::Reset.is_reset());
        assert!(ActuatorCommand::Calibrate.is_calibrate());
    }
}
