//! # Emergence Module
//!
//! Emergent patterns: Hebbian learning, Kuramoto oscillators, TDA.

// mod mimetics;
// mod tda;
// mod signal;

// pub use mimetics::*;
// pub use tda::*;
// pub use signal::*;

use sil_core::state::{SilState, NUM_LAYERS};
use crate::core::tensor::{magnitude, phase, from_mag_phase};
use std::f64::consts::PI;

/// Hebbian learning: Δw = η * x * y
pub fn hebbian_update(
    weights: &SilState,
    pre: &SilState,  // Pre-synaptic
    post: &SilState, // Post-synaptic
    eta: f64,        // Learning rate
) -> SilState {
    let mut result = *weights;
    
    for i in 0..NUM_LAYERS {
        let w = magnitude(&weights.get(i));
        let x = magnitude(&pre.get(i));
        let y = magnitude(&post.get(i));
        
        let delta = eta * x * y;
        let w_new = w + delta;
        
        result = result.with_layer(i, from_mag_phase(w_new, phase(&weights.get(i))));
    }
    
    result
}

/// Oja's rule: Δw = η * y * (x - y * w)
/// Normalized Hebbian that prevents unbounded growth
pub fn oja_update(
    weights: &SilState,
    pre: &SilState,
    post: &SilState,
    eta: f64,
) -> SilState {
    let mut result = *weights;
    
    for i in 0..NUM_LAYERS {
        let w = magnitude(&weights.get(i));
        let x = magnitude(&pre.get(i));
        let y = magnitude(&post.get(i));
        
        let delta = eta * y * (x - y * w);
        let w_new = w + delta;
        
        result = result.with_layer(i, from_mag_phase(w_new, phase(&weights.get(i))));
    }
    
    result
}

/// BCM (Bienenstock-Cooper-Munro) rule
pub fn bcm_update(
    weights: &SilState,
    pre: &SilState,
    post: &SilState,
    theta: f64,  // Modification threshold
    eta: f64,
) -> SilState {
    let mut result = *weights;
    
    for i in 0..NUM_LAYERS {
        let w = magnitude(&weights.get(i));
        let x = magnitude(&pre.get(i));
        let y = magnitude(&post.get(i));
        
        // BCM: Δw = η * y * (y - θ) * x
        let delta = eta * y * (y - theta) * x;
        let w_new = w + delta;
        
        result = result.with_layer(i, from_mag_phase(w_new, phase(&weights.get(i))));
    }
    
    result
}

/// Kuramoto oscillator phase coupling
pub fn kuramoto_step(
    phases: &SilState,
    natural_freqs: &SilState,
    coupling: f64,
    dt: f64,
) -> SilState {
    let mut new_phases = SilState::vacuum();
    
    for i in 0..NUM_LAYERS {
        let phi_i = phase(&phases.get(i));
        let omega_i = magnitude(&natural_freqs.get(i));
        
        // Sum of sin(φj - φi) over all j
        let mut coupling_sum = 0.0;
        for j in 0..NUM_LAYERS {
            if i != j {
                let phi_j = phase(&phases.get(j));
                coupling_sum += (phi_j - phi_i).sin();
            }
        }
        
        // dφi/dt = ωi + (K/N) * Σ sin(φj - φi)
        let dphi = omega_i + (coupling / NUM_LAYERS as f64) * coupling_sum;
        let phi_new = phi_i + dphi * dt;
        
        new_phases = new_phases.with_layer(i, from_mag_phase(1.0, phi_new));
    }
    
    new_phases
}

/// Kuramoto order parameter (synchronization measure)
pub fn kuramoto_order_parameter(phases: &SilState) -> f64 {
    let mut sum_cos = 0.0;
    let mut sum_sin = 0.0;
    
    for i in 0..NUM_LAYERS {
        let phi = phase(&phases.get(i));
        sum_cos += phi.cos();
        sum_sin += phi.sin();
    }
    
    let r = (sum_cos * sum_cos + sum_sin * sum_sin).sqrt() / NUM_LAYERS as f64;
    r
}

/// Discrete Fourier Transform (simplified for 16 elements)
pub fn dft(state: &SilState) -> SilState {
    let mut result = SilState::vacuum();
    let n = NUM_LAYERS as f64;
    
    for k in 0..NUM_LAYERS {
        let mut re = 0.0;
        let mut im = 0.0;
        
        for j in 0..NUM_LAYERS {
            let x = magnitude(&state.get(j));
            let angle = -2.0 * PI * (k as f64) * (j as f64) / n;
            re += x * angle.cos();
            im += x * angle.sin();
        }
        
        let mag = (re * re + im * im).sqrt();
        let phase = im.atan2(re);
        
        result = result.with_layer(k, from_mag_phase(mag, phase));
    }
    
    result
}

/// Inverse Discrete Fourier Transform
pub fn idft(state: &SilState) -> SilState {
    let mut result = SilState::vacuum();
    let n = NUM_LAYERS as f64;
    
    for k in 0..NUM_LAYERS {
        let mut re = 0.0;
        let mut im = 0.0;
        
        for j in 0..NUM_LAYERS {
            let x_mag = magnitude(&state.get(j));
            let x_phase = phase(&state.get(j));
            let angle = 2.0 * PI * (k as f64) * (j as f64) / n;
            
            // Convert to complex, multiply by exp(i*angle)
            let x_re = x_mag * x_phase.cos();
            let x_im = x_mag * x_phase.sin();
            let rot_re = angle.cos();
            let rot_im = angle.sin();
            
            re += x_re * rot_re - x_im * rot_im;
            im += x_re * rot_im + x_im * rot_re;
        }
        
        let mag = (re * re + im * im).sqrt() / n;
        let phase = im.atan2(re);
        
        result = result.with_layer(k, from_mag_phase(mag, phase));
    }
    
    result
}

/// Low-pass filter (zero high frequencies)
pub fn lowpass_filter(state: &SilState, cutoff: usize) -> SilState {
    let freq = dft(state);
    let mut filtered = freq;
    
    // Zero out frequencies above cutoff
    for i in cutoff..NUM_LAYERS {
        filtered = filtered.with_layer(i, from_mag_phase(0.0, 0.0));
    }
    
    idft(&filtered)
}

/// High-pass filter
pub fn highpass_filter(state: &SilState, cutoff: usize) -> SilState {
    let freq = dft(state);
    let mut filtered = freq;
    
    // Zero out frequencies below cutoff
    for i in 0..cutoff {
        filtered = filtered.with_layer(i, from_mag_phase(0.0, 0.0));
    }
    
    idft(&filtered)
}

/// Topological Data Analysis: 0-dimensional persistence (connected components)
pub fn persistence_0d(state: &SilState, threshold: f64) -> Vec<(f64, f64)> {
    let mut mags: Vec<(usize, f64)> = (0..NUM_LAYERS)
        .map(|i| (i, magnitude(&state.get(i))))
        .collect();
    
    // Sort by magnitude (descending)
    mags.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    
    let mut persistence = Vec::new();
    let mut components: Vec<Option<f64>> = vec![None; NUM_LAYERS];
    
    for (idx, birth) in mags {
        // Check if neighbors are in existing components
        let left_comp = if idx > 0 { components[idx - 1] } else { None };
        let right_comp = if idx < NUM_LAYERS - 1 { components[idx + 1] } else { None };
        
        match (left_comp, right_comp) {
            (None, None) => {
                // New component born
                components[idx] = Some(birth);
            }
            (Some(b), None) | (None, Some(b)) => {
                // Join existing component
                components[idx] = Some(b);
            }
            (Some(b1), Some(b2)) => {
                // Merge components - older one survives, younger dies
                let (survivor, death_birth) = if b1 >= b2 { (b1, b2) } else { (b2, b1) };
                if (birth - death_birth).abs() > threshold {
                    persistence.push((death_birth, birth));
                }
                components[idx] = Some(survivor);
            }
        }
    }
    
    persistence
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hebbian() {
        let weights = SilState::neutral();
        let pre = SilState::neutral();
        let post = SilState::neutral();
        
        let updated = hebbian_update(&weights, &pre, &post, 0.1);
        
        // Weights should increase
        for i in 0..NUM_LAYERS {
            assert!(magnitude(&updated.get(i)) >= magnitude(&weights.get(i)) - 0.1);
        }
    }

    #[test]
    fn test_kuramoto_order() {
        // Synchronized phases should have high order
        let mut sync = SilState::vacuum();
        for i in 0..NUM_LAYERS {
            sync = sync.with_layer(i, from_mag_phase(1.0, 0.0)); // All same phase
        }
        
        let order = kuramoto_order_parameter(&sync);
        assert!(order > 0.9);
    }

    #[test]
    fn test_dft_idft() {
        let state = SilState::neutral();
        let freq = dft(&state);
        let recovered = idft(&freq);
        
        // Should approximately recover original
        for i in 0..NUM_LAYERS {
            let orig = magnitude(&state.get(i));
            let rec = magnitude(&recovered.get(i));
            assert!((orig - rec).abs() < 0.5, "Layer {}: {} vs {}", i, orig, rec);
        }
    }
}
