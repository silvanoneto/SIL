//! VSP AOT Compiler CLI
//!
//! Compila bytecode VSP para código nativo.

use sil_core::vsp::bytecode::SilcFile;
use sil_core::vsp::aot::{AotCompiler, AotCache, OptLevel};
use std::path::PathBuf;
use std::process;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        print_usage(&args[0]);
        process::exit(1);
    }
    
    match args[1].as_str() {
        "compile" => cmd_compile(&args[2..]),
        "cache" => cmd_cache(&args[2..]),
        "info" => cmd_info(&args[2..]),
        "--help" | "-h" => {
            print_usage(&args[0]);
            process::exit(0);
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            print_usage(&args[0]);
            process::exit(1);
        }
    }
}

fn cmd_compile(args: &[String]) {
    if args.is_empty() {
        eprintln!("Error: Missing input file");
        eprintln!("Usage: vsp-aot compile <input.vsp> [options]");
        process::exit(1);
    }
    
    let input = PathBuf::from(&args[0]);
    let mut output = input.with_extension("o");
    let mut opt_level = OptLevel::Speed;
    let mut cache_enabled = false;
    
    // Parse options
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--output" => {
                if i + 1 >= args.len() {
                    eprintln!("Error: Missing output path");
                    process::exit(1);
                }
                output = PathBuf::from(&args[i + 1]);
                i += 2;
            }
            "-O0" => {
                opt_level = OptLevel::None;
                i += 1;
            }
            "-O2" => {
                opt_level = OptLevel::Speed;
                i += 1;
            }
            "-O3" | "-Os" => {
                opt_level = OptLevel::SpeedAndSize;
                i += 1;
            }
            "--cache" => {
                cache_enabled = true;
                i += 1;
            }
            _ => {
                eprintln!("Unknown option: {}", args[i]);
                process::exit(1);
            }
        }
    }
    
    // Ler bytecode
    println!("📖 Reading bytecode from {}...", input.display());
    let bytecode_data = std::fs::read(&input).unwrap_or_else(|e| {
        eprintln!("Error reading file: {}", e);
        process::exit(1);
    });
    
    let bytecode = SilcFile::from_bytes(&bytecode_data).unwrap_or_else(|e| {
        eprintln!("Error parsing bytecode: {:?}", e);
        process::exit(1);
    });
    
    println!("   {} bytes", bytecode_data.len());
    
    // Compilar
    println!("🔧 Compiling with optimization level {:?}...", opt_level);
    
    let compiler = AotCompiler::new()
        .with_opt_level(opt_level);
    
    let name = input.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("main");
    
    let compilation = compiler.compile(name, bytecode).unwrap_or_else(|e| {
        eprintln!("Compilation error: {:?}", e);
        process::exit(1);
    });
    
    println!("   Code size: {} bytes", compilation.metadata.code_size);
    
    // Salvar
    if cache_enabled {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("sil-vsp-aot");
        
        let mut cache = AotCache::new(&cache_dir).unwrap_or_else(|e| {
            eprintln!("Cache error: {:?}", e);
            process::exit(1);
        });
        
        let cached_path = cache.put(&compilation).unwrap_or_else(|e| {
            eprintln!("Cache write error: {:?}", e);
            process::exit(1);
        });
        
        println!("💾 Cached at {}", cached_path.display());
    } else {
        compiler.save(&compilation, &output).unwrap_or_else(|e| {
            eprintln!("Save error: {:?}", e);
            process::exit(1);
        });
        
        println!("💾 Saved to {}", output.display());
    }
    
    println!("✅ Compilation successful!");
}

fn cmd_cache(args: &[String]) {
    if args.is_empty() {
        eprintln!("Error: Missing cache command");
        eprintln!("Usage: vsp-aot cache <list|clear|stats>");
        process::exit(1);
    }
    
    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("sil-vsp-aot");
    
    let mut cache = AotCache::new(&cache_dir).unwrap_or_else(|e| {
        eprintln!("Cache error: {:?}", e);
        process::exit(1);
    });
    
    match args[0].as_str() {
        "list" => {
            let stats = cache.stats();
            println!("📦 Cache: {}", cache_dir.display());
            println!("   {} entries", stats.num_entries);
            println!("   {} bytes", stats.total_size_bytes);
        }
        "clear" => {
            cache.clear().unwrap_or_else(|e| {
                eprintln!("Clear error: {:?}", e);
                process::exit(1);
            });
            println!("✅ Cache cleared");
        }
        "stats" => {
            let stats = cache.stats();
            println!("Cache Statistics:");
            println!("  Entries: {}", stats.num_entries);
            println!("  Total size: {} bytes ({:.2} KB)", 
                     stats.total_size_bytes,
                     stats.total_size_bytes as f64 / 1024.0);
        }
        _ => {
            eprintln!("Unknown cache command: {}", args[0]);
            process::exit(1);
        }
    }
}

fn cmd_info(args: &[String]) {
    if args.is_empty() {
        eprintln!("Error: Missing file");
        eprintln!("Usage: vsp-aot info <file.o>");
        process::exit(1);
    }
    
    let path = PathBuf::from(&args[0]);
    let meta_path = path.with_extension("meta");
    
    if !meta_path.exists() {
        eprintln!("Error: Metadata file not found");
        process::exit(1);
    }
    
    let meta_json = std::fs::read_to_string(&meta_path).unwrap_or_else(|e| {
        eprintln!("Error reading metadata: {}", e);
        process::exit(1);
    });
    
    println!("📄 Compilation Info: {}", path.display());
    println!("{}", meta_json);
}

fn print_usage(program: &str) {
    println!("VSP AOT Compiler");
    println!();
    println!("USAGE:");
    println!("    {} <COMMAND> [OPTIONS]", program);
    println!();
    println!("COMMANDS:");
    println!("    compile <input.vsp>    Compile bytecode to native code");
    println!("    cache <list|clear>     Manage compilation cache");
    println!("    info <file.o>          Show compilation info");
    println!();
    println!("COMPILE OPTIONS:");
    println!("    -o, --output <file>    Output file path");
    println!("    -O0                    No optimization (debug)");
    println!("    -O2                    Speed optimization (default)");
    println!("    -O3, -Os               Speed and size optimization");
    println!("    --cache                Use compilation cache");
    println!();
    println!("EXAMPLES:");
    println!("    {} compile program.vsp -O3", program);
    println!("    {} compile program.vsp -o output.o --cache", program);
    println!("    {} cache stats", program);
}
