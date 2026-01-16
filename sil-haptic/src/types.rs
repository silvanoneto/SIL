//! Tipos de dados hápticos

use serde::{Deserialize, Serialize};

/// Pressão em Pascal (Pa)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Pressure {
    /// Valor em Pascal
    pub pa: f32,
}

impl Pressure {
    pub fn new(pa: f32) -> Self {
        Self {
            pa: pa.clamp(0.0, 200_000.0), // 0 a 200kPa (aproximadamente 2 atm)
        }
    }

    /// Retorna pressão normalizada [0.0, 1.0]
    pub fn normalized(&self) -> f32 {
        (self.pa / 100_000.0).clamp(0.0, 1.0) // Normaliza para 1 atm = 1.0
    }

    /// Converte para ρ no formato SIL [-8, 7]
    pub fn to_sil_rho(&self) -> i8 {
        let normalized = self.normalized();
        let rho = (normalized * 15.0 - 8.0).round();
        rho.clamp(-8.0, 7.0) as i8
    }

    /// Cria pressão a partir de valor normalizado
    pub fn from_normalized(normalized: f32) -> Self {
        Self::new(normalized.clamp(0.0, 1.0) * 100_000.0)
    }
}

/// Temperatura em Celsius
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Temperature {
    /// Valor em Celsius
    pub celsius: f32,
}

impl Temperature {
    pub fn new(celsius: f32) -> Self {
        Self {
            celsius: celsius.clamp(-40.0, 100.0), // Faixa típica de sensores táteis
        }
    }

    /// Retorna temperatura normalizada [0.0, 1.0] de -40°C a +100°C
    pub fn normalized(&self) -> f32 {
        ((self.celsius + 40.0) / 140.0).clamp(0.0, 1.0)
    }

    /// Converte para θ no formato SIL [0, 255]
    pub fn to_sil_theta(&self) -> u8 {
        (self.normalized() * 255.0).round().clamp(0.0, 255.0) as u8
    }

    /// Cria temperatura a partir de valor normalizado
    pub fn from_normalized(normalized: f32) -> Self {
        let celsius = normalized.clamp(0.0, 1.0) * 140.0 - 40.0;
        Self::new(celsius)
    }

    /// Converte para Fahrenheit
    pub fn to_fahrenheit(&self) -> f32 {
        self.celsius * 9.0 / 5.0 + 32.0
    }

    /// Converte para Kelvin
    pub fn to_kelvin(&self) -> f32 {
        self.celsius + 273.15
    }
}

/// Vibração em Hz
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vibration {
    /// Frequência em Hz
    pub hz: f32,
    /// Amplitude normalizada [0.0, 1.0]
    pub amplitude: f32,
}

impl Vibration {
    pub fn new(hz: f32, amplitude: f32) -> Self {
        Self {
            hz: hz.clamp(0.0, 1000.0), // 0-1000 Hz (faixa típica de toque)
            amplitude: amplitude.clamp(0.0, 1.0),
        }
    }

    /// Retorna se a vibração é perceptível (> 10 Hz e amplitude > 0.05)
    pub fn is_perceptible(&self) -> bool {
        self.hz >= 10.0 && self.amplitude > 0.05
    }

    /// Converte frequência para θ no formato SIL [0, 255]
    pub fn to_sil_theta(&self) -> u8 {
        let normalized = (self.hz / 1000.0).clamp(0.0, 1.0);
        (normalized * 255.0).round() as u8
    }

    /// Converte amplitude para ρ no formato SIL [-8, 7]
    pub fn to_sil_rho(&self) -> i8 {
        let rho = (self.amplitude * 15.0 - 8.0).round();
        rho.clamp(-8.0, 7.0) as i8
    }
}

/// Leitura individual de sensor háptico
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HapticReading {
    pub pressure: Pressure,
    pub temperature: Temperature,
    pub vibration: Vibration,
    /// Timestamp em milissegundos desde a inicialização
    pub timestamp_ms: u64,
}

impl HapticReading {
    pub fn new(pressure: Pressure, temperature: Temperature, vibration: Vibration) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        Self {
            pressure,
            temperature,
            vibration,
            timestamp_ms,
        }
    }

    /// Cria leitura com timestamp específico
    pub fn with_timestamp(
        pressure: Pressure,
        temperature: Temperature,
        vibration: Vibration,
        timestamp_ms: u64,
    ) -> Self {
        Self {
            pressure,
            temperature,
            vibration,
            timestamp_ms,
        }
    }

    /// Cria leitura vazia (sem toque)
    pub fn zero() -> Self {
        Self::new(
            Pressure::new(101_325.0), // Pressão atmosférica padrão
            Temperature::new(20.0),    // Temperatura ambiente
            Vibration::new(0.0, 0.0),  // Sem vibração
        )
    }
}

/// Dados hápticos capturados (múltiplas leituras)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HapticData {
    pub readings: Vec<HapticReading>,
    pub sample_rate: u32,
}

impl HapticData {
    /// Cria dados hápticos vazios
    pub fn empty(sample_rate: u32) -> Self {
        Self {
            readings: Vec::new(),
            sample_rate,
        }
    }

    /// Cria dados com múltiplas leituras
    pub fn new(readings: Vec<HapticReading>, sample_rate: u32) -> Self {
        Self {
            readings,
            sample_rate,
        }
    }

    /// Cria dados com N leituras zero
    pub fn zeros(count: usize, sample_rate: u32) -> Self {
        Self {
            readings: vec![HapticReading::zero(); count],
            sample_rate,
        }
    }

    /// Adiciona leitura
    pub fn push(&mut self, reading: HapticReading) {
        self.readings.push(reading);
    }

    /// Retorna número de leituras
    pub fn len(&self) -> usize {
        self.readings.len()
    }

    /// Verifica se está vazio
    pub fn is_empty(&self) -> bool {
        self.readings.is_empty()
    }

    /// Retorna pressão média
    pub fn avg_pressure(&self) -> Pressure {
        if self.readings.is_empty() {
            return Pressure::new(0.0);
        }
        let sum: f32 = self.readings.iter().map(|r| r.pressure.pa).sum();
        Pressure::new(sum / self.readings.len() as f32)
    }

    /// Retorna temperatura média
    pub fn avg_temperature(&self) -> Temperature {
        if self.readings.is_empty() {
            return Temperature::new(0.0);
        }
        let sum: f32 = self.readings.iter().map(|r| r.temperature.celsius).sum();
        Temperature::new(sum / self.readings.len() as f32)
    }

    /// Retorna vibração média
    pub fn avg_vibration(&self) -> Vibration {
        if self.readings.is_empty() {
            return Vibration::new(0.0, 0.0);
        }
        let hz_sum: f32 = self.readings.iter().map(|r| r.vibration.hz).sum();
        let amp_sum: f32 = self.readings.iter().map(|r| r.vibration.amplitude).sum();
        let count = self.readings.len() as f32;
        Vibration::new(hz_sum / count, amp_sum / count)
    }

    /// Retorna a leitura mais recente
    pub fn latest(&self) -> Option<&HapticReading> {
        self.readings.last()
    }

    /// Retorna duração total em milissegundos
    pub fn duration_ms(&self) -> u64 {
        if self.readings.len() < 2 {
            return 0;
        }
        self.readings.last().unwrap().timestamp_ms - self.readings.first().unwrap().timestamp_ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pressure() {
        let p = Pressure::new(101_325.0); // Pressão atmosférica
        assert_eq!(p.pa, 101_325.0);
        assert!((p.normalized() - 1.0).abs() < 0.02);
        assert_eq!(p.to_sil_rho(), 7);
    }

    #[test]
    fn test_pressure_normalized() {
        let p = Pressure::new(50_000.0); // Meia atmosfera
        assert!((p.normalized() - 0.5).abs() < 0.01);

        let from_norm = Pressure::from_normalized(0.5);
        assert!((from_norm.pa - 50_000.0).abs() < 100.0);
    }

    #[test]
    fn test_temperature() {
        let t = Temperature::new(20.0);
        assert_eq!(t.celsius, 20.0);
        assert!((t.normalized() - 0.428).abs() < 0.01); // (20+40)/140
        assert_eq!(t.to_fahrenheit(), 68.0);
        assert!((t.to_kelvin() - 293.15).abs() < 0.01);
    }

    #[test]
    fn test_temperature_mapping() {
        let cold = Temperature::new(-40.0);
        assert_eq!(cold.to_sil_theta(), 0);

        let hot = Temperature::new(100.0);
        assert_eq!(hot.to_sil_theta(), 255);

        let room = Temperature::new(20.0);
        let theta = room.to_sil_theta();
        assert!(theta > 100 && theta < 120);
    }

    #[test]
    fn test_vibration() {
        let v = Vibration::new(440.0, 0.8);
        assert_eq!(v.hz, 440.0);
        assert_eq!(v.amplitude, 0.8);
        assert!(v.is_perceptible());

        let weak = Vibration::new(5.0, 0.01);
        assert!(!weak.is_perceptible());
    }

    #[test]
    fn test_haptic_reading() {
        let reading = HapticReading::new(
            Pressure::new(100_000.0),
            Temperature::new(25.0),
            Vibration::new(200.0, 0.5),
        );

        assert_eq!(reading.pressure.pa, 100_000.0);
        assert_eq!(reading.temperature.celsius, 25.0);
        assert!(reading.timestamp_ms > 0);
    }

    #[test]
    fn test_haptic_reading_zero() {
        let zero = HapticReading::zero();
        assert_eq!(zero.pressure.pa, 101_325.0); // Atmosférica
        assert_eq!(zero.temperature.celsius, 20.0);
        assert_eq!(zero.vibration.hz, 0.0);
    }

    #[test]
    fn test_haptic_data() {
        let mut data = HapticData::empty(100);
        assert!(data.is_empty());
        assert_eq!(data.len(), 0);

        data.push(HapticReading::zero());
        assert_eq!(data.len(), 1);
        assert!(!data.is_empty());
    }

    #[test]
    fn test_haptic_data_averages() {
        let readings = vec![
            HapticReading::new(
                Pressure::new(100_000.0),
                Temperature::new(20.0),
                Vibration::new(100.0, 0.5),
            ),
            HapticReading::new(
                Pressure::new(120_000.0),
                Temperature::new(30.0),
                Vibration::new(200.0, 0.7),
            ),
        ];

        let data = HapticData::new(readings, 100);

        let avg_p = data.avg_pressure();
        assert!((avg_p.pa - 110_000.0).abs() < 1.0);

        let avg_t = data.avg_temperature();
        assert!((avg_t.celsius - 25.0).abs() < 0.01);

        let avg_v = data.avg_vibration();
        assert!((avg_v.hz - 150.0).abs() < 1.0);
        assert!((avg_v.amplitude - 0.6).abs() < 0.01);
    }
}
