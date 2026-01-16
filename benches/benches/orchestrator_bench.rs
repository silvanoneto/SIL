//! # Orchestrator Benchmarks
//!
//! Measures performance of the orchestration system: tick rate, event throughput,
//! and component coordination.
//!
//! Run: `cargo bench --bench orchestrator_bench`

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use sil_core::prelude::*;
use sil_orchestration::LockFreeEventBus;

/// Benchmark lock-free event bus
fn bench_event_bus(c: &mut Criterion) {
    let mut group = c.benchmark_group("event_bus");

    let bus = LockFreeEventBus::new();

    group.bench_function("emit", |b| {
        b.iter(|| {
            let _ = bus.emit(SilEvent::Ready {
                component: "test".into(),
            });
        })
    });

    // Subscribe and consume
    let sub = bus.subscribe();

    // Pre-fill
    for _ in 0..100 {
        bus.emit(SilEvent::Ready {
            component: "test".into(),
        });
    }

    group.bench_function("consume_100", |b| {
        b.iter(|| {
            let mut count = 0;
            while sub.try_recv().is_some() {
                count += 1;
            }
            black_box(count)
        })
    });

    group.finish();
}

/// Benchmark pipeline stage transitions
fn bench_pipeline(c: &mut Criterion) {
    use sil_orchestration::{Pipeline, PipelineStage};

    let mut group = c.benchmark_group("pipeline");

    group.bench_function("create_default", |b| {
        b.iter(|| {
            black_box(Pipeline::new())
        })
    });

    group.bench_function("create_with_stages", |b| {
        b.iter(|| {
            black_box(Pipeline::with_stages(vec![
                PipelineStage::Sense,
                PipelineStage::Process,
                PipelineStage::Actuate,
            ]))
        })
    });

    let mut pipeline = Pipeline::new();
    pipeline.start();

    group.bench_function("next_stage", |b| {
        b.iter(|| {
            black_box(pipeline.next_stage())
        })
    });

    group.bench_function("current_stage", |b| {
        b.iter(|| {
            black_box(pipeline.current_stage())
        })
    });

    group.bench_function("stage_layers", |b| {
        b.iter(|| {
            black_box(PipelineStage::Sense.layers())
        })
    });

    // Cycle through all 7 stages
    group.bench_function("full_cycle", |b| {
        b.iter(|| {
            let mut p = Pipeline::new();
            p.start();
            for _ in 0..7 {
                p.next_stage();
            }
            black_box(p.cycles())
        })
    });

    group.finish();
}

/// Benchmark orchestrator operations
fn bench_orchestrator(c: &mut Criterion) {
    use sil_orchestration::Orchestrator;

    let mut group = c.benchmark_group("orchestrator");

    group.bench_function("create", |b| {
        b.iter(|| {
            black_box(Orchestrator::new())
        })
    });

    let orch = Orchestrator::new();

    group.bench_function("state_read", |b| {
        b.iter(|| {
            black_box(orch.state())
        })
    });

    group.bench_function("stats", |b| {
        b.iter(|| {
            black_box(orch.stats())
        })
    });

    group.bench_function("uptime", |b| {
        b.iter(|| {
            black_box(orch.uptime())
        })
    });

    group.finish();
}

/// Benchmark batch state operations via orchestrator
fn bench_orchestrator_batch(c: &mut Criterion) {
    use sil_orchestration::Orchestrator;

    let mut group = c.benchmark_group("orchestrator_batch");

    for cycles in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("run_cycles", cycles),
            cycles,
            |b, &n| {
                b.iter(|| {
                    let orch = Orchestrator::new();
                    let _ = orch.run_cycles(n);
                    black_box(orch.cycles())
                })
            }
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_event_bus,
    bench_pipeline,
    bench_orchestrator,
    bench_orchestrator_batch,
);

criterion_main!(benches);
