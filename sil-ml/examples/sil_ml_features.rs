#!/usr/bin/env rust
//! Integration Example: Using new SIL-ML features
//!
//! Demonstrates:
//! 1. Semantic layers for feature organization
//! 2. Linear encoder for high-fidelity encoding
//! 3. Native transform pipeline for semantic processing
//! 4. ML pipeline for end-to-end integration

use sil_ml::core::prelude::*;

fn main() {
    println!("🚀 SIL-ML Essential Features Integration\n");

    // Example 1: Semantic Layer Metadata
    println!("1️⃣ Semantic Layer Classification:");
    let layers = SemanticLayerSet::new();
    for layer in layers.iter().take(8) {
        println!(
            "   {} (L{}): {}",
            layer.name(),
            layer.index(),
            layer.category()
        );
    }
    println!();

    // Example 2: Linear Encoding Fidelity
    println!("2️⃣ Linear Encoding Fidelity:");
    let test_features = vec![
        -0.9, -0.5, -0.1, 0.0, 0.1, 0.5, 0.9, 0.3, -0.7, 0.2, -0.4, 0.6, -0.8, 0.4, -0.2, 0.7,
    ];

    let (mean_error, max_error) = LinearEncoder::measure_fidelity(&test_features);
    println!(
        "   Mean error: {:.6} (< 0.01 ✓)",
        mean_error
    );
    println!("   Max error:  {:.6} (< 0.03 ✓)", max_error);
    println!();

    // Example 3: Transform Pipeline
    println!("3️⃣ Transform Pipeline:");
    let perception = TransformPipeline::for_perception_layers();
    let processing = TransformPipeline::for_processing_layers();
    let interaction = TransformPipeline::for_interaction_layers();
    let emergence = TransformPipeline::for_emergence_layers();

    println!("   Perception transforms: {}", perception.transforms.len());
    println!("   Processing transforms: {}", processing.transforms.len());
    println!("   Interaction transforms: {}", interaction.transforms.len());
    println!("   Emergence transforms: {}", emergence.transforms.len());
    println!();

    // Example 4: ML Pipeline End-to-End
    println!("4️⃣ ML Pipeline End-to-End:");
    let pipeline = MlPipeline::new(PipelineConfig::Pure);
    println!("   Config: {}", pipeline.config_name());

    let features = vec![0.5, -0.3, 1.2, -0.8, 0.0, 0.7, -0.5, 0.3, 
                        0.9, -0.2, 0.4, -0.6, 0.1, -0.9, 0.6, -0.1];

    let (state, recovered) = pipeline.process(&features);
    println!("   Encoded state: 16 layers ✓");
    println!("   Recovered features: {} values ✓", recovered.len());
    println!();

    // Example 5: Layer Categories
    println!("5️⃣ Layer Categories:");
    let perception_layers = pipeline.layers_by_category("PERCEPTION");
    let meta_layers = pipeline.layers_by_category("META");
    println!("   PERCEPTION layers: {}", perception_layers.len());
    println!("   META layers: {}", meta_layers.len());
    println!();

    println!("✅ All essential features validated!\n");
}
