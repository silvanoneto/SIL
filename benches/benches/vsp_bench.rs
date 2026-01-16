//! # VSP Benchmarks
//!
//! Measures performance of the Virtual SIL Processor: creation, loading, and execution.
//!
//! Run: `cargo bench --bench vsp_bench`

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sil_core::vsp::{Vsp, VspConfig, VspState, VspMemory, SilMode, Assembler};

/// Benchmark VSP creation with different configurations
fn bench_vsp_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("vsp_creation");

    group.bench_function("default_config", |b| {
        b.iter(|| {
            black_box(Vsp::new(VspConfig::default()).unwrap())
        })
    });

    group.bench_function("small_config", |b| {
        let config = VspConfig {
            heap_size: 256,
            stack_size: 64,
            ..VspConfig::default()
        };
        b.iter(|| {
            black_box(Vsp::new(config.clone()).unwrap())
        })
    });

    group.bench_function("large_config", |b| {
        let config = VspConfig {
            heap_size: 262144, // 256K states
            stack_size: 4096,  // 4K frames
            ..VspConfig::default()
        };
        b.iter(|| {
            black_box(Vsp::new(config.clone()).unwrap())
        })
    });

    group.bench_function("with_mode_sil64", |b| {
        let config = VspConfig::default().with_mode(SilMode::Sil64);
        b.iter(|| {
            black_box(Vsp::new(config.clone()).unwrap())
        })
    });

    group.finish();
}

/// Benchmark VSP bytecode loading
fn bench_vsp_loading(c: &mut Criterion) {
    let mut group = c.benchmark_group("vsp_loading");

    // Create simple bytecode: HLT instruction
    let simple_code: Vec<u8> = vec![0x00, 0x00, 0x00, 0x00]; // NOP then HLT
    let simple_data: Vec<u8> = vec![];

    group.bench_function("load_bytes_minimal", |b| {
        b.iter(|| {
            let mut vsp = Vsp::new(VspConfig::default()).unwrap();
            black_box(vsp.load_bytes(&simple_code, &simple_data).unwrap())
        })
    });

    // Larger bytecode
    let large_code: Vec<u8> = vec![0x00; 1024]; // 1KB of NOPs
    let large_data: Vec<u8> = vec![0x00; 4096]; // 4KB data

    group.bench_function("load_bytes_1kb_code", |b| {
        b.iter(|| {
            let mut vsp = Vsp::new(VspConfig::default()).unwrap();
            black_box(vsp.load_bytes(&large_code, &large_data).unwrap())
        })
    });

    let xlarge_code: Vec<u8> = vec![0x00; 10240]; // 10KB
    let xlarge_data: Vec<u8> = vec![0x00; 10240]; // 10KB

    group.bench_function("load_bytes_10kb", |b| {
        b.iter(|| {
            let mut vsp = Vsp::new(VspConfig::default()).unwrap();
            black_box(vsp.load_bytes(&xlarge_code, &xlarge_data).unwrap())
        })
    });

    group.finish();
}

/// Benchmark VSP state and memory access
fn bench_vsp_access(c: &mut Criterion) {
    let mut group = c.benchmark_group("vsp_access");

    let vsp = Vsp::new(VspConfig::default()).unwrap();

    group.bench_function("state_access", |b| {
        b.iter(|| {
            black_box(vsp.state())
        })
    });

    group.bench_function("memory_access", |b| {
        b.iter(|| {
            black_box(vsp.memory())
        })
    });

    group.bench_function("cycles_access", |b| {
        b.iter(|| {
            black_box(vsp.cycles())
        })
    });

    group.bench_function("output_access", |b| {
        b.iter(|| {
            black_box(vsp.output())
        })
    });

    group.finish();
}

/// Benchmark VSP reset
fn bench_vsp_reset(c: &mut Criterion) {
    let mut group = c.benchmark_group("vsp_reset");

    group.bench_function("reset_fresh", |b| {
        b.iter(|| {
            let mut vsp = Vsp::new(VspConfig::default()).unwrap();
            vsp.reset();
            black_box(vsp.cycles())
        })
    });

    group.bench_function("reset_after_load", |b| {
        let code: Vec<u8> = vec![0x00; 1024];
        let data: Vec<u8> = vec![0x00; 1024];
        b.iter(|| {
            let mut vsp = Vsp::new(VspConfig::default()).unwrap();
            vsp.load_bytes(&code, &data).unwrap();
            vsp.reset();
            black_box(vsp.cycles())
        })
    });

    group.finish();
}

/// Benchmark VspState operations
fn bench_vsp_state(c: &mut Criterion) {
    let mut group = c.benchmark_group("vsp_state");

    group.bench_function("state_new", |b| {
        b.iter(|| {
            black_box(VspState::new(SilMode::Sil128))
        })
    });

    group.bench_function("state_new_sil64", |b| {
        b.iter(|| {
            black_box(VspState::new(SilMode::Sil64))
        })
    });

    let state = VspState::new(SilMode::Sil128);

    group.bench_function("register_access", |b| {
        b.iter(|| {
            black_box(state.regs[0])
        })
    });

    group.bench_function("all_registers_access", |b| {
        b.iter(|| {
            for i in 0..16 {
                black_box(state.regs[i]);
            }
        })
    });

    group.finish();
}

/// Benchmark VspMemory operations
fn bench_vsp_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("vsp_memory");

    group.bench_function("memory_new_default", |b| {
        b.iter(|| {
            black_box(VspMemory::new(65536, 1024).unwrap())
        })
    });

    group.bench_function("memory_new_small", |b| {
        b.iter(|| {
            black_box(VspMemory::new(256, 64).unwrap())
        })
    });

    let mut memory = VspMemory::new(65536, 1024).unwrap();
    let code: Vec<u8> = vec![0x00; 1024];
    memory.load_code(&code).unwrap();

    group.bench_function("fetch_instruction", |b| {
        b.iter(|| {
            black_box(memory.fetch(0).unwrap())
        })
    });

    group.finish();
}

/// Benchmark assembler (SIL assembly → bytecode)
fn bench_assembler(c: &mut Criterion) {
    let mut group = c.benchmark_group("assembler");

    // Simple program
    let simple_asm = r#"
        NOP
        HLT
    "#;

    group.bench_function("assemble_simple", |b| {
        b.iter(|| {
            let mut asm = Assembler::new();
            black_box(asm.assemble(simple_asm).unwrap())
        })
    });

    // Medium program with instructions
    let medium_asm = r#"
        MOVI R0, 5
        MOVI R1, 10
        MUL R2, R0, R1
        STORE R2, 0
        HLT
    "#;

    group.bench_function("assemble_medium", |b| {
        b.iter(|| {
            let mut asm = Assembler::new();
            black_box(asm.assemble(medium_asm).unwrap())
        })
    });

    // Larger program
    let large_asm = r#"
        MOVI R0, 1
        MOVI R1, 2
        MOVI R2, 3
        MOVI R3, 4
        MUL R4, R0, R1
        MUL R5, R2, R3
        MUL R6, R4, R5
        DIV R7, R6, R0
        XORL R8, R0, R1
        STORE R8, 0
        LSTATE 0
        SSTATE 1
        HLT
    "#;

    group.bench_function("assemble_large", |b| {
        b.iter(|| {
            let mut asm = Assembler::new();
            black_box(asm.assemble(large_asm).unwrap())
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_vsp_creation,
    bench_vsp_loading,
    bench_vsp_access,
    bench_vsp_reset,
    bench_vsp_state,
    bench_vsp_memory,
    bench_assembler,
);

criterion_main!(benches);
