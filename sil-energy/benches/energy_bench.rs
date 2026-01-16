//! Benchmarks para medição de energia

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sil_energy::{
    EnergyMeter, EnergyModel, CpuEnergyModel, GpuEnergyModel,
    EnergySampler, SamplingMode,
};
use std::time::Duration;

fn bench_cpu_model_estimate(c: &mut Criterion) {
    let model = CpuEnergyModel::detect();

    c.bench_function("cpu_model_estimate_1k_ops", |b| {
        b.iter(|| {
            model.estimate_joules(
                black_box(Duration::from_millis(10)),
                black_box(1000),
                black_box(0.5),
            )
        })
    });

    c.bench_function("cpu_model_estimate_100k_ops", |b| {
        b.iter(|| {
            model.estimate_joules(
                black_box(Duration::from_millis(100)),
                black_box(100_000),
                black_box(0.8),
            )
        })
    });
}

fn bench_gpu_model_estimate(c: &mut Criterion) {
    let model = GpuEnergyModel::integrated();

    c.bench_function("gpu_model_estimate_1k_ops", |b| {
        b.iter(|| {
            model.estimate_joules(
                black_box(Duration::from_millis(10)),
                black_box(1000),
                black_box(0.7),
            )
        })
    });
}

fn bench_energy_meter(c: &mut Criterion) {
    let mut meter = EnergyMeter::auto_detect();

    c.bench_function("meter_begin_end_measurement", |b| {
        b.iter(|| {
            meter.begin_measurement().unwrap();
            black_box(42); // Simula trabalho
            meter.end_measurement(black_box(100)).unwrap()
        })
    });

    c.bench_function("meter_measure_closure", |b| {
        b.iter(|| {
            meter.measure(black_box(100), || {
                black_box(42)
            }).unwrap()
        })
    });
}

fn bench_energy_sampler(c: &mut Criterion) {
    let model = Box::new(CpuEnergyModel::detect());
    let mut sampler = EnergySampler::new(model, SamplingMode::PerOperation(1000));

    c.bench_function("sampler_record_operations", |b| {
        b.iter(|| {
            sampler.record_operations(black_box(100))
        })
    });
}

criterion_group!(
    benches,
    bench_cpu_model_estimate,
    bench_gpu_model_estimate,
    bench_energy_meter,
    bench_energy_sampler,
);
criterion_main!(benches);
