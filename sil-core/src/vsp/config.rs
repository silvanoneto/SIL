//! Environment configuration loading from .env files
//!
//! Loads configuration values from .env or environment variables.
//! Used for VSP interpreter settings and energy model parameters.

use std::env;
use once_cell::sync::Lazy;

// Automatically load .env when config module is accessed
static DOTENV_INIT: Lazy<()> = Lazy::new(|| {
    let _ = dotenv::dotenv();
});

/// Ensure environment is loaded
#[inline]
fn ensure_loaded() {
    let _ = &*DOTENV_INIT;
}

/// Load carbon intensity from environment
/// Default: 0.075 gCO2e/kWh (Brazil grid, hydroelectric heavy)
pub fn carbon_intensity_brazil() -> f64 {
    ensure_loaded();
    env::var("CARBON_INTENSITY_BRAZIL")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.075)
}

/// Load world average carbon intensity from environment
/// Default: 0.475 gCO2e/kWh (global grid average)
pub fn carbon_intensity_world() -> f64 {
    ensure_loaded();
    env::var("CARBON_INTENSITY_WORLD")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.475)
}

/// Load CPU baseline power from environment
/// Default: 15.0 W
pub fn cpu_baseline_power_w() -> f64 {
    ensure_loaded();
    env::var("CPU_BASELINE_POWER_W")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(15.0)
}

/// Load CPU frequency from environment
/// Default: 3.0 GHz
pub fn cpu_frequency_ghz() -> f64 {
    ensure_loaded();
    env::var("CPU_FREQUENCY_GHZ")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3.0)
}

/// Load maximum energy budget from environment
/// Default: 1.0 J
pub fn max_energy_budget_j() -> f64 {
    ensure_loaded();
    env::var("MAX_ENERGY_BUDGET_J")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1.0)
}

/// Load latency budget from environment
/// Default: 100000 µs (100 ms)
pub fn latency_budget_us() -> u64 {
    ensure_loaded();
    env::var("LATENCY_BUDGET_US")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(100000)
}

/// Cached values
pub static CARBON_INTENSITY_BRAZIL: Lazy<f64> = Lazy::new(carbon_intensity_brazil);
pub static CARBON_INTENSITY_WORLD: Lazy<f64> = Lazy::new(carbon_intensity_world);
pub static CPU_BASELINE_POWER_W: Lazy<f64> = Lazy::new(cpu_baseline_power_w);
pub static CPU_FREQUENCY_GHZ: Lazy<f64> = Lazy::new(cpu_frequency_ghz);
pub static MAX_ENERGY_BUDGET_J: Lazy<f64> = Lazy::new(max_energy_budget_j);
pub static LATENCY_BUDGET_US: Lazy<u64> = Lazy::new(latency_budget_us);
