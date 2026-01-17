//! ML Pipeline - End-to-End Integration
//!
//! Unified interface for:
//! 1. Feature encoding (linear with high fidelity)
//! 2. Semantic transforms (optional post-processing)
//! 3. Model inference
//! 4. Result decoding

use sil_core::SilState;
use crate::core::encoder::LinearEncoder;
use crate::core::transforms::TransformPipeline;
use crate::core::semantic_layers::{SemanticLayer, SemanticLayerSet};

#[derive(Debug, Clone, Copy)]
pub enum PipelineConfig {
    /// Pure linear encoding, no semantic transforms
    /// Best for raw ML models (SVM, Neural Networks)
    Pure,

    /// Linear encoding + processing layer transforms
    /// For models sensitive to intermediate processing
    WithProcessing,

    /// Full semantic routing with all 5 pipeline stages
    /// For semantic-aware models (Quantum, Superposition variants)
    FullSemantic,
}

pub struct MlPipeline {
    config: PipelineConfig,
    layers: SemanticLayerSet,
}

impl MlPipeline {
    /// Create pipeline with configuration
    pub fn new(config: PipelineConfig) -> Self {
        MlPipeline {
            config,
            layers: SemanticLayerSet::new(),
        }
    }

    /// Encode features to SilState
    pub fn encode_features(&self, features: &[f32]) -> SilState {
        // Step 1: Linear encoding with maximum fidelity
        let mut state = LinearEncoder::encode(features);

        // Step 2: Apply semantic transforms if configured
        state = match self.config {
            PipelineConfig::Pure => state,
            PipelineConfig::WithProcessing => {
                TransformPipeline::for_processing_layers().apply(state)
            }
            PipelineConfig::FullSemantic => TransformPipeline::full_semantic().apply(state),
        };

        state
    }

    /// Decode SilState back to features
    pub fn decode_features(&self, state: &SilState) -> Vec<f32> {
        // Note: Decoding doesn't reverse semantic transforms
        // (transforms are one-way processing operations)
        // Instead, we recover original features from the encoded state
        LinearEncoder::decode(state)
    }

    /// Process feature vector through full pipeline
    pub fn process(&self, features: &[f32]) -> (SilState, Vec<f32>) {
        let state = self.encode_features(features);
        let recovered = self.decode_features(&state);
        (state, recovered)
    }

    /// Get semantic layer metadata for feature
    pub fn get_layer_metadata(&self, feature_idx: usize) -> Option<SemanticLayer> {
        self.layers.get(feature_idx)
    }

    /// Get all layers by category
    pub fn layers_by_category(&self, category: &str) -> Vec<(usize, SemanticLayer)> {
        self.layers
            .by_category(category)
            .into_iter()
            .map(|layer| (layer.index(), layer))
            .collect()
    }

    /// Measure encoding fidelity
    pub fn measure_fidelity(&self, features: &[f32]) -> (f32, f32) {
        LinearEncoder::measure_fidelity(features)
    }

    /// Get pipeline configuration name
    pub fn config_name(&self) -> &'static str {
        match self.config {
            PipelineConfig::Pure => "Pure Linear Encoding",
            PipelineConfig::WithProcessing => "Linear + Processing Transforms",
            PipelineConfig::FullSemantic => "Full Semantic Routing",
        }
    }
}

impl Default for MlPipeline {
    fn default() -> Self {
        Self::new(PipelineConfig::Pure)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_creation() {
        let pipeline_pure = MlPipeline::new(PipelineConfig::Pure);
        let pipeline_semantic = MlPipeline::new(PipelineConfig::FullSemantic);

        assert_eq!(pipeline_pure.config_name(), "Pure Linear Encoding");
        assert_eq!(
            pipeline_semantic.config_name(),
            "Full Semantic Routing"
        );
    }

    #[test]
    fn test_encode_decode_fidelity() {
        let pipeline = MlPipeline::new(PipelineConfig::Pure);

        let features = vec![
            -0.9, -0.5, -0.1, 0.0, 0.1, 0.5, 0.9, 0.3, -0.7, 0.2, -0.4, 0.6, -0.8, 0.4, -0.2,
            0.7,
        ];

        let (mean_error, max_error) = pipeline.measure_fidelity(&features);

        println!("Pipeline fidelity ({}): ", pipeline.config_name());
        println!("  Mean error: {:.6}", mean_error);
        println!("  Max error:  {:.6}", max_error);

        assert!(mean_error < 0.01, "Mean error exceeds ML requirement");
        assert!(max_error < 0.03, "Max error exceeds threshold");
    }

    #[test]
    fn test_layer_metadata() {
        let pipeline = MlPipeline::new(PipelineConfig::Pure);

        let layer_0 = pipeline.get_layer_metadata(0);
        assert_eq!(layer_0, Some(SemanticLayer::Photonic));

        let layer_5 = pipeline.get_layer_metadata(5);
        assert_eq!(layer_5, Some(SemanticLayer::Electronic));
    }

    #[test]
    fn test_category_layers() {
        let pipeline = MlPipeline::new(PipelineConfig::Pure);

        let perception = pipeline.layers_by_category("PERCEPTION");
        assert_eq!(perception.len(), 5);

        let meta = pipeline.layers_by_category("META");
        assert_eq!(meta.len(), 3);
    }
}
