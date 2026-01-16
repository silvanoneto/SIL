//! # Layer Benchmarks
//!
//! Measures performance of layer-specific operations across the 16-layer SIL architecture.
//!
//! Run: `cargo bench --bench layer_bench`

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use sil_core::prelude::*;
use sil_core::state::layers;

/// Benchmark layer group access
fn bench_layer_groups(c: &mut Criterion) {
    let mut group = c.benchmark_group("layer_groups");

    let state = SilState::neutral()
        .with_layer(layers::PHOTONIC, ByteSil::new(7, 0))
        .with_layer(layers::ACOUSTIC, ByteSil::new(6, 32))
        .with_layer(layers::ELECTRONIC, ByteSil::new(5, 64))
        .with_layer(layers::DERMIC, ByteSil::new(4, 128));

    group.bench_function("single_layer_access", |b| {
        b.iter(|| black_box(state.layers[layers::PHOTONIC]))
    });

    group.bench_function("perception_5_layers", |b| {
        b.iter(|| {
            black_box(state.layers[0]);
            black_box(state.layers[1]);
            black_box(state.layers[2]);
            black_box(state.layers[3]);
            black_box(state.layers[4]);
        })
    });

    group.bench_function("perception_group", |b| {
        b.iter(|| black_box(state.perception()))
    });

    group.bench_function("processing_group", |b| {
        b.iter(|| black_box(state.processing()))
    });

    group.bench_function("interaction_group", |b| {
        b.iter(|| black_box(state.interaction()))
    });

    group.bench_function("emergence_group", |b| {
        b.iter(|| black_box(state.emergence()))
    });

    group.bench_function("meta_group", |b| {
        b.iter(|| black_box(state.meta()))
    });

    group.finish();
}

/// Benchmark layer fusion (XOR across layers)
fn bench_layer_fusion(c: &mut Criterion) {
    let mut group = c.benchmark_group("layer_fusion");

    let state = SilState::neutral()
        .with_layer(0, ByteSil::new(7, 0))
        .with_layer(1, ByteSil::new(6, 32))
        .with_layer(2, ByteSil::new(5, 64))
        .with_layer(3, ByteSil::new(4, 96))
        .with_layer(4, ByteSil::new(3, 128));

    group.bench_function("perception_xor_manual", |b| {
        b.iter(|| {
            let fused = state.layers[0]
                .xor(&state.layers[1])
                .xor(&state.layers[2])
                .xor(&state.layers[3])
                .xor(&state.layers[4]);
            black_box(fused)
        })
    });

    group.bench_function("all_layers_xor", |b| {
        b.iter(|| {
            let mut result = ByteSil::NULL;
            for i in 0..16 {
                result = result.xor(&state.layers[i]);
            }
            black_box(result)
        })
    });

    group.bench_function("collapse_xor", |b| {
        b.iter(|| black_box(state.collapse(CollapseStrategy::Xor)))
    });

    group.finish();
}

/// Benchmark layer projection (mask-based selection)
fn bench_layer_projection(c: &mut Criterion) {
    let mut group = c.benchmark_group("layer_projection");

    let state = SilState::maximum();

    // Perception mask: L0-L4 = 0b0000000000011111 = 0x001F
    group.bench_function("perception_mask", |b| {
        b.iter(|| black_box(state.project(0x001F)))
    });

    // Processing mask: L5-L7 = 0b0000000011100000 = 0x00E0
    group.bench_function("processing_mask", |b| {
        b.iter(|| black_box(state.project(0x00E0)))
    });

    // Interaction mask: L8-LA = 0b0000011100000000 = 0x0700
    group.bench_function("interaction_mask", |b| {
        b.iter(|| black_box(state.project(0x0700)))
    });

    // Meta mask: LD-LF = 0b1110000000000000 = 0xE000
    group.bench_function("meta_mask", |b| {
        b.iter(|| black_box(state.project(0xE000)))
    });

    // All layers: 0xFFFF
    group.bench_function("all_layers_mask", |b| {
        b.iter(|| black_box(state.project(0xFFFF)))
    });

    group.finish();
}

/// Benchmark layer-by-layer tensor product
fn bench_layer_tensor(c: &mut Criterion) {
    let mut group = c.benchmark_group("layer_tensor");

    let state_a = SilState::neutral()
        .with_layer(0, ByteSil::new(5, 32))
        .with_layer(5, ByteSil::new(3, 64));

    let state_b = SilState::neutral()
        .with_layer(0, ByteSil::new(2, 16))
        .with_layer(5, ByteSil::new(4, 128));

    group.bench_function("tensor_product", |b| {
        b.iter(|| black_box(state_a.tensor(&state_b)))
    });

    group.bench_function("tensor_manual", |b| {
        b.iter(|| {
            let mut result = SilState::vacuum();
            for i in 0..16 {
                result = result.with_layer(i, state_a.layers[i].mul(&state_b.layers[i]));
            }
            black_box(result)
        })
    });

    group.finish();
}

/// Benchmark batch layer operations
fn bench_layer_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("layer_batch");

    for size in [10, 100, 1000].iter() {
        let states: Vec<SilState> = (0..*size)
            .map(|i| {
                SilState::vacuum()
                    .with_layer(i % 16, ByteSil::new((i % 16) as i8 - 8, (i * 16 % 256) as u8))
            })
            .collect();

        group.bench_with_input(
            BenchmarkId::new("collapse_xor_batch", size),
            &states,
            |b, s| {
                b.iter(|| {
                    let mut result = ByteSil::NULL;
                    for state in s.iter() {
                        result = result.xor(&state.collapse(CollapseStrategy::Xor));
                    }
                    black_box(result)
                })
            }
        );

        group.bench_with_input(
            BenchmarkId::new("xor_chain", size),
            &states,
            |b, s| {
                b.iter(|| {
                    let mut result = SilState::vacuum();
                    for state in s.iter() {
                        result = result.xor(state);
                    }
                    black_box(result)
                })
            }
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_layer_groups,
    bench_layer_fusion,
    bench_layer_projection,
    bench_layer_tensor,
    bench_layer_batch,
);

criterion_main!(benches);
