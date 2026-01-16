//! Estratégias de merge

use serde::{Deserialize, Serialize};

/// Estratégias de merge de estados
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergeStrategy {
    /// XOR: combina com operação exclusiva
    Xor,
    /// Max: escolhe valor máximo
    Max,
    /// Min: escolhe valor mínimo
    Min,
    /// Average: média aritmética
    Average,
    /// Weighted: média ponderada (favorece destino)
    Weighted { weight: u8 }, // 0-100
}

impl Default for MergeStrategy {
    fn default() -> Self {
        Self::Average
    }
}

impl MergeStrategy {
    /// Aplica estratégia a dois valores
    pub fn apply(&self, a: f32, b: f32) -> f32 {
        match self {
            Self::Xor => {
                // XOR simulado: se sinais diferentes, soma; se iguais, diferença
                if (a >= 0.0) != (b >= 0.0) {
                    a + b
                } else {
                    (a - b).abs()
                }
            }
            Self::Max => a.max(b),
            Self::Min => a.min(b),
            Self::Average => (a + b) / 2.0,
            Self::Weighted { weight } => {
                let w = (*weight as f32) / 100.0;
                a * w + b * (1.0 - w)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_strategy() {
        let strategy = MergeStrategy::Max;
        assert_eq!(strategy.apply(1.0, 5.0), 5.0);
        assert_eq!(strategy.apply(10.0, 3.0), 10.0);
    }

    #[test]
    fn test_min_strategy() {
        let strategy = MergeStrategy::Min;
        assert_eq!(strategy.apply(1.0, 5.0), 1.0);
        assert_eq!(strategy.apply(10.0, 3.0), 3.0);
    }

    #[test]
    fn test_average_strategy() {
        let strategy = MergeStrategy::Average;
        assert_eq!(strategy.apply(0.0, 10.0), 5.0);
        assert_eq!(strategy.apply(2.0, 8.0), 5.0);
    }

    #[test]
    fn test_weighted_strategy() {
        let strategy = MergeStrategy::Weighted { weight: 75 };
        let result = strategy.apply(8.0, 4.0);
        assert!((result - 7.0).abs() < 0.01); // 8*0.75 + 4*0.25 = 7
    }
}
