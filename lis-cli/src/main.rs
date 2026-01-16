//! LIS - Language for Intelligent Systems
//! Command-line interface for compiling and running LIS programs

use clap::{Parser, Subcommand};
use colored::*;
use lis_core::{compile, Manifest, ModuleResolver};
use lis_core::manifest::create_manifest;
use sil_core::vsp::{Vsp, VspConfig, assemble};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "lis")]
#[command(author = "SIL Contributors")]
#[command(version = "2026.1.16")]
#[command(about = "LIS - Language for Intelligent Systems", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new LIS project
    New {
        /// Project name
        #[arg(value_name = "NAME")]
        name: String,

        /// Author name and email (e.g., "Name <email@example.com>")
        #[arg(short, long)]
        author: Option<String>,
    },

    /// Initialize a LIS project in the current directory
    Init {
        /// Author name and email (e.g., "Name <email@example.com>")
        #[arg(short, long)]
        author: Option<String>,
    },

    /// Compile LIS source to VSP assembly
    Compile {
        /// Input LIS file (.lis) or directory with lis.toml
        #[arg(value_name = "INPUT")]
        input: Option<PathBuf>,

        /// Output assembly file (.sil)
        #[arg(short, long, value_name = "OUTPUT")]
        output: Option<PathBuf>,

        /// Print assembly to stdout
        #[arg(short, long)]
        print: bool,
    },

    /// Build LIS project to VSP bytecode
    Build {
        /// Input LIS file (.lis) or directory with lis.toml
        #[arg(value_name = "INPUT")]
        input: Option<PathBuf>,

        /// Output bytecode file (.silc)
        #[arg(short, long, value_name = "OUTPUT")]
        output: Option<PathBuf>,

        /// Build in release mode (optimized)
        #[arg(short, long)]
        release: bool,
    },

    /// Run LIS program directly
    Run {
        /// Input LIS file (.lis) or directory with lis.toml
        #[arg(value_name = "INPUT")]
        input: Option<PathBuf>,
    },

    /// Check LIS syntax without compiling
    Check {
        /// Input LIS file (.lis) or directory with lis.toml
        #[arg(value_name = "INPUT")]
        input: Option<PathBuf>,
    },

    /// Show information about LIS
    Info,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::New { name, author } => {
            if let Err(e) = new_command(&name, author.as_deref()) {
                eprintln!("{} {}", "error:".red().bold(), e);
                std::process::exit(1);
            }
        }

        Commands::Init { author } => {
            if let Err(e) = init_command(author.as_deref()) {
                eprintln!("{} {}", "error:".red().bold(), e);
                std::process::exit(1);
            }
        }

        Commands::Compile {
            input,
            output,
            print,
        } => {
            if let Err(e) = compile_command(input.as_deref(), output.as_deref(), print) {
                eprintln!("{} {}", "error:".red().bold(), e);
                std::process::exit(1);
            }
        }

        Commands::Build { input, output, release } => {
            if let Err(e) = build_command(input.as_deref(), output.as_deref(), release) {
                eprintln!("{} {}", "error:".red().bold(), e);
                std::process::exit(1);
            }
        }

        Commands::Run { input } => {
            if let Err(e) = run_command(input.as_deref()) {
                eprintln!("{} {}", "error:".red().bold(), e);
                std::process::exit(1);
            }
        }

        Commands::Check { input } => {
            if let Err(e) = check_command(input.as_deref()) {
                eprintln!("{} {}", "error:".red().bold(), e);
                std::process::exit(1);
            }
        }

        Commands::Info => {
            print_info();
        }
    }
}

// ============================================================================
// Project scaffolding commands
// ============================================================================

fn new_command(name: &str, author: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let project_dir = PathBuf::from(name);

    if project_dir.exists() {
        return Err(format!("Directory '{}' already exists", name).into());
    }

    println!("{} new project '{}'", "Creating".green().bold(), name.cyan());

    // Create project structure
    fs::create_dir_all(project_dir.join("src"))?;

    // Create lis.toml
    let authors = author.map(|a| vec![a.to_string()]).unwrap_or_default();
    let manifest = create_manifest(name, authors);
    let manifest_toml = manifest.to_string()?;
    fs::write(project_dir.join("lis.toml"), manifest_toml)?;

    // Create src/main.lis with hello world
    let main_source = r#"// Main entry point for the LIS program

fn main() {
    print_string("Hello from LIS!");
}
"#;
    fs::write(project_dir.join("src/main.lis"), main_source)?;

    // Create .gitignore
    let gitignore = r#"/target
*.sil
*.silc
"#;
    fs::write(project_dir.join(".gitignore"), gitignore)?;

    println!("{} {}", "   Created".green().bold(), "lis.toml".cyan());
    println!("{} {}", "   Created".green().bold(), "src/main.lis".cyan());
    println!("{} {}", "   Created".green().bold(), ".gitignore".cyan());
    println!();
    println!("{}", "To get started, run:".bold());
    println!("  cd {}", name);
    println!("  lis run");

    Ok(())
}

fn init_command(author: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = std::env::current_dir()?;
    let manifest_path = current_dir.join("lis.toml");

    if manifest_path.exists() {
        return Err("lis.toml already exists in this directory".into());
    }

    let name = current_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("my-project");

    println!("{} project in '{}'", "Initializing".green().bold(), current_dir.display().to_string().cyan());

    // Create src directory if it doesn't exist
    let src_dir = current_dir.join("src");
    if !src_dir.exists() {
        fs::create_dir_all(&src_dir)?;
    }

    // Create lis.toml
    let authors = author.map(|a| vec![a.to_string()]).unwrap_or_default();
    let manifest = create_manifest(name, authors);
    let manifest_toml = manifest.to_string()?;
    fs::write(&manifest_path, manifest_toml)?;

    // Create src/main.lis if it doesn't exist
    let main_path = src_dir.join("main.lis");
    if !main_path.exists() {
        let main_source = r#"// Main entry point for the LIS program

fn main() {
    print_string("Hello from LIS!");
}
"#;
        fs::write(&main_path, main_source)?;
        println!("{} {}", "   Created".green().bold(), "src/main.lis".cyan());
    }

    println!("{} {}", "   Created".green().bold(), "lis.toml".cyan());
    println!();
    println!("{}", "Project initialized. Run 'lis run' to execute.".bold());

    Ok(())
}

// ============================================================================
// Compilation commands
// ============================================================================

/// Resolve input path: if None, look for lis.toml in current dir
/// If it's a .lis file, return it directly
/// If it's a directory or has lis.toml, use project mode
fn resolve_input(input: Option<&Path>) -> Result<InputMode, Box<dyn std::error::Error>> {
    match input {
        Some(path) if path.extension().map(|e| e == "lis").unwrap_or(false) => {
            // Direct .lis file
            Ok(InputMode::SingleFile(path.to_path_buf()))
        }
        Some(path) => {
            // Directory or project path
            let manifest_path = if path.is_dir() {
                path.join("lis.toml")
            } else if path.file_name().map(|n| n == "lis.toml").unwrap_or(false) {
                path.to_path_buf()
            } else {
                return Err(format!("Unknown input type: {}", path.display()).into());
            };

            if manifest_path.exists() {
                let project_root = manifest_path.parent().unwrap().to_path_buf();
                Ok(InputMode::Project(project_root))
            } else {
                Err(format!("No lis.toml found in {}", path.display()).into())
            }
        }
        None => {
            // Look for lis.toml in current directory
            let current_dir = std::env::current_dir()?;
            let manifest_path = current_dir.join("lis.toml");

            if manifest_path.exists() {
                Ok(InputMode::Project(current_dir))
            } else {
                Err("No input file specified and no lis.toml found in current directory".into())
            }
        }
    }
}

enum InputMode {
    SingleFile(PathBuf),
    Project(PathBuf),
}

fn compile_command(
    input: Option<&Path>,
    output: Option<&Path>,
    print: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    match resolve_input(input)? {
        InputMode::SingleFile(file_path) => {
            compile_single_file(&file_path, output, print)
        }
        InputMode::Project(project_root) => {
            compile_project(&project_root, output, print)
        }
    }
}

fn compile_single_file(
    input: &Path,
    output: Option<&Path>,
    print: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Read source file
    let source = fs::read_to_string(input)?;

    // Compile
    println!(
        "{} {}",
        "Compiling".green().bold(),
        input.display().to_string().cyan()
    );

    let assembly = compile(&source)?;

    // Print if requested
    if print {
        println!("\n{}", "Generated Assembly:".bold());
        println!("{}", assembly);
    }

    // Write output
    if let Some(output_path) = output {
        fs::write(output_path, &assembly)?;
        println!(
            "{} {}",
            "   Created".green().bold(),
            output_path.display().to_string().cyan()
        );
    } else {
        // Default output: replace .lis with .sil
        let default_output = input.with_extension("sil");
        fs::write(&default_output, &assembly)?;
        println!(
            "{} {}",
            "   Created".green().bold(),
            default_output.display().to_string().cyan()
        );
    }

    println!("{}", "    Finished".green().bold());

    Ok(())
}

fn compile_project(
    project_root: &Path,
    output: Option<&Path>,
    print: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let manifest = Manifest::from_file(&project_root.join("lis.toml"))?;

    println!(
        "{} {} v{}",
        "Compiling".green().bold(),
        manifest.package.name.cyan(),
        manifest.package.version
    );

    // Resolve all modules
    let mut resolver = ModuleResolver::new(manifest.clone(), project_root.to_path_buf())?;
    let modules = resolver.resolve_all()?;

    println!(
        "{} {} module(s)",
        "  Resolved".green().bold(),
        modules.len()
    );

    // Compile all modules into a single assembly
    let mut full_assembly = String::new();

    for module in &modules {
        let source = fs::read_to_string(&module.file_path)?;
        let assembly = compile(&source)?;
        full_assembly.push_str(&format!("; === Module: {} ===\n", module.canonical_name));
        full_assembly.push_str(&assembly);
        full_assembly.push('\n');
    }

    // Print if requested
    if print {
        println!("\n{}", "Generated Assembly:".bold());
        println!("{}", full_assembly);
    }

    // Write output
    let output_path = if let Some(p) = output {
        p.to_path_buf()
    } else {
        let output_dir = project_root.join("target");
        fs::create_dir_all(&output_dir)?;
        output_dir.join(format!("{}.sil", manifest.package.name))
    };

    fs::write(&output_path, &full_assembly)?;
    println!(
        "{} {}",
        "   Created".green().bold(),
        output_path.display().to_string().cyan()
    );

    println!("{}", "    Finished".green().bold());

    Ok(())
}

fn build_command(
    input: Option<&Path>,
    output: Option<&Path>,
    release: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    match resolve_input(input)? {
        InputMode::SingleFile(file_path) => {
            build_single_file(&file_path, output, release)
        }
        InputMode::Project(project_root) => {
            build_project(&project_root, output, release)
        }
    }
}

fn build_single_file(
    input: &Path,
    output: Option<&Path>,
    _release: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let source = fs::read_to_string(input)?;

    println!(
        "{} {}",
        "   Building".green().bold(),
        input.display().to_string().cyan()
    );

    // 1. Compile LIS to SIL assembly
    let assembly = compile(&source)?;

    // 2. Assemble to bytecode
    let silc = assemble(&assembly)?;
    let bytecode = silc.to_bytes();

    // Write bytecode
    let output_path = if let Some(p) = output {
        p.to_path_buf()
    } else {
        input.with_extension("silc")
    };

    fs::write(&output_path, &bytecode)?;

    println!(
        "{} {} ({} bytes)",
        "   Created".green().bold(),
        output_path.display().to_string().cyan(),
        bytecode.len()
    );
    println!("{}", "    Finished".green().bold());

    Ok(())
}

fn build_project(
    project_root: &Path,
    output: Option<&Path>,
    release: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let manifest = Manifest::from_file(&project_root.join("lis.toml"))?;

    let mode = if release { "release" } else { "debug" };
    println!(
        "{} {} v{} ({})",
        "   Building".green().bold(),
        manifest.package.name.cyan(),
        manifest.package.version,
        mode
    );

    // Resolve all modules
    let mut resolver = ModuleResolver::new(manifest.clone(), project_root.to_path_buf())?;
    let modules = resolver.resolve_all()?;

    // Compile all modules into a single assembly
    let mut full_assembly = String::new();

    for module in &modules {
        let source = fs::read_to_string(&module.file_path)?;
        let assembly = compile(&source)?;
        full_assembly.push_str(&format!("; === Module: {} ===\n", module.canonical_name));
        full_assembly.push_str(&assembly);
        full_assembly.push('\n');
    }

    // Assemble to bytecode
    let silc = assemble(&full_assembly)?;
    let bytecode = silc.to_bytes();

    // Write bytecode
    let output_dir = project_root.join("target").join(mode);
    fs::create_dir_all(&output_dir)?;

    let output_path = if let Some(p) = output {
        p.to_path_buf()
    } else {
        output_dir.join(format!("{}.silc", manifest.package.name))
    };

    fs::write(&output_path, &bytecode)?;

    println!(
        "{} {} ({} bytes)",
        "   Created".green().bold(),
        output_path.display().to_string().cyan(),
        bytecode.len()
    );
    println!("{}", "    Finished".green().bold());

    Ok(())
}

fn run_command(input: Option<&Path>) -> Result<(), Box<dyn std::error::Error>> {
    match resolve_input(input)? {
        InputMode::SingleFile(file_path) => {
            run_single_file(&file_path)
        }
        InputMode::Project(project_root) => {
            run_project(&project_root)
        }
    }
}

fn run_single_file(input: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let source = fs::read_to_string(input)?;

    println!(
        "{} {}",
        "   Compiling".green().bold(),
        input.display().to_string().cyan()
    );

    // 1. Compile LIS to SIL assembly
    let assembly = compile(&source)?;

    println!(
        "{} Generated assembly ({} bytes)",
        "  Assembling".green().bold(),
        assembly.len()
    );

    // 2. Assemble SIL to bytecode
    let silc = assemble(&assembly)?;
    let bytecode = silc.to_bytes();

    println!(
        "{} Generated bytecode ({} bytes)",
        "     Running".green().bold(),
        bytecode.len()
    );

    // 3. Create VSP and execute
    let mut vsp = Vsp::new(VspConfig::default())?;
    vsp.load(&bytecode)?;

    let result = vsp.run()?;

    println!(
        "{} Execution completed",
        "    Finished".green().bold()
    );

    // Show result state
    println!("\n{}", "Final State:".bold());
    println!("  L0 (Photonic):     {:?}", result.layer(0));
    println!("  L1 (Acoustic):     {:?}", result.layer(1));
    println!("  L2 (Olfactory):    {:?}", result.layer(2));
    println!("  L3 (Gustatory):    {:?}", result.layer(3));
    println!("  L4 (Dermic):       {:?}", result.layer(4));
    println!("  L5 (Electronic):   {:?}", result.layer(5));
    println!("  L6 (Psychomotor):  {:?}", result.layer(6));
    println!("  L7 (Environmental):{:?}", result.layer(7));
    println!("  L8 (Cybernetic):   {:?}", result.layer(8));
    println!("  L9 (Geopolitical): {:?}", result.layer(9));
    println!("  LA (Cosmopolitic): {:?}", result.layer(10));
    println!("  LB (Synergic):     {:?}", result.layer(11));
    println!("  LC (Quantum):      {:?}", result.layer(12));
    println!("  LD (Superpose):    {:?}", result.layer(13));
    println!("  LE (Entanglement): {:?}", result.layer(14));
    println!("  LF (Collapse):     {:?}", result.layer(15));

    Ok(())
}

fn run_project(project_root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let manifest = Manifest::from_file(&project_root.join("lis.toml"))?;

    println!(
        "{} {} v{}",
        "   Compiling".green().bold(),
        manifest.package.name.cyan(),
        manifest.package.version
    );

    // Resolve all modules
    let mut resolver = ModuleResolver::new(manifest.clone(), project_root.to_path_buf())?;
    let modules = resolver.resolve_all()?;

    // Compile all modules into a single assembly
    let mut full_assembly = String::new();

    for module in &modules {
        let source = fs::read_to_string(&module.file_path)?;
        let assembly = compile(&source)?;
        full_assembly.push_str(&format!("; === Module: {} ===\n", module.canonical_name));
        full_assembly.push_str(&assembly);
        full_assembly.push('\n');
    }

    println!(
        "{} Generated assembly ({} bytes)",
        "  Assembling".green().bold(),
        full_assembly.len()
    );

    // Assemble to bytecode
    let silc = assemble(&full_assembly)?;
    let bytecode = silc.to_bytes();

    println!(
        "{} Generated bytecode ({} bytes)",
        "     Running".green().bold(),
        bytecode.len()
    );

    // Create VSP and execute
    let mut vsp = Vsp::new(VspConfig::default())?;
    vsp.load(&bytecode)?;

    let result = vsp.run()?;

    println!(
        "{} Execution completed",
        "    Finished".green().bold()
    );

    // Show result state
    println!("\n{}", "Final State:".bold());
    println!("  L0 (Photonic):     {:?}", result.layer(0));
    println!("  L1 (Acoustic):     {:?}", result.layer(1));
    println!("  L2 (Olfactory):    {:?}", result.layer(2));
    println!("  L3 (Gustatory):    {:?}", result.layer(3));
    println!("  L4 (Dermic):       {:?}", result.layer(4));
    println!("  L5 (Electronic):   {:?}", result.layer(5));
    println!("  L6 (Psychomotor):  {:?}", result.layer(6));
    println!("  L7 (Environmental):{:?}", result.layer(7));
    println!("  L8 (Cybernetic):   {:?}", result.layer(8));
    println!("  L9 (Geopolitical): {:?}", result.layer(9));
    println!("  LA (Cosmopolitic): {:?}", result.layer(10));
    println!("  LB (Synergic):     {:?}", result.layer(11));
    println!("  LC (Quantum):      {:?}", result.layer(12));
    println!("  LD (Superpose):    {:?}", result.layer(13));
    println!("  LE (Entanglement): {:?}", result.layer(14));
    println!("  LF (Collapse):     {:?}", result.layer(15));

    Ok(())
}

fn check_command(input: Option<&Path>) -> Result<(), Box<dyn std::error::Error>> {
    match resolve_input(input)? {
        InputMode::SingleFile(file_path) => {
            check_single_file(&file_path)
        }
        InputMode::Project(project_root) => {
            check_project(&project_root)
        }
    }
}

fn check_single_file(input: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let source = fs::read_to_string(input)?;

    println!(
        "{} {}",
        "Checking".green().bold(),
        input.display().to_string().cyan()
    );

    // Try to compile (validates syntax and semantics)
    compile(&source)?;

    println!(
        "{} No errors found",
        "    Finished".green().bold()
    );

    Ok(())
}

fn check_project(project_root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let manifest = Manifest::from_file(&project_root.join("lis.toml"))?;

    println!(
        "{} {} v{}",
        "  Checking".green().bold(),
        manifest.package.name.cyan(),
        manifest.package.version
    );

    // Resolve all modules (this validates all imports and dependencies)
    let mut resolver = ModuleResolver::new(manifest.clone(), project_root.to_path_buf())?;
    let modules = resolver.resolve_all()?;

    // Check all modules compile
    for module in &modules {
        let source = fs::read_to_string(&module.file_path)?;
        compile(&source)?;
        println!(
            "  {} {}",
            "✓".green(),
            module.canonical_name.cyan()
        );
    }

    println!(
        "{} No errors found ({} module(s))",
        "    Finished".green().bold(),
        modules.len()
    );

    Ok(())
}

fn print_info() {
    println!("{}", "LIS - Language for Intelligent Systems".bold());
    println!();
    println!("{}", "A language for modeling non-linear systems that compiles to SIL VSP bytecode.");
    println!();
    println!("{}", "Features:".bold());
    println!("  • {} Native support for feedback loops and causal cycles", "✓".green());
    println!("  • {} Topology and continuous transformations", "✓".green());
    println!("  • {} Emergence and self-organization", "✓".green());
    println!("  • {} Reflexive metaprogramming", "✓".yellow());
    println!("  • {} Hardware-aware compilation (CPU/GPU/NPU)", "✓".green());
    println!();
    println!("{}", "Architecture:".bold());
    println!("  LIS Source (.lis)");
    println!("       ↓");
    println!("  VSP Assembly (.sil)");
    println!("       ↓");
    println!("  Bytecode (.silc)");
    println!("       ↓");
    println!("  Execution (CPU/GPU/NPU)");
    println!();
    println!("{}", "Project Commands:".bold());
    println!("  lis new my-project               # Create new project");
    println!("  lis init                         # Initialize project in current dir");
    println!("  lis build                        # Build project to bytecode");
    println!("  lis build --release              # Build in release mode");
    println!("  lis run                          # Build and run project");
    println!();
    println!("{}", "File Commands:".bold());
    println!("  lis compile program.lis          # Compile to assembly");
    println!("  lis compile program.lis -p       # Compile and print");
    println!("  lis check program.lis            # Check syntax only");
    println!("  lis info                         # Show this info");
    println!();
    println!("{}", "Integration:".bold());
    println!("  • Compiles to VSP (Virtual Sil Processor) assembly");
    println!("  • Uses SIL's 16-layer state model (L0-LF)");
    println!("  • ByteSil log-polar complex arithmetic");
    println!("  • AGPL-3.0 licensed - edge computing, swarm intelligence");
    println!();
    println!("{}", "\"We are the swarm. We are the vapor. We are the edge.\"".italic());
    println!("{}", "理信 (Lǐxìn) - Where logic and information are indistinguishable.".italic());
}
