//! # Processor Benchmarks
//!
//! Measures performance of processor detection and selection.
//!
//! Run: `cargo bench --bench processor_bench`

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sil_core::processors::{ProcessorType, ProcessorCapability, ProcessorInfo};

/// Benchmark processor availability detection
fn bench_processor_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("processor_detection");

    group.bench_function("cpu_is_available", |b| {
        b.iter(|| black_box(ProcessorType::Cpu.is_available()))
    });

    group.bench_function("gpu_is_available", |b| {
        b.iter(|| black_box(ProcessorType::Gpu.is_available()))
    });

    group.bench_function("npu_is_available", |b| {
        b.iter(|| black_box(ProcessorType::Npu.is_available()))
    });

    group.bench_function("fpga_is_available", |b| {
        b.iter(|| black_box(ProcessorType::Fpga.is_available()))
    });

    group.bench_function("hybrid_is_available", |b| {
        b.iter(|| black_box(ProcessorType::Hybrid.is_available()))
    });

    group.bench_function("check_all_types", |b| {
        b.iter(|| {
            black_box(ProcessorType::Cpu.is_available());
            black_box(ProcessorType::Gpu.is_available());
            black_box(ProcessorType::Npu.is_available());
            black_box(ProcessorType::Fpga.is_available());
        })
    });

    group.finish();
}

/// Benchmark processor listing (fresh vs cached)
fn bench_processor_listing(c: &mut Criterion) {
    let mut group = c.benchmark_group("processor_listing");

    // Note: available() internally uses available_cached()
    group.bench_function("available", |b| {
        b.iter(|| black_box(ProcessorType::available()))
    });

    // Cached version (should be near-instant after first call)
    group.bench_function("available_cached", |b| {
        b.iter(|| black_box(ProcessorType::available_cached()))
    });

    // Warm up the cache first, then measure
    let _ = ProcessorType::available_cached();
    group.bench_function("available_cached_warm", |b| {
        b.iter(|| black_box(ProcessorType::available_cached()))
    });

    group.finish();
}

/// Benchmark ProcessorInfo creation
fn bench_processor_info(c: &mut Criterion) {
    let mut group = c.benchmark_group("processor_info");

    group.bench_function("create_cpu_info", |b| {
        b.iter(|| {
            black_box(ProcessorInfo {
                processor_type: ProcessorType::Cpu,
                name: "Test CPU".to_string(),
                vendor: "Test Vendor".to_string(),
                capabilities: vec![
                    ProcessorCapability::MatrixOps,
                    ProcessorCapability::Gradients,
                ],
                memory_bytes: Some(16 * 1024 * 1024 * 1024), // 16GB
                compute_units: Some(8),
            })
        })
    });

    group.bench_function("create_gpu_info", |b| {
        b.iter(|| {
            black_box(ProcessorInfo {
                processor_type: ProcessorType::Gpu,
                name: "Test GPU".to_string(),
                vendor: "Test Vendor".to_string(),
                capabilities: vec![
                    ProcessorCapability::MatrixOps,
                    ProcessorCapability::Gradients,
                    ProcessorCapability::Interpolation,
                    ProcessorCapability::Reduction,
                ],
                memory_bytes: Some(8 * 1024 * 1024 * 1024), // 8GB VRAM
                compute_units: Some(4096),
            })
        })
    });

    group.bench_function("create_npu_info", |b| {
        b.iter(|| {
            black_box(ProcessorInfo {
                processor_type: ProcessorType::Npu,
                name: "Test NPU".to_string(),
                vendor: "Test Vendor".to_string(),
                capabilities: vec![
                    ProcessorCapability::Inference,
                    ProcessorCapability::Quantization,
                ],
                memory_bytes: None,
                compute_units: Some(16),
            })
        })
    });

    group.finish();
}

/// Benchmark ProcessorCapability operations
fn bench_processor_capability(c: &mut Criterion) {
    let mut group = c.benchmark_group("processor_capability");

    let capabilities = vec![
        ProcessorCapability::MatrixOps,
        ProcessorCapability::Gradients,
        ProcessorCapability::Interpolation,
        ProcessorCapability::Inference,
        ProcessorCapability::Quantization,
        ProcessorCapability::Reduction,
    ];

    group.bench_function("capability_contains", |b| {
        b.iter(|| {
            black_box(capabilities.contains(&ProcessorCapability::Gradients))
        })
    });

    group.bench_function("capability_clone_vec", |b| {
        b.iter(|| {
            black_box(capabilities.clone())
        })
    });

    group.bench_function("capability_iter_count", |b| {
        b.iter(|| {
            black_box(capabilities.iter().count())
        })
    });

    group.finish();
}

/// Benchmark processor type comparison and matching
fn bench_processor_matching(c: &mut Criterion) {
    let mut group = c.benchmark_group("processor_matching");

    let processor = ProcessorType::Gpu;

    group.bench_function("type_equality", |b| {
        b.iter(|| black_box(processor == ProcessorType::Gpu))
    });

    group.bench_function("type_inequality", |b| {
        b.iter(|| black_box(processor != ProcessorType::Cpu))
    });

    group.bench_function("type_match", |b| {
        b.iter(|| {
            black_box(match processor {
                ProcessorType::Cpu => 0,
                ProcessorType::Gpu => 1,
                ProcessorType::Npu => 2,
                ProcessorType::Fpga => 3,
                ProcessorType::Hybrid => 4,
            })
        })
    });

    let processors = vec![
        ProcessorType::Cpu,
        ProcessorType::Gpu,
        ProcessorType::Npu,
    ];

    group.bench_function("find_in_vec", |b| {
        b.iter(|| {
            black_box(processors.iter().find(|&&p| p == ProcessorType::Gpu))
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_processor_detection,
    bench_processor_listing,
    bench_processor_info,
    bench_processor_capability,
    bench_processor_matching,
);

criterion_main!(benches);
