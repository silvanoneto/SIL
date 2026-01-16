//! # SIMD Benchmarks
//!
//! Measures performance of SIMD-accelerated layer and batch operations.
//!
//! Run: `cargo bench --bench simd_bench`

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use sil_core::prelude::*;
use sil_core::state::simd::{
    xor_layers_simd, and_layers_simd, or_layers_simd,
    rotate_layers_simd, fold_layers_simd, FoldOp,
    batch_multiply, batch_divide, batch_xor, batch_power,
    batch_conjugate, batch_scale_rho, batch_rotate_theta,
    batch_sum, reduce_xor,
};

/// Benchmark SIMD layer operations (XOR, AND, OR)
fn bench_simd_layers(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_layers");

    let state = SilState::neutral()
        .with_layer(0, ByteSil::new(7, 0))
        .with_layer(1, ByteSil::new(6, 32))
        .with_layer(2, ByteSil::new(5, 64))
        .with_layer(3, ByteSil::new(4, 96))
        .with_layer(4, ByteSil::new(3, 128))
        .with_layer(5, ByteSil::new(2, 160))
        .with_layer(10, ByteSil::new(-2, 192))
        .with_layer(15, ByteSil::new(-5, 224));

    group.bench_function("xor_layers_simd", |b| {
        b.iter(|| black_box(xor_layers_simd(&state)))
    });

    group.bench_function("and_layers_simd", |b| {
        b.iter(|| black_box(and_layers_simd(&state)))
    });

    group.bench_function("or_layers_simd", |b| {
        b.iter(|| black_box(or_layers_simd(&state)))
    });

    // Compare with scalar collapse
    group.bench_function("collapse_xor_scalar", |b| {
        b.iter(|| black_box(state.collapse(CollapseStrategy::Xor)))
    });

    group.finish();
}

/// Benchmark SIMD layer rotation
fn bench_simd_rotate(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_rotate");

    let state = SilState::neutral()
        .with_layer(0, ByteSil::new(7, 0))
        .with_layer(5, ByteSil::new(5, 64))
        .with_layer(10, ByteSil::new(3, 128))
        .with_layer(15, ByteSil::new(1, 192));

    group.bench_function("rotate_1", |b| {
        b.iter(|| black_box(rotate_layers_simd(&state, 1)))
    });

    group.bench_function("rotate_4", |b| {
        b.iter(|| black_box(rotate_layers_simd(&state, 4)))
    });

    group.bench_function("rotate_8", |b| {
        b.iter(|| black_box(rotate_layers_simd(&state, 8)))
    });

    group.bench_function("rotate_15", |b| {
        b.iter(|| black_box(rotate_layers_simd(&state, 15)))
    });

    group.finish();
}

/// Benchmark SIMD fold operations
fn bench_simd_fold(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_fold");

    let state = SilState::maximum();

    group.bench_function("fold_xor", |b| {
        b.iter(|| black_box(fold_layers_simd(&state, FoldOp::Xor)))
    });

    group.bench_function("fold_add", |b| {
        b.iter(|| black_box(fold_layers_simd(&state, FoldOp::Add)))
    });

    group.bench_function("fold_mul", |b| {
        b.iter(|| black_box(fold_layers_simd(&state, FoldOp::Mul)))
    });

    group.finish();
}

/// Benchmark batch ByteSil operations
fn bench_simd_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_batch");

    // Create test vectors
    let size = 1024;
    let a: Vec<ByteSil> = (0..size)
        .map(|i| ByteSil::new((i % 16) as i8 - 8, (i % 256) as u8))
        .collect();
    let b: Vec<ByteSil> = (0..size)
        .map(|i| ByteSil::new(((i + 5) % 16) as i8 - 8, ((i * 17) % 256) as u8))
        .collect();

    group.throughput(Throughput::Elements(size as u64));

    group.bench_function("batch_multiply", |b_| {
        b_.iter(|| black_box(batch_multiply(&a, &b)))
    });

    group.bench_function("batch_divide", |b_| {
        b_.iter(|| black_box(batch_divide(&a, &b)))
    });

    group.bench_function("batch_xor", |b_| {
        b_.iter(|| black_box(batch_xor(&a, &b)))
    });

    group.bench_function("batch_power_2", |b_| {
        b_.iter(|| black_box(batch_power(&a, 2)))
    });

    group.bench_function("batch_power_5", |b_| {
        b_.iter(|| black_box(batch_power(&a, 5)))
    });

    group.bench_function("batch_conjugate", |b_| {
        b_.iter(|| black_box(batch_conjugate(&a)))
    });

    group.bench_function("batch_scale_rho", |b_| {
        b_.iter(|| black_box(batch_scale_rho(&a, 2)))
    });

    group.bench_function("batch_rotate_theta", |b_| {
        b_.iter(|| black_box(batch_rotate_theta(&a, 32)))
    });

    group.finish();
}

/// Benchmark batch reduction operations
fn bench_simd_reduce(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_reduce");

    for size in [16u64, 64, 256, 1024, 4096].iter() {
        let values: Vec<ByteSil> = (0..*size as usize)
            .map(|i| ByteSil::new((i % 16) as i8 - 8, (i % 256) as u8))
            .collect();

        group.throughput(Throughput::Elements(*size));

        group.bench_with_input(
            BenchmarkId::new("reduce_xor", size),
            &values,
            |b, v| {
                b.iter(|| black_box(reduce_xor(v)))
            }
        );

        group.bench_with_input(
            BenchmarkId::new("batch_sum", size),
            &values,
            |b, v| {
                b.iter(|| black_box(batch_sum(v)))
            }
        );
    }

    group.finish();
}

/// Benchmark batch operations with varying sizes
fn bench_simd_batch_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_batch_scaling");

    for size in [64u64, 256, 1024, 4096, 16384].iter() {
        let a: Vec<ByteSil> = (0..*size as usize)
            .map(|i| ByteSil::new((i % 16) as i8 - 8, (i % 256) as u8))
            .collect();
        let b: Vec<ByteSil> = (0..*size as usize)
            .map(|i| ByteSil::new(((i + 3) % 16) as i8 - 8, ((i * 7) % 256) as u8))
            .collect();

        group.throughput(Throughput::Elements(*size));

        group.bench_with_input(
            BenchmarkId::new("batch_multiply", size),
            &(&a, &b),
            |bench, (a, b)| {
                bench.iter(|| black_box(batch_multiply(a, b)))
            }
        );

        group.bench_with_input(
            BenchmarkId::new("batch_xor", size),
            &(&a, &b),
            |bench, (a, b)| {
                bench.iter(|| black_box(batch_xor(a, b)))
            }
        );
    }

    group.finish();
}

/// Benchmark SIMD vs scalar comparison
fn bench_simd_vs_scalar(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_vs_scalar");

    let state = SilState::maximum();

    // SIMD XOR
    group.bench_function("simd_xor_all_layers", |b| {
        b.iter(|| black_box(xor_layers_simd(&state)))
    });

    // Scalar XOR (manual loop)
    group.bench_function("scalar_xor_all_layers", |b| {
        b.iter(|| {
            let mut result = state.layers[0];
            for i in 1..16 {
                result = result.xor(&state.layers[i]);
            }
            black_box(result)
        })
    });

    // Batch multiply comparison
    let size = 256;
    let a: Vec<ByteSil> = (0..size)
        .map(|i| ByteSil::new((i % 16) as i8 - 8, (i % 256) as u8))
        .collect();
    let b: Vec<ByteSil> = (0..size)
        .map(|i| ByteSil::new(((i + 5) % 16) as i8 - 8, ((i * 17) % 256) as u8))
        .collect();

    group.bench_function("simd_batch_multiply_256", |bench| {
        bench.iter(|| black_box(batch_multiply(&a, &b)))
    });

    group.bench_function("scalar_batch_multiply_256", |bench| {
        bench.iter(|| {
            let result: Vec<ByteSil> = a.iter()
                .zip(b.iter())
                .map(|(x, y)| x.mul(y))
                .collect();
            black_box(result)
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_simd_layers,
    bench_simd_rotate,
    bench_simd_fold,
    bench_simd_batch,
    bench_simd_reduce,
    bench_simd_batch_scaling,
    bench_simd_vs_scalar,
);

criterion_main!(benches);
