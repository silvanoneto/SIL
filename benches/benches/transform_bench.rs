//! # Transform Pipeline Benchmarks
//!
//! Measures throughput of transform pipelines and individual transforms.
//! Transforms are pure functions: State → State.
//!
//! Run: `cargo bench --bench transform_bench`

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use sil_core::prelude::*;
use sil_core::transforms::{
    PhaseShift, MagnitudeScale, LayerSwap, LayerXor, Identity,
    perception::{PerceptionAmplify, PerceptionNormalize},
    processing::{ProcessingAmplify, ProcessingRotate},
    meta::{PrepareCollapse, SetSuperposition, SetEntanglement},
};

/// Benchmark individual transforms
fn bench_individual_transforms(c: &mut Criterion) {
    let mut group = c.benchmark_group("individual_transforms");

    let state = SilState::neutral()
        .with_layer(0, ByteSil::new(5, 0))
        .with_layer(5, ByteSil::new(4, 64));

    // Phase shift
    let phase_shift = PhaseShift(32);
    group.bench_function("phase_shift", |b| {
        b.iter(|| {
            black_box(phase_shift.transform(&state))
        })
    });

    // Magnitude scale
    let mag_scale = MagnitudeScale(2);
    group.bench_function("magnitude_scale", |b| {
        b.iter(|| {
            black_box(mag_scale.transform(&state))
        })
    });

    // Layer swap
    let swap = LayerSwap(0, 5);
    group.bench_function("layer_swap", |b| {
        b.iter(|| {
            black_box(swap.transform(&state))
        })
    });

    // Layer XOR
    let xor = LayerXor { src_a: 0, src_b: 5, dest: 10 };
    group.bench_function("layer_xor", |b| {
        b.iter(|| {
            black_box(xor.transform(&state))
        })
    });

    // Identity
    let identity = Identity;
    group.bench_function("identity", |b| {
        b.iter(|| {
            black_box(identity.transform(&state))
        })
    });

    group.finish();
}

/// Benchmark perception transforms
fn bench_perception_transforms(c: &mut Criterion) {
    let mut group = c.benchmark_group("perception_transforms");

    let state = SilState::vacuum()
        .with_layer(0, ByteSil::new(5, 0))
        .with_layer(1, ByteSil::new(4, 32))
        .with_layer(2, ByteSil::new(3, 64))
        .with_layer(3, ByteSil::new(2, 96))
        .with_layer(4, ByteSil::new(4, 128));

    let amplify = PerceptionAmplify(2);
    group.bench_function("perception_amplify", |b| {
        b.iter(|| {
            black_box(amplify.transform(&state))
        })
    });

    let normalize = PerceptionNormalize;
    group.bench_function("perception_normalize", |b| {
        b.iter(|| {
            black_box(normalize.transform(&state))
        })
    });

    group.finish();
}

/// Benchmark processing transforms
fn bench_processing_transforms(c: &mut Criterion) {
    let mut group = c.benchmark_group("processing_transforms");

    let state = SilState::vacuum()
        .with_layer(5, ByteSil::new(5, 0))
        .with_layer(6, ByteSil::new(4, 32))
        .with_layer(7, ByteSil::new(3, 64));

    let amplify = ProcessingAmplify(1);
    group.bench_function("processing_amplify", |b| {
        b.iter(|| {
            black_box(amplify.transform(&state))
        })
    });

    let rotate = ProcessingRotate(64);
    group.bench_function("processing_rotate", |b| {
        b.iter(|| {
            black_box(rotate.transform(&state))
        })
    });

    group.finish();
}

/// Benchmark meta transforms
fn bench_meta_transforms(c: &mut Criterion) {
    let mut group = c.benchmark_group("meta_transforms");

    let state = SilState::neutral()
        .with_layer(0, ByteSil::new(5, 0))
        .with_layer(5, ByteSil::new(4, 64))
        .with_layer(12, ByteSil::new(6, 128));

    let prepare = PrepareCollapse;
    group.bench_function("prepare_collapse", |b| {
        b.iter(|| {
            black_box(prepare.transform(&state))
        })
    });

    let set_super = SetSuperposition(ByteSil::new(5, 0));
    group.bench_function("set_superposition", |b| {
        b.iter(|| {
            black_box(set_super.transform(&state))
        })
    });

    let set_entangle = SetEntanglement(ByteSil::new(4, 64));
    group.bench_function("set_entanglement", |b| {
        b.iter(|| {
            black_box(set_entangle.transform(&state))
        })
    });

    group.finish();
}

/// Benchmark pipeline execution
fn bench_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("pipeline");

    let state = SilState::neutral()
        .with_layer(0, ByteSil::new(5, 0))
        .with_layer(5, ByteSil::new(4, 64));

    // Small pipeline (2 transforms)
    let small_pipeline = Pipeline::new(vec![
        Box::new(PhaseShift(16)),
        Box::new(MagnitudeScale(1)),
    ]);

    group.bench_function("pipeline_2", |b| {
        b.iter(|| {
            black_box(small_pipeline.transform(&state))
        })
    });

    // Medium pipeline (5 transforms)
    let medium_pipeline = Pipeline::new(vec![
        Box::new(PhaseShift(16)),
        Box::new(MagnitudeScale(1)),
        Box::new(LayerSwap(0, 5)),
        Box::new(PhaseShift(32)),
        Box::new(PrepareCollapse),
    ]);

    group.bench_function("pipeline_5", |b| {
        b.iter(|| {
            black_box(medium_pipeline.transform(&state))
        })
    });

    // Large pipeline (10 transforms)
    let large_pipeline = Pipeline::new(vec![
        Box::new(PhaseShift(8)),
        Box::new(MagnitudeScale(1)),
        Box::new(LayerSwap(0, 1)),
        Box::new(PhaseShift(16)),
        Box::new(MagnitudeScale(1)),
        Box::new(LayerSwap(1, 2)),
        Box::new(PhaseShift(24)),
        Box::new(MagnitudeScale(1)),
        Box::new(LayerSwap(2, 3)),
        Box::new(PrepareCollapse),
    ]);

    group.bench_function("pipeline_10", |b| {
        b.iter(|| {
            black_box(large_pipeline.transform(&state))
        })
    });

    group.finish();
}

/// Benchmark pipeline throughput (states/second)
fn bench_pipeline_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("pipeline_throughput");

    let pipeline = Pipeline::new(vec![
        Box::new(PhaseShift(16)),
        Box::new(MagnitudeScale(1)),
        Box::new(LayerXor { src_a: 0, src_b: 5, dest: 10 }),
        Box::new(PrepareCollapse),
    ]);

    for batch_size in [10u64, 100, 1000, 10000].iter() {
        let states: Vec<SilState> = (0..*batch_size as usize)
            .map(|i| SilState::vacuum()
                .with_layer(0, ByteSil::new((i % 16) as i8 - 8, (i % 256) as u8)))
            .collect();

        group.throughput(Throughput::Elements(*batch_size));
        group.bench_with_input(
            BenchmarkId::new("batch", batch_size),
            &states,
            |b, s| {
                b.iter(|| {
                    for state in s.iter() {
                        black_box(pipeline.transform(state));
                    }
                })
            }
        );
    }

    group.finish();
}

/// Benchmark transform composition patterns
fn bench_transform_composition(c: &mut Criterion) {
    let mut group = c.benchmark_group("transform_composition");

    let state = SilState::neutral()
        .with_layer(0, ByteSil::new(5, 0));

    let t1 = PhaseShift(16);
    let t2 = MagnitudeScale(1);
    let t3 = PhaseShift(32);

    // Manual chaining
    group.bench_function("manual_chain_3", |b| {
        b.iter(|| {
            let s = t1.transform(&state);
            let s = t2.transform(&s);
            let s = t3.transform(&s);
            black_box(s)
        })
    });

    // Pipeline
    let pipeline = Pipeline::new(vec![
        Box::new(PhaseShift(16)),
        Box::new(MagnitudeScale(1)),
        Box::new(PhaseShift(32)),
    ]);

    group.bench_function("pipeline_3", |b| {
        b.iter(|| {
            black_box(pipeline.transform(&state))
        })
    });

    group.finish();
}

/// Benchmark full pipeline cycle (perception → processing → interaction → emergence → meta)
fn bench_full_cycle(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_cycle");

    let state = SilState::vacuum()
        .with_layer(0, ByteSil::new(6, 0))    // Photonic
        .with_layer(1, ByteSil::new(4, 32))   // Acoustic
        .with_layer(4, ByteSil::new(5, 64));  // Dermic

    // Full pipeline simulating L0→LF
    let full_pipeline = Pipeline::new(vec![
        // Perception processing
        Box::new(PerceptionAmplify(1)),
        // Perception → Electronic
        Box::new(LayerXor { src_a: 0, src_b: 1, dest: 5 }),
        // Electronic processing
        Box::new(ProcessingAmplify(1)),
        // Processing → Cybernetic
        Box::new(LayerSwap(5, 8)),
        // Interaction
        Box::new(PhaseShift(32)),
        // Emergence
        Box::new(MagnitudeScale(1)),
        // Meta
        Box::new(PrepareCollapse),
    ]);

    group.bench_function("full_l0_to_lf", |b| {
        b.iter(|| {
            black_box(full_pipeline.transform(&state))
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_individual_transforms,
    bench_perception_transforms,
    bench_processing_transforms,
    bench_meta_transforms,
    bench_pipeline,
    bench_pipeline_throughput,
    bench_transform_composition,
    bench_full_cycle,
);

criterion_main!(benches);
