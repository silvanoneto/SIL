//! # ByteSil Benchmarks
//!
//! Measures performance of ByteSil operations vs standard f64 complex numbers.
//! ByteSil's log-polar encoding enables O(1) multiplication, division, power, and root.
//!
//! Run: `cargo bench --bench bytesil_bench`

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use sil_core::prelude::*;

/// Benchmark ByteSil creation
fn bench_bytesil_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("bytesil_creation");

    group.bench_function("new", |b| {
        b.iter(|| {
            black_box(ByteSil::new(5, 64))
        })
    });

    // from_complex requires num_complex::Complex, skip in benchmarks
    // as the conversion is not a hot path

    group.bench_function("special_values", |b| {
        b.iter(|| {
            let _ = black_box(ByteSil::NULL);
            let _ = black_box(ByteSil::ONE);
            let _ = black_box(ByteSil::I);
            let _ = black_box(ByteSil::MAX);
        })
    });

    group.finish();
}

/// Benchmark ByteSil O(1) arithmetic operations
fn bench_bytesil_arithmetic(c: &mut Criterion) {
    let mut group = c.benchmark_group("bytesil_arithmetic");

    let z1 = ByteSil::new(5, 32);
    let z2 = ByteSil::new(3, 64);

    // Multiplication: O(1) via addition of logs
    group.bench_function("mul", |b| {
        b.iter(|| {
            black_box(z1.mul(&z2))
        })
    });

    // Division: O(1) via subtraction of logs
    group.bench_function("div", |b| {
        b.iter(|| {
            black_box(z1.div(&z2))
        })
    });

    // Power: O(1) via multiplication of log by scalar
    group.bench_function("pow_2", |b| {
        b.iter(|| {
            black_box(z1.pow(2))
        })
    });

    group.bench_function("pow_10", |b| {
        b.iter(|| {
            black_box(z1.pow(10))
        })
    });

    // Root: O(1) via division of log by scalar
    group.bench_function("root_2", |b| {
        b.iter(|| {
            black_box(z1.root(2))
        })
    });

    group.bench_function("root_3", |b| {
        b.iter(|| {
            black_box(z1.root(3))
        })
    });

    group.finish();
}

/// Benchmark ByteSil vs f64 complex multiplication
fn bench_bytesil_vs_f64(c: &mut Criterion) {
    let mut group = c.benchmark_group("bytesil_vs_f64");

    // ByteSil
    let bs1 = ByteSil::new(5, 32);
    let bs2 = ByteSil::new(3, 64);

    // f64 complex (re, im)
    let f1 = (1.5f64, 0.5f64);
    let f2 = (0.8f64, 1.2f64);

    // Multiplication
    group.bench_function("bytesil_mul", |b| {
        b.iter(|| {
            black_box(bs1.mul(&bs2))
        })
    });

    group.bench_function("f64_complex_mul", |b| {
        b.iter(|| {
            // (a+bi)(c+di) = (ac-bd) + (ad+bc)i
            let (a, b_im) = f1;
            let (c, d) = f2;
            black_box((a*c - b_im*d, a*d + b_im*c))
        })
    });

    // Division
    group.bench_function("bytesil_div", |b| {
        b.iter(|| {
            black_box(bs1.div(&bs2))
        })
    });

    group.bench_function("f64_complex_div", |b| {
        b.iter(|| {
            // (a+bi)/(c+di) = ((ac+bd) + (bc-ad)i) / (c²+d²)
            let (a, b_im) = f1;
            let (c, d) = f2;
            let denom = c*c + d*d;
            black_box(((a*c + b_im*d)/denom, (b_im*c - a*d)/denom))
        })
    });

    // Power (squared)
    group.bench_function("bytesil_pow2", |b| {
        b.iter(|| {
            black_box(bs1.pow(2))
        })
    });

    group.bench_function("f64_complex_pow2", |b| {
        b.iter(|| {
            let (a, b_im) = f1;
            // (a+bi)² = (a²-b²) + 2abi
            black_box((a*a - b_im*b_im, 2.0*a*b_im))
        })
    });

    group.finish();
}

/// Benchmark ByteSil unary operations
fn bench_bytesil_unary(c: &mut Criterion) {
    let mut group = c.benchmark_group("bytesil_unary");

    let z = ByteSil::new(5, 64);

    group.bench_function("conj", |b| {
        b.iter(|| {
            black_box(z.conj())
        })
    });

    group.bench_function("inv", |b| {
        b.iter(|| {
            black_box(z.inv())
        })
    });

    group.bench_function("norm", |b| {
        b.iter(|| {
            black_box(z.norm())
        })
    });

    group.bench_function("phase", |b| {
        b.iter(|| {
            black_box(z.phase())
        })
    });

    group.bench_function("to_polar", |b| {
        b.iter(|| {
            black_box(z.to_polar())
        })
    });

    group.finish();
}

/// Benchmark ByteSil XOR (for hashing)
fn bench_bytesil_xor(c: &mut Criterion) {
    let mut group = c.benchmark_group("bytesil_xor");

    let values: Vec<ByteSil> = (0..16)
        .map(|i| ByteSil::new(i as i8 - 8, (i * 16) as u8))
        .collect();

    group.bench_function("xor_pair", |b| {
        b.iter(|| {
            black_box(values[0].xor(&values[1]))
        })
    });

    group.bench_function("xor_16", |b| {
        b.iter(|| {
            let mut result = values[0];
            for v in values.iter().skip(1) {
                result = result.xor(v);
            }
            black_box(result)
        })
    });

    group.finish();
}

/// Benchmark ByteSil batch operations
fn bench_bytesil_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("bytesil_batch");

    for size in [16, 64, 256, 1024].iter() {
        let values: Vec<ByteSil> = (0..*size)
            .map(|i| ByteSil::new((i % 16) as i8 - 8, (i % 256) as u8))
            .collect();

        group.bench_with_input(
            BenchmarkId::new("mul_chain", size),
            &values,
            |b, v| {
                b.iter(|| {
                    let mut result = ByteSil::ONE;
                    for val in v.iter() {
                        result = result.mul(val);
                    }
                    black_box(result)
                })
            }
        );

        group.bench_with_input(
            BenchmarkId::new("xor_chain", size),
            &values,
            |b, v| {
                b.iter(|| {
                    let mut result = ByteSil::NULL;
                    for val in v.iter() {
                        result = result.xor(val);
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
    bench_bytesil_creation,
    bench_bytesil_arithmetic,
    bench_bytesil_vs_f64,
    bench_bytesil_unary,
    bench_bytesil_xor,
    bench_bytesil_batch,
);

criterion_main!(benches);
