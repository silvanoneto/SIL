//! # VSP CLI — Virtual Sil Processor Command Line Interface
//!
//! ```bash
//! vsp run program.sil          # Assembla e executa
//! vsp run program.silc         # Executa bytecode
//! vsp asm program.sil          # Assembla para .silc
//! vsp dis program.silc         # Disassembla
//! vsp repl                     # REPL interativo
//! vsp debug program.sil        # Debugger
//! ```

use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

use sil_core::vsp::{
    Vsp, VspConfig, SilMode,
    assemble, disassemble,
    Repl, Debugger,
};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        return ExitCode::from(1);
    }
    
    let result = match args[1].as_str() {
        "run" => cmd_run(&args[2..]),
        "asm" | "assemble" => cmd_assemble(&args[2..]),
        "dis" | "disassemble" => cmd_disassemble(&args[2..]),
        "repl" => cmd_repl(&args[2..]),
        "debug" => cmd_debug(&args[2..]),
        "help" | "--help" | "-h" => {
            print_usage();
            Ok(())
        }
        "version" | "--version" | "-V" => {
            println!("vsp {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            print_usage();
            Err("Unknown command".into())
        }
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
╔═══════════════════════════════════════════════════════════════════════════════╗
║                    VSP — Virtual Sil Processor                                 ║
╠═══════════════════════════════════════════════════════════════════════════════╣
║                                                                                ║
║  USAGE:                                                                        ║
║      vsp <command> [options] [file]                                           ║
║                                                                                ║
║  COMMANDS:                                                                     ║
║      run <file>           Assemble and run .sil, or run .silc directly        ║
║      asm <file>           Assemble .sil to .silc bytecode                     ║
║      dis <file>           Disassemble .silc to .sil                           ║
║      repl                 Interactive REPL                                     ║
║      debug <file>         Debug with breakpoints                               ║
║                                                                                ║
║  OPTIONS:                                                                      ║
║      --mode <MODE>        SIL mode: SIL-8, SIL-16, SIL-32, SIL-64, SIL-128    ║
║      -i, --input <file>   Input file (loaded into sensors)                    ║
║      -o, --output <file>  Output file (receives actuator data)                ║
║      -v, --verbose        Verbose output                                       ║
║                                                                                ║
║  EXAMPLES:                                                                     ║
║      vsp run hello.sil                                                         ║
║      vsp run compress.sil -i data.txt -o data.silc                            ║
║      vsp asm program.sil -o program.silc                                       ║
║      vsp repl --mode SIL-128                                                   ║
║      vsp debug program.sil                                                     ║
║                                                                                ║
╚═══════════════════════════════════════════════════════════════════════════════╝
"#);
}

fn cmd_run(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        return Err("Missing file argument".into());
    }
    
    let file = &args[0];
    let path = Path::new(file);
    let mode = parse_mode(args);
    let input_file = parse_input(args);
    let output_file = parse_output(args);
    let verbose = args.iter().any(|a| a == "-v" || a == "--verbose");
    
    // Configurar VM
    let config = VspConfig {
        default_mode: mode,
        debug: verbose,
        ..Default::default()
    };
    
    let mut vsp = Vsp::new(config)?;
    
    // Carregar arquivo de input nos sensores (se especificado)
    if let Some(ref input_path) = input_file {
        let input_data = fs::read(input_path)?;
        println!("📥 Loading input: {} ({} bytes)", input_path, input_data.len());
        
        // Carregar dados nos sensores da VM
        for (i, &byte) in input_data.iter().enumerate() {
            let sensor_id = i % 16;  // Rotaciona entre os 16 sensores
            vsp.memory_mut().set_sensor(sensor_id, sil_core::state::ByteSil::from_u8(byte))?;
        }
        
        // Também guardar no buffer de I/O para SENSE ler sequencialmente
        vsp.memory_mut().load_input(&input_data)?;
    }
    
    // Carregar programa
    if path.extension().map(|e| e == "silc").unwrap_or(false) {
        // Bytecode direto
        vsp.load_silc(path)?;
    } else {
        // Assemblar primeiro
        let source = fs::read_to_string(path)?;
        let bytecode = assemble(&source)?;
        vsp.load_bytes(&bytecode.code, &bytecode.data)?;
    }
    
    // Executar
    println!("🚀 Running {}...", file);
    println!("   Processor: {}", vsp.current_processor());
    let result = vsp.run()?;
    
    // Coletar output
    let output = vsp.output();
    let output_bytes: Vec<u8> = output.iter().map(|b| b.to_u8()).collect();
    
    // Salvar output em arquivo (se especificado)
    if let Some(ref output_path) = output_file {
        fs::write(output_path, &output_bytes)?;
        println!("\n💾 Output saved: {} ({} bytes)", output_path, output_bytes.len());
    }
    
    // Mostrar output no terminal
    if !output.is_empty() {
        println!("\n📤 Output ({} bytes):", output.len());
        
        // Mostrar em hex (limitado a 64 bytes se não verbose)
        let show_limit = if verbose { output.len() } else { 64.min(output.len()) };
        print!("   Hex: ");
        for (i, b) in output.iter().take(show_limit).enumerate() {
            if i > 0 && i % 16 == 0 {
                print!("\n        ");
            }
            print!("{:02X} ", b.to_u8());
        }
        if output.len() > show_limit {
            print!("... ({} more)", output.len() - show_limit);
        }
        println!();
        
        // Mostrar como texto se imprimível
        let text: String = output.iter()
            .take(show_limit)
            .map(|b| {
                let c = b.to_u8();
                if c >= 0x20 && c < 0x7F { c as char } else { '.' }
            })
            .collect();
        if text.chars().any(|c| c != '.') {
            println!("   Text: {}{}", text, if output.len() > show_limit { "..." } else { "" });
        }
    }
    
    // Mostrar resultado
    println!("\n✅ Completed in {} cycles", vsp.cycles());
    println!("   Processor: {}", vsp.current_processor());
    if verbose {
        println!("   State: {:?}", result);
    }
    
    Ok(())
}

/// Parse --input ou -i
fn parse_input(args: &[String]) -> Option<String> {
    for i in 0..args.len() {
        if (args[i] == "--input" || args[i] == "-i") && i + 1 < args.len() {
            return Some(args[i + 1].clone());
        }
    }
    None
}

fn cmd_assemble(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        return Err("Missing file argument".into());
    }
    
    let input = &args[0];
    let output = parse_output(args)
        .unwrap_or_else(|| input.replace(".sil", ".silc"));
    
    let source = fs::read_to_string(input)?;
    let bytecode = assemble(&source)?;
    
    bytecode.save(Path::new(&output))?;
    
    println!("✅ Assembled {} -> {}", input, output);
    println!("   Code: {} bytes", bytecode.code.len());
    println!("   Data: {} bytes", bytecode.data.len());
    
    Ok(())
}

fn cmd_disassemble(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        return Err("Missing file argument".into());
    }
    
    let input = &args[0];
    let bytes = fs::read(input)?;
    
    // Se é .silc, extrair código do container
    let code = if input.ends_with(".silc") {
        use sil_core::vsp::SilcFile;
        let silc = SilcFile::from_bytes(&bytes)?;
        silc.code
    } else {
        bytes
    };
    
    let source = disassemble(&code);
    
    if let Some(output) = parse_output(args) {
        fs::write(&output, &source)?;
        println!("✅ Disassembled {} -> {}", input, output);
    } else {
        println!("{}", source);
    }
    
    Ok(())
}

fn cmd_repl(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mode = parse_mode(args);
    let mut repl = Repl::new().with_mode(mode);
    repl.run()?;
    Ok(())
}

fn cmd_debug(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        return Err("Missing file argument".into());
    }
    
    let file = &args[0];
    let mode = parse_mode(args);
    
    let source = fs::read_to_string(file)?;
    let bytecode = assemble(&source)?;
    
    let config = VspConfig {
        default_mode: mode,
        debug: true,
        ..Default::default()
    };
    
    let mut vsp = Vsp::new(config)?;
    vsp.load_bytes(&bytecode.code, &bytecode.data)?;
    
    let _debugger = Debugger::new(vsp);
    
    // Debug mode - show initial state
    println!("🐛 Debug mode for {}", file);
    println!("   Use the API to set breakpoints and step through");
    println!("   Debugger created with {} bytes of code", bytecode.code.len());
    
    Ok(())
}

fn parse_mode(args: &[String]) -> SilMode {
    for (i, arg) in args.iter().enumerate() {
        if arg == "--mode" {
            if let Some(mode_str) = args.get(i + 1) {
                return match mode_str.to_uppercase().as_str() {
                    "SIL-8" => SilMode::Sil8,
                    "SIL-16" => SilMode::Sil16,
                    "SIL-32" => SilMode::Sil32,
                    "SIL-64" => SilMode::Sil64,
                    "SIL-128" | _ => SilMode::Sil128,
                };
            }
        }
    }
    SilMode::Sil128
}

fn parse_output(args: &[String]) -> Option<String> {
    for (i, arg) in args.iter().enumerate() {
        if arg == "-o" || arg == "--output" {
            return args.get(i + 1).cloned();
        }
    }
    None
}
