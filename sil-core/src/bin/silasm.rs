//! # silasm — SIL Assembler
//!
//! ```bash
//! silasm program.sil                  # Assembla para program.silc
//! silasm program.sil -o out.silc      # Output customizado
//! silasm -d program.silc              # Disassembla
//! ```

use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

use sil_core::vsp::{assemble, disassemble};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        return ExitCode::from(1);
    }
    
    let result = if args.iter().any(|a| a == "-d" || a == "--disassemble") {
        cmd_disassemble(&args)
    } else {
        cmd_assemble(&args)
    };
    
    match result {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::FAILURE
        }
    }
}

fn print_usage() {
    println!(r#"
silasm — SIL Assembler

USAGE:
    silasm [options] <file>

OPTIONS:
    -o <file>           Output file
    -d, --disassemble   Disassemble .silc to .sil
    -v, --verbose       Verbose output
    -h, --help          Show help

EXAMPLES:
    silasm program.sil                  # Assembles to program.silc
    silasm program.sil -o out.silc      # Custom output
    silasm -d program.silc              # Disassemble
"#);
}

fn cmd_assemble(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let input = args.iter()
        .skip(1)
        .find(|a| !a.starts_with('-') && a.ends_with(".sil"))
        .ok_or("Missing .sil input file")?;
    
    let output = parse_output(args)
        .unwrap_or_else(|| input.replace(".sil", ".silc"));
    
    let source = fs::read_to_string(input)?;
    let bytecode = assemble(&source)?;
    
    bytecode.save(Path::new(&output))?;
    
    println!("✅ {} -> {}", input, output);
    println!("   Code: {} bytes", bytecode.code.len());
    println!("   Data: {} bytes", bytecode.data.len());
    
    if args.iter().any(|a| a == "-v" || a == "--verbose") {
        println!("\nSymbols:");
        for sym in &bytecode.symbols {
            println!("   {:20} 0x{:04X}", sym.name, sym.address);
        }
    }
    
    Ok(())
}

fn cmd_disassemble(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let input = args.iter()
        .skip(1)
        .find(|a| !a.starts_with('-') && a.ends_with(".silc"))
        .ok_or("Missing .silc input file")?;
    
    let bytecode = fs::read(input)?;
    let source = disassemble(&bytecode);
    
    if let Some(output) = parse_output(args) {
        fs::write(&output, &source)?;
        println!("✅ {} -> {}", input, output);
    } else {
        println!("{}", source);
    }
    
    Ok(())
}

fn parse_output(args: &[String]) -> Option<String> {
    for (i, arg) in args.iter().enumerate() {
        if arg == "-o" || arg == "--output" {
            return args.get(i + 1).cloned();
        }
    }
    None
}
