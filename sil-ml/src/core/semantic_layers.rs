//! Semantic Layer Classification and Routing
//!
//! 16-layer topology with semantic meaning:
//! - L0-L4:   PERCEPTION (sensory input layers)
//! - L5-L7:   PROCESSING (digital transformation)
//! - L8-LA:   INTERACTION (cross-layer routing)
//! - LB-LC:   EMERGENCE (synergic properties)
//! - LD-LF:   META (reflection and collapse)

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticLayer {
    // PERCEPTION LAYERS (0-4)
    Photonic = 0,      // Visual/light information
    Acoustic = 1,      // Sound/audio information
    Olfactory = 2,     // Smell/chemical information
    Gustatory = 3,     // Taste information
    Dermic = 4,        // Touch/haptic information

    // PROCESSING LAYERS (5-7)
    Electronic = 5,    // Digital signal processing
    Psychomotor = 6,   // Motor control/actuation
    Environmental = 7, // Context/state management

    // INTERACTION LAYERS (8-A)
    Cybernetic = 8,    // Feedback loops
    Geopolitical = 9,  // Social/network effects
    Cosmopolitical = 10, // Environmental scope

    // EMERGENCE LAYERS (B-C)
    Synergic = 11,     // Combined effects
    Quantum = 12,      // Superposition effects

    // META LAYERS (D-F)
    Superposition = 13, // Multiple states
    Entanglement = 14,  // Correlations
    Collapse = 15,      // Final resolution
}

impl SemanticLayer {
    pub fn index(&self) -> usize {
        *self as usize
    }

    pub fn name(&self) -> &'static str {
        match self {
            SemanticLayer::Photonic => "Photonic",
            SemanticLayer::Acoustic => "Acoustic",
            SemanticLayer::Olfactory => "Olfactory",
            SemanticLayer::Gustatory => "Gustatory",
            SemanticLayer::Dermic => "Dermic",
            SemanticLayer::Electronic => "Electronic",
            SemanticLayer::Psychomotor => "Psychomotor",
            SemanticLayer::Environmental => "Environmental",
            SemanticLayer::Cybernetic => "Cybernetic",
            SemanticLayer::Geopolitical => "Geopolitical",
            SemanticLayer::Cosmopolitical => "Cosmopolitical",
            SemanticLayer::Synergic => "Synergic",
            SemanticLayer::Quantum => "Quantum",
            SemanticLayer::Superposition => "Superposition",
            SemanticLayer::Entanglement => "Entanglement",
            SemanticLayer::Collapse => "Collapse",
        }
    }

    pub fn category(&self) -> &'static str {
        match *self {
            SemanticLayer::Photonic
            | SemanticLayer::Acoustic
            | SemanticLayer::Olfactory
            | SemanticLayer::Gustatory
            | SemanticLayer::Dermic => "PERCEPTION",

            SemanticLayer::Electronic
            | SemanticLayer::Psychomotor
            | SemanticLayer::Environmental => "PROCESSING",

            SemanticLayer::Cybernetic
            | SemanticLayer::Geopolitical
            | SemanticLayer::Cosmopolitical => "INTERACTION",

            SemanticLayer::Synergic | SemanticLayer::Quantum => "EMERGENCE",

            SemanticLayer::Superposition
            | SemanticLayer::Entanglement
            | SemanticLayer::Collapse => "META",
        }
    }

    /// Check if layer accepts linear encoding (no transform needed)
    pub fn is_linear_encoding(&self) -> bool {
        // PERCEPTION and PROCESSING use raw linear encoding
        matches!(
            self,
            SemanticLayer::Photonic
                | SemanticLayer::Acoustic
                | SemanticLayer::Olfactory
                | SemanticLayer::Gustatory
                | SemanticLayer::Dermic
                | SemanticLayer::Electronic
                | SemanticLayer::Psychomotor
                | SemanticLayer::Environmental
        )
    }

    /// Get recommended transform for post-encoding semantic processing
    pub fn recommended_transform(&self) -> Option<&'static str> {
        match self {
            // PERCEPTION: No transform needed for raw features
            SemanticLayer::Photonic
            | SemanticLayer::Acoustic
            | SemanticLayer::Olfactory
            | SemanticLayer::Gustatory
            | SemanticLayer::Dermic => None,

            // PROCESSING: Light quantization
            SemanticLayer::Electronic | SemanticLayer::Psychomotor => Some("quantize_4bit"),
            SemanticLayer::Environmental => Some("mix_neutral"),

            // INTERACTION: Blending operations
            SemanticLayer::Cybernetic
            | SemanticLayer::Geopolitical
            | SemanticLayer::Cosmopolitical => Some("mix"),

            // EMERGENCE: Power amplification
            SemanticLayer::Synergic => Some("pow_2"),
            SemanticLayer::Quantum => Some("pow_3"),

            // META: Final transforms
            SemanticLayer::Superposition | SemanticLayer::Entanglement => Some("pow_2"),
            SemanticLayer::Collapse => Some("collapse"),
        }
    }
}

/// Set of semantic layers with quick lookup
#[derive(Debug, Clone)]
pub struct SemanticLayerSet {
    layers: [SemanticLayer; 16],
}

impl SemanticLayerSet {
    pub fn new() -> Self {
        SemanticLayerSet {
            layers: [
                SemanticLayer::Photonic,
                SemanticLayer::Acoustic,
                SemanticLayer::Olfactory,
                SemanticLayer::Gustatory,
                SemanticLayer::Dermic,
                SemanticLayer::Electronic,
                SemanticLayer::Psychomotor,
                SemanticLayer::Environmental,
                SemanticLayer::Cybernetic,
                SemanticLayer::Geopolitical,
                SemanticLayer::Cosmopolitical,
                SemanticLayer::Synergic,
                SemanticLayer::Quantum,
                SemanticLayer::Superposition,
                SemanticLayer::Entanglement,
                SemanticLayer::Collapse,
            ],
        }
    }

    pub fn get(&self, index: usize) -> Option<SemanticLayer> {
        if index < 16 {
            Some(self.layers[index])
        } else {
            None
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = SemanticLayer> + '_ {
        self.layers.iter().copied()
    }

    pub fn by_category(&self, category: &str) -> Vec<SemanticLayer> {
        self.layers
            .iter()
            .filter(|layer| layer.category() == category)
            .copied()
            .collect()
    }
}

impl Default for SemanticLayerSet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_layers() {
        let layers = SemanticLayerSet::new();
        assert_eq!(layers.get(0), Some(SemanticLayer::Photonic));
        assert_eq!(layers.get(15), Some(SemanticLayer::Collapse));
        assert_eq!(layers.get(16), None);
    }

    #[test]
    fn test_layer_categories() {
        let perception = SemanticLayer::Photonic.category();
        assert_eq!(perception, "PERCEPTION");

        let meta = SemanticLayer::Collapse.category();
        assert_eq!(meta, "META");
    }

    #[test]
    fn test_by_category() {
        let layers = SemanticLayerSet::new();
        let perception_layers = layers.by_category("PERCEPTION");
        assert_eq!(perception_layers.len(), 5);
    }
}
