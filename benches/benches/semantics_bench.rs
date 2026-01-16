//! # Semantics Benchmarks
//!
//! Measures performance of layer semantic interpretation.
//!
//! Run: `cargo bench --bench semantics_bench`

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use sil_core::semantics::{
    LayerGroup, interpret_rho_for_layer, interpret_theta_for_layer,
};

/// Benchmark LayerGroup operations
fn bench_layer_group(c: &mut Criterion) {
    let mut group = c.benchmark_group("layer_group");

    // LayerGroup::from_index for different layers
    group.bench_function("from_index_perception", |b| {
        b.iter(|| black_box(LayerGroup::from_index(0)))
    });

    group.bench_function("from_index_processing", |b| {
        b.iter(|| black_box(LayerGroup::from_index(5)))
    });

    group.bench_function("from_index_interaction", |b| {
        b.iter(|| black_box(LayerGroup::from_index(8)))
    });

    group.bench_function("from_index_emergence", |b| {
        b.iter(|| black_box(LayerGroup::from_index(11)))
    });

    group.bench_function("from_index_meta", |b| {
        b.iter(|| black_box(LayerGroup::from_index(15)))
    });

    // All 16 layers
    group.bench_function("from_index_all_16", |b| {
        b.iter(|| {
            for i in 0..16 {
                black_box(LayerGroup::from_index(i));
            }
        })
    });

    group.finish();
}

/// Benchmark LayerGroup methods
fn bench_layer_group_methods(c: &mut Criterion) {
    let mut group = c.benchmark_group("layer_group_methods");

    let perception = LayerGroup::Perception;
    let processing = LayerGroup::Processing;
    let interaction = LayerGroup::Interaction;
    let emergence = LayerGroup::Emergence;
    let meta = LayerGroup::Meta;

    group.bench_function("name_perception", |b| {
        b.iter(|| black_box(perception.name()))
    });

    group.bench_function("name_meta", |b| {
        b.iter(|| black_box(meta.name()))
    });

    group.bench_function("layers_perception", |b| {
        b.iter(|| black_box(perception.layers()))
    });

    group.bench_function("layers_processing", |b| {
        b.iter(|| black_box(processing.layers()))
    });

    group.bench_function("layers_interaction", |b| {
        b.iter(|| black_box(interaction.layers()))
    });

    group.bench_function("layers_emergence", |b| {
        b.iter(|| black_box(emergence.layers()))
    });

    group.bench_function("layers_meta", |b| {
        b.iter(|| black_box(meta.layers()))
    });

    // Access all layers from all groups
    group.bench_function("all_groups_layers", |b| {
        b.iter(|| {
            black_box(perception.layers());
            black_box(processing.layers());
            black_box(interaction.layers());
            black_box(emergence.layers());
            black_box(meta.layers());
        })
    });

    group.finish();
}

/// Benchmark rho interpretation for different layers
fn bench_interpret_rho(c: &mut Criterion) {
    let mut group = c.benchmark_group("interpret_rho");

    // Test different rho values
    let rho_values: [i8; 5] = [-8, -4, 0, 4, 7];

    // Perception layers (L0-L4)
    group.bench_function("layer_0_photonic", |b| {
        b.iter(|| black_box(interpret_rho_for_layer(0, 5)))
    });

    group.bench_function("layer_1_acoustic", |b| {
        b.iter(|| black_box(interpret_rho_for_layer(1, 5)))
    });

    group.bench_function("layer_2_olfactory", |b| {
        b.iter(|| black_box(interpret_rho_for_layer(2, 5)))
    });

    group.bench_function("layer_3_gustatory", |b| {
        b.iter(|| black_box(interpret_rho_for_layer(3, 5)))
    });

    group.bench_function("layer_4_haptic", |b| {
        b.iter(|| black_box(interpret_rho_for_layer(4, 5)))
    });

    // Processing layers (L5-L7)
    group.bench_function("layer_5_electronic", |b| {
        b.iter(|| black_box(interpret_rho_for_layer(5, 5)))
    });

    group.bench_function("layer_6_actuator", |b| {
        b.iter(|| black_box(interpret_rho_for_layer(6, 5)))
    });

    group.bench_function("layer_7_environment", |b| {
        b.iter(|| black_box(interpret_rho_for_layer(7, 5)))
    });

    // Meta layers (LD-LF)
    group.bench_function("layer_13_fork", |b| {
        b.iter(|| black_box(interpret_rho_for_layer(13, 5)))
    });

    group.bench_function("layer_14_entanglement", |b| {
        b.iter(|| black_box(interpret_rho_for_layer(14, 5)))
    });

    group.bench_function("layer_15_collapse", |b| {
        b.iter(|| black_box(interpret_rho_for_layer(15, 5)))
    });

    // All layers with same rho
    group.bench_function("all_16_layers", |b| {
        b.iter(|| {
            for layer in 0..16 {
                black_box(interpret_rho_for_layer(layer, 5));
            }
        })
    });

    // Single layer with all rho values
    group.bench_function("layer_0_all_rho_values", |b| {
        b.iter(|| {
            for rho in rho_values {
                black_box(interpret_rho_for_layer(0, rho));
            }
        })
    });

    group.finish();
}

/// Benchmark theta interpretation for different layers
fn bench_interpret_theta(c: &mut Criterion) {
    let mut group = c.benchmark_group("interpret_theta");

    // Test different theta values
    let theta_values: [u8; 5] = [0, 64, 128, 192, 255];

    // Perception layers
    group.bench_function("layer_0_photonic", |b| {
        b.iter(|| black_box(interpret_theta_for_layer(0, 128)))
    });

    group.bench_function("layer_1_acoustic", |b| {
        b.iter(|| black_box(interpret_theta_for_layer(1, 128)))
    });

    // Processing layers
    group.bench_function("layer_5_electronic", |b| {
        b.iter(|| black_box(interpret_theta_for_layer(5, 128)))
    });

    // Interaction layers
    group.bench_function("layer_8_cybernetic", |b| {
        b.iter(|| black_box(interpret_theta_for_layer(8, 128)))
    });

    group.bench_function("layer_9_geopolitical", |b| {
        b.iter(|| black_box(interpret_theta_for_layer(9, 128)))
    });

    group.bench_function("layer_10_cosmopolitan", |b| {
        b.iter(|| black_box(interpret_theta_for_layer(10, 128)))
    });

    // Meta layers
    group.bench_function("layer_15_collapse", |b| {
        b.iter(|| black_box(interpret_theta_for_layer(15, 128)))
    });

    // All layers with same theta
    group.bench_function("all_16_layers", |b| {
        b.iter(|| {
            for layer in 0..16 {
                black_box(interpret_theta_for_layer(layer, 128));
            }
        })
    });

    // Single layer with all theta values
    group.bench_function("layer_0_all_theta_values", |b| {
        b.iter(|| {
            for theta in theta_values {
                black_box(interpret_theta_for_layer(0, theta));
            }
        })
    });

    group.finish();
}

/// Benchmark combined rho + theta interpretation
fn bench_interpret_combined(c: &mut Criterion) {
    let mut group = c.benchmark_group("interpret_combined");

    // Interpret both rho and theta for same layer
    group.bench_function("layer_0_both", |b| {
        b.iter(|| {
            black_box(interpret_rho_for_layer(0, 5));
            black_box(interpret_theta_for_layer(0, 128));
        })
    });

    group.bench_function("layer_5_both", |b| {
        b.iter(|| {
            black_box(interpret_rho_for_layer(5, 5));
            black_box(interpret_theta_for_layer(5, 128));
        })
    });

    group.bench_function("layer_15_both", |b| {
        b.iter(|| {
            black_box(interpret_rho_for_layer(15, 5));
            black_box(interpret_theta_for_layer(15, 128));
        })
    });

    // Full state interpretation (all 16 layers, both rho and theta)
    group.bench_function("full_state_interpretation", |b| {
        b.iter(|| {
            for layer in 0u8..16 {
                black_box(interpret_rho_for_layer(layer, 5));
                black_box(interpret_theta_for_layer(layer, 128));
            }
        })
    });

    group.finish();
}

/// Benchmark batch semantics interpretation
fn bench_interpret_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("interpret_batch");

    for batch_size in [10u64, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("rho_batch", batch_size),
            batch_size,
            |b, &size| {
                b.iter(|| {
                    for i in 0..size {
                        let layer = (i % 16) as u8;
                        let rho = ((i % 16) as i8) - 8;
                        black_box(interpret_rho_for_layer(layer, rho));
                    }
                })
            }
        );

        group.bench_with_input(
            BenchmarkId::new("theta_batch", batch_size),
            batch_size,
            |b, &size| {
                b.iter(|| {
                    for i in 0..size {
                        let layer = (i % 16) as u8;
                        let theta = (i % 256) as u8;
                        black_box(interpret_theta_for_layer(layer, theta));
                    }
                })
            }
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_layer_group,
    bench_layer_group_methods,
    bench_interpret_rho,
    bench_interpret_theta,
    bench_interpret_combined,
    bench_interpret_batch,
);

criterion_main!(benches);
