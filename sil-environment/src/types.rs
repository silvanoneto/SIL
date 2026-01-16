//! Tipos de dados ambientais

use serde::{Deserialize, Serialize};

/// Dados ambientais completos capturados por sensores climáticos
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnvironmentData {
    /// Temperatura em graus Celsius
    pub temperature: f32,

    /// Umidade relativa (0.0 - 100.0)
    pub humidity: f32,

    /// Pressão atmosférica em hPa (hectopascal)
    pub pressure: f32,

    /// Qualidade do ar (AQI - Air Quality Index, 0-500)
    pub air_quality: f32,

    /// Concentração de CO2 em ppm (parts per million)
    pub co2_ppm: f32,

    /// Concentração de VOC (Volatile Organic Compounds) em ppb
    pub voc_ppb: f32,

    /// Nível de partículas PM2.5 em µg/m³
    pub pm25: f32,

    /// Nível de partículas PM10 em µg/m³
    pub pm10: f32,
}

impl EnvironmentData {
    /// Cria dados ambientais com valores padrão (ambiente ideal)
    pub fn default_ideal() -> Self {
        Self {
            temperature: 22.0,   // 22°C
            humidity: 50.0,      // 50%
            pressure: 1013.25,   // Pressão ao nível do mar
            air_quality: 50.0,   // Boa qualidade
            co2_ppm: 400.0,      // Normal outdoor
            voc_ppb: 50.0,       // Baixo
            pm25: 10.0,          // Bom
            pm10: 20.0,          // Bom
        }
    }

    /// Calcula score de conforto normalizado (0.0 = péssimo, 1.0 = ideal)
    pub fn comfort_score(&self) -> f32 {
        let temp_score = self.temperature_score();
        let humidity_score = self.humidity_score();
        let air_score = self.air_quality_score();

        (temp_score + humidity_score + air_score) / 3.0
    }

    /// Score de temperatura (ótimo: 20-24°C)
    fn temperature_score(&self) -> f32 {
        let optimal_min = 20.0;
        let optimal_max = 24.0;

        if self.temperature >= optimal_min && self.temperature <= optimal_max {
            1.0
        } else if self.temperature < optimal_min {
            ((self.temperature - 0.0) / optimal_min).clamp(0.0, 1.0)
        } else {
            ((40.0 - self.temperature) / (40.0 - optimal_max)).clamp(0.0, 1.0)
        }
    }

    /// Score de umidade (ótimo: 40-60%)
    fn humidity_score(&self) -> f32 {
        let optimal_min = 40.0;
        let optimal_max = 60.0;

        if self.humidity >= optimal_min && self.humidity <= optimal_max {
            1.0
        } else if self.humidity < optimal_min {
            (self.humidity / optimal_min).clamp(0.0, 1.0)
        } else {
            ((100.0 - self.humidity) / (100.0 - optimal_max)).clamp(0.0, 1.0)
        }
    }

    /// Score de qualidade do ar (ótimo: 0-50 AQI)
    fn air_quality_score(&self) -> f32 {
        if self.air_quality <= 50.0 {
            1.0
        } else if self.air_quality <= 100.0 {
            (100.0 - self.air_quality) / 50.0
        } else {
            ((500.0 - self.air_quality) / 400.0).clamp(0.0, 0.5)
        }
    }

    /// Calcula índice de qualidade do ar baseado em todos os fatores
    pub fn composite_aqi(&self) -> f32 {
        let mut aqi = self.air_quality;

        // CO2 contribui para AQI
        if self.co2_ppm > 1000.0 {
            aqi += (self.co2_ppm - 1000.0) / 10.0;
        }

        // VOC contribui
        if self.voc_ppb > 100.0 {
            aqi += (self.voc_ppb - 100.0) / 5.0;
        }

        // PM2.5 (valores EPA)
        aqi = aqi.max(Self::pm25_to_aqi(self.pm25));

        aqi.clamp(0.0, 500.0)
    }

    /// Converte PM2.5 para AQI (EPA standard)
    pub fn pm25_to_aqi(pm25: f32) -> f32 {
        if pm25 <= 12.0 {
            (50.0 / 12.0) * pm25
        } else if pm25 <= 35.4 {
            50.0 + ((100.0 - 50.0) / (35.4 - 12.0)) * (pm25 - 12.0)
        } else if pm25 <= 55.4 {
            100.0 + ((150.0 - 100.0) / (55.4 - 35.4)) * (pm25 - 35.4)
        } else if pm25 <= 150.4 {
            150.0 + ((200.0 - 150.0) / (150.4 - 55.4)) * (pm25 - 55.4)
        } else {
            200.0 + ((300.0 - 200.0) / (250.4 - 150.4)) * (pm25 - 150.4).min(100.0)
        }
    }

    /// Verifica se algum valor está fora dos limites seguros
    pub fn has_alerts(&self) -> bool {
        self.temperature < 10.0 || self.temperature > 35.0
            || self.humidity > 80.0
            || self.composite_aqi() > 100.0
            || self.co2_ppm > 2000.0
    }

    /// Retorna lista de alertas ativos
    pub fn get_alerts(&self) -> Vec<String> {
        let mut alerts = Vec::new();

        if self.temperature < 10.0 {
            alerts.push(format!("Temperature too low: {:.1}°C", self.temperature));
        } else if self.temperature > 35.0 {
            alerts.push(format!("Temperature too high: {:.1}°C", self.temperature));
        }

        if self.humidity > 80.0 {
            alerts.push(format!("Humidity too high: {:.1}%", self.humidity));
        } else if self.humidity < 20.0 {
            alerts.push(format!("Humidity too low: {:.1}%", self.humidity));
        }

        let aqi = self.composite_aqi();
        if aqi > 150.0 {
            alerts.push(format!("Air quality unhealthy: AQI {:.0}", aqi));
        } else if aqi > 100.0 {
            alerts.push(format!("Air quality moderate: AQI {:.0}", aqi));
        }

        if self.co2_ppm > 2000.0 {
            alerts.push(format!("CO2 level high: {:.0} ppm", self.co2_ppm));
        }

        alerts
    }
}

impl Default for EnvironmentData {
    fn default() -> Self {
        Self::default_ideal()
    }
}

/// Configuração de limites de alerta
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentLimits {
    pub temp_min: f32,
    pub temp_max: f32,
    pub humidity_max: f32,
    pub aqi_max: f32,
    pub co2_max: f32,
}

impl Default for EnvironmentLimits {
    fn default() -> Self {
        Self {
            temp_min: 10.0,
            temp_max: 35.0,
            humidity_max: 80.0,
            aqi_max: 100.0,
            co2_max: 2000.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_data_default() {
        let data = EnvironmentData::default();
        assert_eq!(data.temperature, 22.0);
        assert_eq!(data.humidity, 50.0);
        assert_eq!(data.pressure, 1013.25);
    }

    #[test]
    fn test_comfort_score_ideal() {
        let data = EnvironmentData::default_ideal();
        let score = data.comfort_score();
        assert!(score > 0.9, "Ideal environment should score > 0.9");
    }

    #[test]
    fn test_temperature_score() {
        let mut data = EnvironmentData::default();

        // Ideal temperature
        data.temperature = 22.0;
        assert!(data.temperature_score() == 1.0);

        // Too cold
        data.temperature = 5.0;
        assert!(data.temperature_score() < 0.5);

        // Too hot
        data.temperature = 35.0;
        assert!(data.temperature_score() < 0.5);
    }

    #[test]
    fn test_humidity_score() {
        let mut data = EnvironmentData::default();

        // Ideal humidity
        data.humidity = 50.0;
        assert!(data.humidity_score() == 1.0);

        // Too dry
        data.humidity = 20.0;
        assert!(data.humidity_score() < 1.0);

        // Too humid
        data.humidity = 85.0;
        assert!(data.humidity_score() < 0.5);
    }

    #[test]
    fn test_air_quality_score() {
        let mut data = EnvironmentData::default();

        // Good air quality
        data.air_quality = 25.0;
        assert!(data.air_quality_score() == 1.0);

        // Moderate
        data.air_quality = 75.0;
        assert!(data.air_quality_score() < 1.0);

        // Poor
        data.air_quality = 200.0;
        assert!(data.air_quality_score() <= 0.5);
    }

    #[test]
    fn test_composite_aqi() {
        let mut data = EnvironmentData::default();
        let aqi = data.composite_aqi();
        assert!(aqi < 100.0, "Default should have good AQI");

        // High CO2
        data.co2_ppm = 2000.0;
        let aqi_high_co2 = data.composite_aqi();
        assert!(aqi_high_co2 > aqi);
    }

    #[test]
    fn test_pm25_to_aqi() {
        // Good
        assert!(EnvironmentData::pm25_to_aqi(10.0) < 50.0);
        // Moderate
        assert!(EnvironmentData::pm25_to_aqi(25.0) > 50.0);
        assert!(EnvironmentData::pm25_to_aqi(25.0) < 100.0);
        // Unhealthy
        assert!(EnvironmentData::pm25_to_aqi(60.0) > 150.0);
    }

    #[test]
    fn test_has_alerts() {
        let mut data = EnvironmentData::default();
        assert!(!data.has_alerts(), "Default should have no alerts");

        // Temperature alert
        data.temperature = 40.0;
        assert!(data.has_alerts());

        // Reset and test CO2
        data = EnvironmentData::default();
        data.co2_ppm = 3000.0;
        assert!(data.has_alerts());
    }

    #[test]
    fn test_get_alerts() {
        let mut data = EnvironmentData::default();
        assert!(data.get_alerts().is_empty());

        data.temperature = 40.0;
        data.co2_ppm = 2500.0;
        let alerts = data.get_alerts();
        assert!(alerts.len() >= 2);
        assert!(alerts.iter().any(|a| a.contains("Temperature")));
        assert!(alerts.iter().any(|a| a.contains("CO2")));
    }

    #[test]
    fn test_serialization() {
        let data = EnvironmentData::default();
        let json = serde_json::to_string(&data).unwrap();
        let deserialized: EnvironmentData = serde_json::from_str(&json).unwrap();
        assert_eq!(data, deserialized);
    }

    #[test]
    fn test_environment_limits_default() {
        let limits = EnvironmentLimits::default();
        assert_eq!(limits.temp_min, 10.0);
        assert_eq!(limits.temp_max, 35.0);
    }
}
