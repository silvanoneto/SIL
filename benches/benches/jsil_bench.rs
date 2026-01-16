//! # JSIL I/O Benchmarks
//!
//! Measures performance of JSIL compression and I/O operations.
//!
//! Run: `cargo bench --bench jsil_bench`

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use sil_core::io::jsil::{JsilCompressor, JsilHeader, CompressionMode};

/// Benchmark JSIL header operations
fn bench_jsil_header(c: &mut Criterion) {
    let mut group = c.benchmark_group("jsil_header");

    group.bench_function("header_new", |b| {
        b.iter(|| {
            black_box(JsilHeader::new(CompressionMode::XorRotate))
        })
    });

    let header = JsilHeader::new(CompressionMode::XorRotate);

    group.bench_function("header_to_bytes", |b| {
        b.iter(|| {
            black_box(header.to_bytes())
        })
    });

    let bytes = header.to_bytes();

    group.bench_function("header_from_bytes", |b| {
        b.iter(|| {
            black_box(JsilHeader::from_bytes(&bytes).unwrap())
        })
    });

    group.bench_function("header_roundtrip", |b| {
        b.iter(|| {
            let h = JsilHeader::new(CompressionMode::Adaptive);
            let bytes = h.to_bytes();
            black_box(JsilHeader::from_bytes(&bytes).unwrap())
        })
    });

    group.bench_function("compression_ratio", |b| {
        b.iter(|| {
            black_box(header.compression_ratio())
        })
    });

    group.finish();
}

/// Benchmark JSIL compressor creation
fn bench_jsil_compressor_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("jsil_compressor_creation");

    group.bench_function("new_none", |b| {
        b.iter(|| {
            black_box(JsilCompressor::new(CompressionMode::None, 0))
        })
    });

    group.bench_function("new_xor", |b| {
        b.iter(|| {
            black_box(JsilCompressor::new(CompressionMode::Xor, 0x5A))
        })
    });

    group.bench_function("new_rotate", |b| {
        b.iter(|| {
            black_box(JsilCompressor::new(CompressionMode::Rotate, 4))
        })
    });

    group.bench_function("new_xor_rotate", |b| {
        b.iter(|| {
            black_box(JsilCompressor::new(CompressionMode::XorRotate, 0x5A))
        })
    });

    group.bench_function("default", |b| {
        b.iter(|| {
            black_box(JsilCompressor::default())
        })
    });

    group.bench_function("adaptive", |b| {
        b.iter(|| {
            black_box(JsilCompressor::adaptive())
        })
    });

    group.finish();
}

/// Benchmark JSIL compression with different modes
fn bench_jsil_compression(c: &mut Criterion) {
    let mut group = c.benchmark_group("jsil_compression");

    // Test data: JSON-like repetitive pattern
    let small_data: Vec<u8> = br#"{"layer":0,"rho":5,"theta":32}"#.repeat(10).to_vec();
    let medium_data: Vec<u8> = br#"{"layer":0,"rho":5,"theta":32}"#.repeat(100).to_vec();
    let large_data: Vec<u8> = br#"{"layer":0,"rho":5,"theta":32}"#.repeat(1000).to_vec();

    // None (passthrough)
    let comp_none = JsilCompressor::new(CompressionMode::None, 0);
    group.bench_function("compress_none_small", |b| {
        b.iter(|| black_box(comp_none.compress(&small_data)))
    });

    // XOR compression
    let comp_xor = JsilCompressor::new(CompressionMode::Xor, 0x5A);

    group.bench_function("compress_xor_small", |b| {
        b.iter(|| black_box(comp_xor.compress(&small_data)))
    });

    group.bench_function("compress_xor_medium", |b| {
        b.iter(|| black_box(comp_xor.compress(&medium_data)))
    });

    group.bench_function("compress_xor_large", |b| {
        b.iter(|| black_box(comp_xor.compress(&large_data)))
    });

    // Rotate compression
    let comp_rotate = JsilCompressor::new(CompressionMode::Rotate, 4);

    group.bench_function("compress_rotate_small", |b| {
        b.iter(|| black_box(comp_rotate.compress(&small_data)))
    });

    group.bench_function("compress_rotate_medium", |b| {
        b.iter(|| black_box(comp_rotate.compress(&medium_data)))
    });

    // XorRotate (combined)
    let comp_xor_rotate = JsilCompressor::new(CompressionMode::XorRotate, 0x5A);

    group.bench_function("compress_xor_rotate_small", |b| {
        b.iter(|| black_box(comp_xor_rotate.compress(&small_data)))
    });

    group.bench_function("compress_xor_rotate_medium", |b| {
        b.iter(|| black_box(comp_xor_rotate.compress(&medium_data)))
    });

    group.bench_function("compress_xor_rotate_large", |b| {
        b.iter(|| black_box(comp_xor_rotate.compress(&large_data)))
    });

    // Adaptive
    let comp_adaptive = JsilCompressor::adaptive();

    group.bench_function("compress_adaptive_small", |b| {
        b.iter(|| black_box(comp_adaptive.compress(&small_data)))
    });

    group.bench_function("compress_adaptive_medium", |b| {
        b.iter(|| black_box(comp_adaptive.compress(&medium_data)))
    });

    group.finish();
}

/// Benchmark JSIL decompression
fn bench_jsil_decompression(c: &mut Criterion) {
    let mut group = c.benchmark_group("jsil_decompression");

    let original_data: Vec<u8> = br#"{"layer":0,"rho":5,"theta":32}"#.repeat(100).to_vec();

    // XOR decompression
    let comp_xor = JsilCompressor::new(CompressionMode::Xor, 0x5A);
    let compressed_xor = comp_xor.compress(&original_data);

    group.bench_function("decompress_xor", |b| {
        b.iter(|| black_box(comp_xor.decompress(&compressed_xor)))
    });

    // Rotate decompression
    let comp_rotate = JsilCompressor::new(CompressionMode::Rotate, 4);
    let compressed_rotate = comp_rotate.compress(&original_data);

    group.bench_function("decompress_rotate", |b| {
        b.iter(|| black_box(comp_rotate.decompress(&compressed_rotate)))
    });

    // XorRotate decompression
    let comp_xor_rotate = JsilCompressor::new(CompressionMode::XorRotate, 0x5A);
    let compressed_xor_rotate = comp_xor_rotate.compress(&original_data);

    group.bench_function("decompress_xor_rotate", |b| {
        b.iter(|| black_box(comp_xor_rotate.decompress(&compressed_xor_rotate)))
    });

    group.finish();
}

/// Benchmark JSIL roundtrip (compress + decompress)
fn bench_jsil_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("jsil_roundtrip");

    let data: Vec<u8> = br#"{"layer":0,"rho":5,"theta":32}"#.repeat(100).to_vec();

    let comp_xor = JsilCompressor::new(CompressionMode::Xor, 0x5A);
    group.bench_function("roundtrip_xor", |b| {
        b.iter(|| {
            let compressed = comp_xor.compress(&data);
            black_box(comp_xor.decompress(&compressed))
        })
    });

    let comp_rotate = JsilCompressor::new(CompressionMode::Rotate, 4);
    group.bench_function("roundtrip_rotate", |b| {
        b.iter(|| {
            let compressed = comp_rotate.compress(&data);
            black_box(comp_rotate.decompress(&compressed))
        })
    });

    let comp_xor_rotate = JsilCompressor::new(CompressionMode::XorRotate, 0x5A);
    group.bench_function("roundtrip_xor_rotate", |b| {
        b.iter(|| {
            let compressed = comp_xor_rotate.compress(&data);
            black_box(comp_xor_rotate.decompress(&compressed))
        })
    });

    group.finish();
}

/// Benchmark compression throughput with varying data sizes
fn bench_jsil_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("jsil_throughput");

    let comp = JsilCompressor::new(CompressionMode::XorRotate, 0x5A);

    for size in [1024u64, 10240, 102400, 1048576].iter() {
        let data: Vec<u8> = (0..*size as usize)
            .map(|i| (i % 256) as u8)
            .collect();

        group.throughput(Throughput::Bytes(*size));
        group.bench_with_input(
            BenchmarkId::new("compress", size),
            &data,
            |b, d| {
                b.iter(|| black_box(comp.compress(d)))
            }
        );

        let compressed = comp.compress(&data);
        group.bench_with_input(
            BenchmarkId::new("decompress", size),
            &compressed,
            |b, c| {
                b.iter(|| black_box(comp.decompress(c)))
            }
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_jsil_header,
    bench_jsil_compressor_creation,
    bench_jsil_compression,
    bench_jsil_decompression,
    bench_jsil_roundtrip,
    bench_jsil_throughput,
);

criterion_main!(benches);
