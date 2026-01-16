//! # SilState Benchmarks
//!
//! Measures performance of SilState operations: creation, access, modification, and collapse.
//! All operations are O(1) or O(16) due to fixed architecture.
//!
//! Run: `cargo bench --bench state_bench`

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use sil_core::prelude::*;

/// Benchmark SilState creation
fn bench_state_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("state_creation");

    group.bench_function("vacuum", |b| {
        b.iter(|| {
            black_box(SilState::vacuum())
        })
    });

    group.bench_function("neutral", |b| {
        b.iter(|| {
            black_box(SilState::neutral())
        })
    });

    group.bench_function("maximum", |b| {
        b.iter(|| {
            black_box(SilState::maximum())
        })
    });

    group.bench_function("default", |b| {
        b.iter(|| {
            black_box(SilState::default())
        })
    });

    group.finish();
}

/// Benchmark SilState layer access
fn bench_state_access(c: &mut Criterion) {
    let mut group = c.benchmark_group("state_access");

    let state = SilState::neutral()
        .with_layer(0, ByteSil::new(7, 0))
        .with_layer(5, ByteSil::new(5, 64))
        .with_layer(15, ByteSil::new(3, 128));

    group.bench_function("get_single", |b| {
        b.iter(|| {
            black_box(state.get(5))
        })
    });

    group.bench_function("layer_single", |b| {
        b.iter(|| {
            black_box(state.layer(5))
        })
    });

    group.bench_function("layers_field", |b| {
        b.iter(|| {
            black_box(state.layers[5])
        })
    });

    group.bench_function("access_all_16", |b| {
        b.iter(|| {
            for i in 0..16 {
                black_box(state.layers[i]);
            }
        })
    });

    group.finish();
}

/// Benchmark SilState modification
fn bench_state_modification(c: &mut Criterion) {
    let mut group = c.benchmark_group("state_modification");

    let state = SilState::neutral();
    let value = ByteSil::new(5, 64);

    group.bench_function("with_layer_single", |b| {
        b.iter(|| {
            black_box(state.with_layer(5, value))
        })
    });

    group.bench_function("with_layer_chain_4", |b| {
        b.iter(|| {
            black_box(
                state
                    .with_layer(0, value)
                    .with_layer(5, value)
                    .with_layer(10, value)
                    .with_layer(15, value)
            )
        })
    });

    group.bench_function("with_layer_all_16", |b| {
        b.iter(|| {
            let mut s = state;
            for i in 0..16 {
                s = s.with_layer(i, value);
            }
            black_box(s)
        })
    });

    group.finish();
}

/// Benchmark SilState global operations
fn bench_state_global(c: &mut Criterion) {
    let mut group = c.benchmark_group("state_global");

    let state = SilState::neutral()
        .with_layer(0, ByteSil::new(7, 0))
        .with_layer(5, ByteSil::new(5, 64))
        .with_layer(10, ByteSil::new(3, 128))
        .with_layer(15, ByteSil::new(6, 192));

    let state2 = SilState::vacuum()
        .with_layer(1, ByteSil::new(4, 32))
        .with_layer(6, ByteSil::new(2, 96));

    group.bench_function("xor_two_states", |b| {
        b.iter(|| {
            black_box(state.xor(&state2))
        })
    });

    group.bench_function("tensor_two_states", |b| {
        b.iter(|| {
            black_box(state.tensor(&state2))
        })
    });

    group.bench_function("hash", |b| {
        b.iter(|| {
            black_box(state.hash())
        })
    });

    group.bench_function("perception_group", |b| {
        b.iter(|| {
            black_box(state.perception())
        })
    });

    group.bench_function("processing_group", |b| {
        b.iter(|| {
            black_box(state.processing())
        })
    });

    group.bench_function("interaction_group", |b| {
        b.iter(|| {
            black_box(state.interaction())
        })
    });

    group.bench_function("emergence_group", |b| {
        b.iter(|| {
            black_box(state.emergence())
        })
    });

    group.bench_function("meta_group", |b| {
        b.iter(|| {
            black_box(state.meta())
        })
    });

    group.finish();
}

/// Benchmark SilState collapse strategies
fn bench_state_collapse(c: &mut Criterion) {
    let mut group = c.benchmark_group("state_collapse");

    let state = SilState::neutral()
        .with_layer(0, ByteSil::new(7, 0))
        .with_layer(5, ByteSil::new(5, 64))
        .with_layer(10, ByteSil::new(3, 128))
        .with_layer(15, ByteSil::new(6, 192));

    group.bench_function("collapse_xor", |b| {
        b.iter(|| {
            black_box(state.collapse(CollapseStrategy::Xor))
        })
    });

    group.bench_function("collapse_sum", |b| {
        b.iter(|| {
            black_box(state.collapse(CollapseStrategy::Sum))
        })
    });

    group.bench_function("collapse_first", |b| {
        b.iter(|| {
            black_box(state.collapse(CollapseStrategy::First))
        })
    });

    group.bench_function("collapse_last", |b| {
        b.iter(|| {
            black_box(state.collapse(CollapseStrategy::Last))
        })
    });

    group.finish();
}

/// Benchmark SilState predicates
fn bench_state_predicates(c: &mut Criterion) {
    let mut group = c.benchmark_group("state_predicates");

    let vacuum = SilState::vacuum();
    let neutral = SilState::neutral();
    let mixed = SilState::vacuum()
        .with_layer(0, ByteSil::new(5, 0))
        .with_layer(5, ByteSil::ONE);

    group.bench_function("equality_same", |b| {
        b.iter(|| {
            black_box(mixed == mixed)
        })
    });

    group.bench_function("equality_different", |b| {
        b.iter(|| {
            black_box(vacuum == neutral)
        })
    });

    group.bench_function("vacuum_vs_neutral", |b| {
        b.iter(|| {
            black_box(vacuum != neutral)
        })
    });

    group.finish();
}

/// Benchmark SilState serialization
fn bench_state_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("state_serialization");

    let state = SilState::neutral()
        .with_layer(0, ByteSil::new(7, 0))
        .with_layer(5, ByteSil::new(5, 64))
        .with_layer(10, ByteSil::new(3, 128))
        .with_layer(15, ByteSil::new(6, 192));

    let bytes = state.to_bytes();

    group.bench_function("to_bytes", |b| {
        b.iter(|| {
            black_box(state.to_bytes())
        })
    });

    group.bench_function("from_bytes", |b| {
        b.iter(|| {
            black_box(SilState::from_bytes(&bytes))
        })
    });

    group.bench_function("roundtrip", |b| {
        b.iter(|| {
            let bytes = state.to_bytes();
            black_box(SilState::from_bytes(&bytes))
        })
    });

    group.finish();
}

/// Benchmark batch state operations
fn bench_state_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("state_batch");

    for size in [10, 100, 1000].iter() {
        let states: Vec<SilState> = (0..*size)
            .map(|i| SilState::vacuum()
                .with_layer((i % 16) as usize, ByteSil::new((i % 16) as i8 - 8, (i % 256) as u8)))
            .collect();

        group.bench_with_input(
            BenchmarkId::new("xor_reduce", size),
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

        group.bench_with_input(
            BenchmarkId::new("hash_all", size),
            &states,
            |b, s| {
                b.iter(|| {
                    let mut total = 0u128;
                    for state in s.iter() {
                        total = total.wrapping_add(state.hash());
                    }
                    black_box(total)
                })
            }
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_state_creation,
    bench_state_access,
    bench_state_modification,
    bench_state_global,
    bench_state_collapse,
    bench_state_predicates,
    bench_state_serialization,
    bench_state_batch,
);

criterion_main!(benches);
