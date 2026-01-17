//! Native Transform Pipeline
//!
//! Post-encoding semantic processing using native _sil_core operations:
//! - pow(n): Non-linear amplification
//! - mul(): Interaction between values
//! - mix(): Blending/interpolation
//! - xor(): Binary operations
//!
//! These transforms are applied AFTER linear encoding for semantic routing.

use sil_core::{ByteSil, SilState};

#[derive(Debug, Clone, Copy)]
pub enum NativeTransform {
    /// Power operation: x^n (in log-polar space)
    Power(u32),

    /// Multiply by another value: x * y
    Multiply(u8),

    /// Blend with neutral: (x + neutral) / 2
    MixNeutral,

    /// Blend with specific value: (x + y) / 2
    MixWith(u8),

    /// XOR operation for binary features
    XorWith(u8),

    /// Identity (no transform)
    Identity,
}

pub struct TransformPipeline {
    transforms: Vec<(usize, NativeTransform)>, // (layer_index, transform)
}

impl TransformPipeline {
    /// Create empty pipeline
    pub fn new() -> Self {
        TransformPipeline {
            transforms: Vec::new(),
        }
    }

    /// Add transform for specific layer
    pub fn with_transform(mut self, layer: usize, transform: NativeTransform) -> Self {
        if layer < 16 {
            self.transforms.push((layer, transform));
        }
        self
    }

    /// Apply all transforms to state
    pub fn apply(&self, state: SilState) -> SilState {
        let mut result = state;

        for (layer_idx, transform) in &self.transforms {
            let byte_obj = result.get(*layer_idx);
            let transformed = self.apply_transform(byte_obj, *transform);
            result = result.with_layer(*layer_idx, transformed);
        }

        result
    }

    fn apply_transform(&self, byte: ByteSil, transform: NativeTransform) -> ByteSil {
        match transform {
            NativeTransform::Power(n) => byte.pow(n as i32),

            NativeTransform::Multiply(val) => {
                let other = ByteSil::from_u8(val);
                byte.mul(&other)
            }

            NativeTransform::MixNeutral => {
                let neutral = ByteSil::from_u8(128);
                byte.mix(&neutral)
            }

            NativeTransform::MixWith(val) => {
                let other = ByteSil::from_u8(val);
                byte.mix(&other)
            }

            NativeTransform::XorWith(val) => {
                let other = ByteSil::from_u8(val);
                byte.xor(&other)
            }

            NativeTransform::Identity => byte,
        }
    }

    /// Create semantic pipeline for layer category
    pub fn for_perception_layers() -> Self {
        // PERCEPTION layers (0-4): Raw linear encoding, no transforms
        TransformPipeline::new()
    }

    pub fn for_processing_layers() -> Self {
        // PROCESSING layers (5-7): Light quantization
        TransformPipeline::new()
            .with_transform(5, NativeTransform::Power(1)) // Electronic: mild
            .with_transform(6, NativeTransform::Power(1)) // Psychomotor: mild
            .with_transform(7, NativeTransform::MixNeutral) // Environmental: blend
    }

    pub fn for_interaction_layers() -> Self {
        // INTERACTION layers (8-A): Cross-layer blending
        TransformPipeline::new()
            .with_transform(8, NativeTransform::MixNeutral)  // Cybernetic
            .with_transform(9, NativeTransform::MixNeutral)  // Geopolitical
            .with_transform(10, NativeTransform::MixNeutral) // Cosmopolitical
    }

    pub fn for_emergence_layers() -> Self {
        // EMERGENCE layers (B-C): Power amplification
        TransformPipeline::new()
            .with_transform(11, NativeTransform::Power(2))   // Synergic
            .with_transform(12, NativeTransform::Power(3))   // Quantum
    }

    pub fn for_meta_layers() -> Self {
        // META layers (D-F): Final transformations
        TransformPipeline::new()
            .with_transform(13, NativeTransform::Power(2))   // Superposition
            .with_transform(14, NativeTransform::Power(2))   // Entanglement
            .with_transform(15, NativeTransform::Identity)   // Collapse
    }

    /// Full semantic routing with all layers
    pub fn full_semantic() -> Self {
        let mut pipeline = TransformPipeline::new();

        // Extend with all semantic pipelines
        let perception = Self::for_perception_layers();
        let processing = Self::for_processing_layers();
        let interaction = Self::for_interaction_layers();
        let emergence = Self::for_emergence_layers();
        let meta = Self::for_meta_layers();

        for t in &perception.transforms {
            pipeline = pipeline.with_transform(t.0, t.1);
        }
        for t in &processing.transforms {
            pipeline = pipeline.with_transform(t.0, t.1);
        }
        for t in &interaction.transforms {
            pipeline = pipeline.with_transform(t.0, t.1);
        }
        for t in &emergence.transforms {
            pipeline = pipeline.with_transform(t.0, t.1);
        }
        for t in &meta.transforms {
            pipeline = pipeline.with_transform(t.0, t.1);
        }

        pipeline
    }
}

impl Default for TransformPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_creation() {
        let pipeline = TransformPipeline::new()
            .with_transform(0, NativeTransform::Power(2))
            .with_transform(1, NativeTransform::MixNeutral);

        assert_eq!(pipeline.transforms.len(), 2);
    }

    #[test]
    fn test_semantic_pipelines() {
        let perception = TransformPipeline::for_perception_layers();
        let processing = TransformPipeline::for_processing_layers();
        let full = TransformPipeline::full_semantic();

        // Perception should be empty (no transforms for raw input)
        assert_eq!(perception.transforms.len(), 0);

        // Processing should have transforms
        assert!(processing.transforms.len() > 0);

        // Full semantic should have most transforms
        assert!(full.transforms.len() > 0);
    }
}
