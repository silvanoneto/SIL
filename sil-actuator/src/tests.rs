//! Integration tests for sil-actuator

use crate::*;
use sil_core::traits::{Actuator, ActuatorStatus, SilComponent};

// ═══════════════════════════════════════════════════════════════════════════
// SERVO INTEGRATION TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_servo_full_lifecycle() {
    let mut servo = ServoActuator::new().unwrap();

    // Initial state
    assert_eq!(servo.position().unwrap(), 90.0);
    assert_eq!(servo.status(), ActuatorStatus::Ready);

    // Move to various positions
    servo.move_to(0.0).unwrap();
    assert_eq!(servo.position().unwrap(), 0.0);

    servo.move_to(180.0).unwrap();
    assert_eq!(servo.position().unwrap(), 180.0);

    servo.move_to(90.0).unwrap();
    assert_eq!(servo.position().unwrap(), 90.0);

    // Emergency stop
    servo.emergency_stop().unwrap();
    assert_eq!(servo.status(), ActuatorStatus::Off);

    // Reset
    servo.reset().unwrap();
    assert_eq!(servo.status(), ActuatorStatus::Ready);
    assert_eq!(servo.position().unwrap(), 90.0);
}

#[test]
fn test_servo_with_actuator_trait() {
    let mut servo = ServoActuator::named("test-servo", 1).unwrap();

    // Use Actuator trait methods
    let pos = ServoPosition::new(45.0).unwrap();
    assert!(servo.send(pos).is_ok());
    assert_eq!(servo.status(), ActuatorStatus::Ready);

    let pos2 = ServoPosition::new(135.0).unwrap();
    assert!(servo.send(pos2).is_ok());
}

#[test]
fn test_servo_sil_component_trait() {
    let servo = ServoActuator::named("gripper", 3).unwrap();

    assert_eq!(servo.name(), "gripper");
    assert_eq!(servo.layers(), &[6]);
    assert!(servo.version().contains("2026"));
    assert!(servo.is_ready());
}

#[test]
fn test_servo_custom_limits() {
    let config = ServoConfig {
        name: "limited-servo".into(),
        servo_id: 5,
        min_angle: 45.0,
        max_angle: 135.0,
        speed_deg_per_sec: 60.0,
    };

    let mut servo = ServoActuator::with_config(config).unwrap();

    // Within limits
    assert!(servo.move_to(90.0).is_ok());

    // Outside limits
    assert!(servo.move_to(30.0).is_err());
    assert!(servo.move_to(150.0).is_err());
}

#[test]
fn test_servo_movement_counter() {
    let mut servo = ServoActuator::new().unwrap();
    assert_eq!(servo.movement_count(), 0);

    for i in 0..10 {
        let angle = (i as f32 * 18.0) % 180.0;
        servo.move_to(angle).unwrap();
    }

    assert_eq!(servo.movement_count(), 10);
}

#[test]
fn test_servo_pulse_width_conversion() {
    let pos_0 = ServoPosition::new(0.0).unwrap();
    assert_eq!(pos_0.to_pulse_width_us(), 500);

    let pos_90 = ServoPosition::new(90.0).unwrap();
    assert_eq!(pos_90.to_pulse_width_us(), 1500);

    let pos_180 = ServoPosition::new(180.0).unwrap();
    assert_eq!(pos_180.to_pulse_width_us(), 2500);
}

#[test]
fn test_servo_from_pulse_width() {
    let pos = ServoPosition::from_pulse_width_us(1000).unwrap();
    assert!((pos.angle - 45.0).abs() < 1.0);

    let pos = ServoPosition::from_pulse_width_us(2000).unwrap();
    assert!((pos.angle - 135.0).abs() < 1.0);
}

// ═══════════════════════════════════════════════════════════════════════════
// MOTOR INTEGRATION TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_motor_full_lifecycle() {
    let mut motor = MotorActuator::new().unwrap();

    // Initial state
    assert_eq!(motor.speed().unwrap(), 0.0);
    assert_eq!(motor.direction(), MotorDirection::Stopped);
    assert_eq!(motor.status(), ActuatorStatus::Ready);

    // Forward
    motor.set_speed(75.0).unwrap();
    assert_eq!(motor.speed().unwrap(), 75.0);
    assert_eq!(motor.direction(), MotorDirection::Forward);

    // Reverse
    motor.set_speed(-50.0).unwrap();
    assert_eq!(motor.speed().unwrap(), -50.0);
    assert_eq!(motor.direction(), MotorDirection::Reverse);

    // Stop
    motor.stop().unwrap();
    assert_eq!(motor.speed().unwrap(), 0.0);
    assert_eq!(motor.direction(), MotorDirection::Stopped);
}

#[test]
fn test_motor_with_actuator_trait() {
    let mut motor = MotorActuator::named("test-motor", 2).unwrap();

    // Use Actuator trait methods
    let speed = MotorSpeed::new(60.0).unwrap();
    assert!(motor.send(speed).is_ok());
    assert_eq!(motor.status(), ActuatorStatus::Ready);

    let speed2 = MotorSpeed::new(-40.0).unwrap();
    assert!(motor.send(speed2).is_ok());
}

#[test]
fn test_motor_sil_component_trait() {
    let motor = MotorActuator::named("wheel-left", 4).unwrap();

    assert_eq!(motor.name(), "wheel-left");
    assert_eq!(motor.layers(), &[6]);
    assert!(motor.version().contains("2026"));
    assert!(motor.is_ready());
}

#[test]
fn test_motor_direction_detection() {
    let speed_forward = MotorSpeed::new(50.0).unwrap();
    assert_eq!(speed_forward.direction(), MotorDirection::Forward);

    let speed_reverse = MotorSpeed::new(-50.0).unwrap();
    assert_eq!(speed_reverse.direction(), MotorDirection::Reverse);

    let speed_stop = MotorSpeed::new(0.0).unwrap();
    assert_eq!(speed_stop.direction(), MotorDirection::Stopped);
}

#[test]
fn test_motor_pwm_conversion() {
    let speed_full = MotorSpeed::new(100.0).unwrap();
    assert_eq!(speed_full.to_pwm_u8(), 255);

    let speed_half = MotorSpeed::new(50.0).unwrap();
    assert!((speed_half.to_pwm_u8() as f32 - 127.5).abs() < 1.0);

    let speed_zero = MotorSpeed::new(0.0).unwrap();
    assert_eq!(speed_zero.to_pwm_u8(), 0);
}

#[test]
fn test_motor_pwm_12bit_conversion() {
    let speed = MotorSpeed::new(100.0).unwrap();
    assert_eq!(speed.to_pwm_u12(), 4095);

    let speed = MotorSpeed::new(50.0).unwrap();
    assert!((speed.to_pwm_u12() as f32 - 2047.5).abs() < 1.0);

    let speed = MotorSpeed::new(0.0).unwrap();
    assert_eq!(speed.to_pwm_u12(), 0);
}

#[test]
fn test_motor_invert_direction() {
    let config = MotorConfig {
        name: "inverted-motor".into(),
        motor_id: 10,
        invert_direction: true,
        ..Default::default()
    };

    let mut motor = MotorActuator::with_config(config).unwrap();

    // Set positive speed, expect negative
    motor.set_speed(50.0).unwrap();
    assert_eq!(motor.speed().unwrap(), -50.0);

    // Set negative speed, expect positive
    motor.set_speed(-30.0).unwrap();
    assert_eq!(motor.speed().unwrap(), 30.0);
}

#[test]
fn test_motor_current_simulation() {
    let mut motor = MotorActuator::new().unwrap();

    motor.set_speed(0.0).unwrap();
    assert_eq!(motor.current_a(), 0.0);

    motor.set_speed(50.0).unwrap();
    let current_50 = motor.current_a();
    assert!(current_50 > 0.0);

    motor.set_speed(100.0).unwrap();
    let current_100 = motor.current_a();
    assert!(current_100 > current_50);

    // Negative speed should have same current as positive
    motor.set_speed(-100.0).unwrap();
    assert_eq!(motor.current_a(), current_100);
}

#[test]
fn test_motor_current_limit_protection() {
    let config = MotorConfig {
        name: "limited-motor".into(),
        motor_id: 15,
        current_limit_a: 2.0,
        ..Default::default()
    };

    let mut motor = MotorActuator::with_config(config).unwrap();

    // Low speed should work
    assert!(motor.set_speed(30.0).is_ok());
    assert_eq!(motor.status(), ActuatorStatus::Ready);

    // High speed should trigger fault
    let result = motor.set_speed(100.0);
    assert!(result.is_err());
    assert_eq!(motor.status(), ActuatorStatus::Fault);
}

#[test]
fn test_motor_command_counter() {
    let mut motor = MotorActuator::new().unwrap();
    assert_eq!(motor.command_count(), 0);

    for i in 1..=5 {
        motor.set_speed((i as f32) * 10.0).unwrap();
    }

    assert_eq!(motor.command_count(), 5);
}

// ═══════════════════════════════════════════════════════════════════════════
// ACTUATOR COMMAND TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_actuator_command_servo() {
    let cmd = ActuatorCommand::servo(90.0).unwrap();
    assert!(matches!(cmd, ActuatorCommand::Servo(_)));

    match cmd {
        ActuatorCommand::Servo(pos) => assert_eq!(pos.angle, 90.0),
        _ => panic!("Expected Servo command"),
    }
}

#[test]
fn test_actuator_command_motor() {
    let cmd = ActuatorCommand::motor(75.0).unwrap();
    assert!(matches!(cmd, ActuatorCommand::Motor(_)));

    match cmd {
        ActuatorCommand::Motor(speed) => assert_eq!(speed.percent, 75.0),
        _ => panic!("Expected Motor command"),
    }
}

#[test]
fn test_actuator_command_emergency() {
    let cmd = ActuatorCommand::EmergencyStop;
    assert!(cmd.is_emergency());
    assert!(!cmd.is_reset());
    assert!(!cmd.is_calibrate());
}

#[test]
fn test_actuator_command_reset() {
    let cmd = ActuatorCommand::Reset;
    assert!(!cmd.is_emergency());
    assert!(cmd.is_reset());
    assert!(!cmd.is_calibrate());
}

#[test]
fn test_actuator_command_calibrate() {
    let cmd = ActuatorCommand::Calibrate;
    assert!(!cmd.is_emergency());
    assert!(!cmd.is_reset());
    assert!(cmd.is_calibrate());
}

// ═══════════════════════════════════════════════════════════════════════════
// ERROR HANDLING TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_servo_error_out_of_range() {
    let result = ServoPosition::new(-10.0);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ActuatorError::OutOfRange(_)));

    let result = ServoPosition::new(200.0);
    assert!(result.is_err());
}

#[test]
fn test_motor_error_out_of_range() {
    let result = MotorSpeed::new(-150.0);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ActuatorError::OutOfRange(_)));

    let result = MotorSpeed::new(150.0);
    assert!(result.is_err());
}

#[test]
fn test_error_conversion_to_core() {
    let err = ActuatorError::CommandFailed("test".into());
    let core_err: sil_core::traits::ActuatorError = err.into();
    assert!(core_err.to_string().contains("Command failed"));
}

#[test]
fn test_error_conversion_from_core() {
    let core_err = sil_core::traits::ActuatorError::Busy;
    let err: ActuatorError = core_err.into();
    assert_eq!(err, ActuatorError::Busy);
}

// ═══════════════════════════════════════════════════════════════════════════
// CONCURRENCY TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_servo_clone_and_share() {
    let servo1 = ServoActuator::new().unwrap();
    let mut servo2 = servo1.clone();

    // Both servos share the same internal state via Arc
    servo2.move_to(45.0).unwrap();
    assert_eq!(servo1.position().unwrap(), 45.0);
}

#[test]
fn test_motor_clone_and_share() {
    let motor1 = MotorActuator::new().unwrap();
    let mut motor2 = motor1.clone();

    // Both motors share the same internal state via Arc
    motor2.set_speed(60.0).unwrap();
    assert_eq!(motor1.speed().unwrap(), 60.0);
}

#[test]
fn test_servo_thread_safety() {
    use std::thread;

    let servo = ServoActuator::new().unwrap();
    let mut servo_clone = servo.clone();

    let handle = thread::spawn(move || {
        servo_clone.move_to(135.0).unwrap();
    });

    handle.join().unwrap();
    assert_eq!(servo.position().unwrap(), 135.0);
}

#[test]
fn test_motor_thread_safety() {
    use std::thread;

    let motor = MotorActuator::new().unwrap();
    let mut motor_clone = motor.clone();

    let handle = thread::spawn(move || {
        motor_clone.set_speed(80.0).unwrap();
    });

    handle.join().unwrap();
    assert_eq!(motor.speed().unwrap(), 80.0);
}

// ═══════════════════════════════════════════════════════════════════════════
// PRESET VALUES TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_servo_presets() {
    assert_eq!(ServoPosition::min().angle, 0.0);
    assert_eq!(ServoPosition::max().angle, 180.0);
    assert_eq!(ServoPosition::center().angle, 90.0);
}

#[test]
fn test_motor_presets() {
    assert_eq!(MotorSpeed::stop().percent, 0.0);
    assert_eq!(MotorSpeed::full_forward().percent, 100.0);
    assert_eq!(MotorSpeed::full_reverse().percent, -100.0);
    assert_eq!(MotorSpeed::half_forward().percent, 50.0);
    assert_eq!(MotorSpeed::half_reverse().percent, -50.0);
}

// ═══════════════════════════════════════════════════════════════════════════
// VALIDATION TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_servo_position_validation() {
    let pos = ServoPosition::new(90.0).unwrap();
    assert!(pos.validate().is_ok());

    // Create invalid position by directly setting angle (for testing)
    let mut pos = ServoPosition::new(90.0).unwrap();
    pos.angle = 200.0;
    assert!(pos.validate().is_err());
}

#[test]
fn test_motor_speed_validation() {
    let speed = MotorSpeed::new(75.0).unwrap();
    assert!(speed.validate().is_ok());

    // Create invalid speed by directly setting percent (for testing)
    let mut speed = MotorSpeed::new(75.0).unwrap();
    speed.percent = 150.0;
    assert!(speed.validate().is_err());
}

#[test]
fn test_motor_abs_speed() {
    let speed = MotorSpeed::new(75.0).unwrap();
    assert_eq!(speed.abs_speed(), 75.0);

    let speed = MotorSpeed::new(-60.0).unwrap();
    assert_eq!(speed.abs_speed(), 60.0);
}
