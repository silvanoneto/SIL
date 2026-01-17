//! Benchmark SIL-ML Forward Pass with proper timing
//! Run: cargo run --example benchmark_sil -p sil-ml --release

use sil_core::state::{SilState, ByteSil};
use std::time::Instant;
use num_complex::Complex;

/// Model ensemble combining multiple architectures
#[derive(Debug, Clone)]
pub struct ModelEnsemble {
    weights: Vec<f64>,
}

impl ModelEnsemble {
    pub fn new() -> Self {
        Self {
            weights: vec![0.33, 0.33, 0.34],
        }
    }

    pub fn forward(&self, input: &SilState) -> SilState {
        let mut result = SilState::vacuum();

        // MLP forward
        let mut mlp_out = SilState::vacuum();
        for i in 0..16 {
            let val = input.get(i);
            let cmplx = val.to_complex();
            let mag = (cmplx.norm()).tanh().abs();
            let scaled = ByteSil::from_complex(Complex::from_polar(mag, 0.0));
            mlp_out = mlp_out.with_layer(i, scaled);
        }

        // Recurrent forward
        let mut rec_out = SilState::neutral();
        for i in 0..16 {
            let input_val = input.get(i);
            let state_val = rec_out.get(i);

            let input_cmplx = input_val.to_complex();
            let state_cmplx = state_val.to_complex();
            let combined_mag = (input_cmplx.norm() + state_cmplx.norm()) / 2.0;

            let scaled = ByteSil::from_complex(Complex::from_polar(combined_mag, 0.0));
            rec_out = rec_out.with_layer(i, scaled);
        }

        // SSM forward
        let mut ssm_out = SilState::vacuum();
        let decay = 0.9;
        for i in 0..16 {
            let val = input.get(i);
            let cmplx = val.to_complex();
            let mag = cmplx.norm() * decay;
            let scaled = ByteSil::from_complex(Complex::from_polar(mag, 0.0));
            ssm_out = ssm_out.with_layer(i, scaled);
        }

        // Weighted ensemble
        for layer in 0..16 {
            let mlp_val = mlp_out.get(layer);
            let rec_val = rec_out.get(layer);
            let ssm_val = ssm_out.get(layer);

            let mlp_cmplx = mlp_val.to_complex();
            let rec_cmplx = rec_val.to_complex();
            let ssm_cmplx = ssm_val.to_complex();

            let weighted_mag = mlp_cmplx.norm() * self.weights[0]
                + rec_cmplx.norm() * self.weights[1]
                + ssm_cmplx.norm() * self.weights[2];

            let scaled = ByteSil::from_complex(Complex::from_polar(weighted_mag, 0.0));
            result = result.with_layer(layer, scaled);
        }

        result
    }
}

fn main() {
    println!("🧠 SIL-ML Forward Pass Benchmark");
    println!("================================\n");

    let ensemble = ModelEnsemble::new();
    let input = SilState::neutral();

    // Warmup
    println!("Warming up...");
    for _ in 0..1000 {
        let _ = ensemble.forward(&input);
    }

    // Actual benchmark
    println!("Running benchmark with 100,000 iterations...\n");
    
    let iterations = 100_000;
    let start = Instant::now();
    
    let mut result = SilState::vacuum();
    for _ in 0..iterations {
        result = ensemble.forward(&input);
    }
    
    let elapsed = start.elapsed();
    let _ = std::hint::black_box(result); // Prevent optimization

    let total_seconds = elapsed.as_secs_f64();
    let per_iteration_us = total_seconds / iterations as f64 * 1_000_000.0;
    let per_iteration_ns = per_iteration_us * 1_000.0;
    let throughput = iterations as f64 / total_seconds;

    println!("Results:");
    println!("  Total time:      {:.6} s", total_seconds);
    println!("  Per iteration:   {:.3} μs", per_iteration_us);
    println!("  Per iteration:   {:.1} ns", per_iteration_ns);
    println!("  Throughput:      {:.0} samples/sec", throughput);
    println!("\nComparison with PyTorch:");
    println!("  PyTorch:         10.336 μs");
    println!("  SIL-ML:          {:.3} μs", per_iteration_us);
    
    if per_iteration_us > 0.0 {
        let ratio = per_iteration_us / 10.336;
        println!("  Ratio:           {:.1}x", ratio);
        if ratio < 1.0 {
            println!("  Winner:          SIL-ML ✅");
        } else {
            println!("  Winner:          PyTorch ✅");
        }
    }
}
