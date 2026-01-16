//! CLI tool for running LIS programs

use lis_runtime::{LisRuntime, RuntimeConfig};
use std::process;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: lis-run <file.lis> [--debug]");
        process::exit(1);
    }

    let file_path = &args[1];
    let debug = args.iter().any(|arg| arg == "--debug" || arg == "-d");

    let config = RuntimeConfig {
        debug,
        ..Default::default()
    };

    let mut runtime = LisRuntime::new(config);

    match runtime.run_file(file_path) {
        Ok(state) => {
            if !debug {
                println!("✅ Program executed successfully");
                println!("\n📊 Final state:");
                for layer in 0..16 {
                    let byte = state.get(layer);
                    if byte.rho != 0 || byte.theta != 0 {
                        println!("   L{:X}: ρ={:+3}, θ={:3}", layer, byte.rho, byte.theta);
                    }
                }
            }
            process::exit(0);
        }
        Err(e) => {
            eprintln!("❌ Runtime error: {}", e);
            process::exit(1);
        }
    }
}
