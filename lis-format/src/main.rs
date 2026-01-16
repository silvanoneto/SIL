//! LIS Formatter CLI
//!
//! Command-line interface for formatting LIS source code.

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use lis_format::{format_with_config, FormatConfig, IndentStyle};
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "lis-format")]
#[command(author, version, about = "Format LIS source code", long_about = None)]
struct Cli {
    /// Input files to format (use - for stdin)
    #[arg(value_name = "FILE")]
    files: Vec<PathBuf>,

    /// Write formatted output back to file(s)
    #[arg(short, long)]
    write: bool,

    /// Check if files are formatted without writing
    #[arg(short, long)]
    check: bool,

    /// Indentation style
    #[arg(long, value_enum, default_value = "spaces")]
    indent: IndentType,

    /// Number of spaces for indentation (when using spaces)
    #[arg(long, default_value = "4")]
    indent_size: usize,

    /// Maximum line width
    #[arg(long, default_value = "100")]
    max_width: usize,

    /// Disable alignment of assignments
    #[arg(long)]
    no_align_assignments: bool,

    /// Disable spaces around operators
    #[arg(long)]
    no_space_operators: bool,

    /// Use compact style (minimal whitespace)
    #[arg(long)]
    compact: bool,

    /// Use readable style (extra whitespace)
    #[arg(long)]
    readable: bool,

    /// Print statistics
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum IndentType {
    Spaces,
    Tabs,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Build configuration
    let config = build_config(&cli);

    // Handle stdin
    if cli.files.is_empty() || (cli.files.len() == 1 && cli.files[0].to_str() == Some("-")) {
        return format_stdin(&config, cli.write);
    }

    let mut total_files = 0;
    let mut formatted_files = 0;
    let mut already_formatted = 0;
    let mut errors = 0;

    for file in &cli.files {
        total_files += 1;

        match process_file(file, &config, cli.write, cli.check) {
            Ok(status) => match status {
                FileStatus::Formatted => formatted_files += 1,
                FileStatus::AlreadyFormatted => already_formatted += 1,
                FileStatus::NeedsFormatting => {
                    eprintln!("❌ {}: needs formatting", file.display());
                    errors += 1;
                }
            },
            Err(e) => {
                eprintln!("❌ {}: {}", file.display(), e);
                errors += 1;
            }
        }
    }

    if cli.verbose {
        eprintln!("\n📊 Statistics:");
        eprintln!("  Total files: {}", total_files);
        eprintln!("  Formatted: {}", formatted_files);
        eprintln!("  Already formatted: {}", already_formatted);
        eprintln!("  Errors: {}", errors);
    }

    if cli.check && errors > 0 {
        std::process::exit(1);
    }

    Ok(())
}

enum FileStatus {
    Formatted,
    AlreadyFormatted,
    NeedsFormatting,
}

fn process_file(
    path: &PathBuf,
    config: &FormatConfig,
    write_back: bool,
    check_only: bool,
) -> Result<FileStatus> {
    let source = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let formatted = format_with_config(&source, config)
        .with_context(|| format!("Failed to format file: {}", path.display()))?;

    if source == formatted {
        return Ok(FileStatus::AlreadyFormatted);
    }

    if check_only {
        return Ok(FileStatus::NeedsFormatting);
    }

    if write_back {
        fs::write(path, &formatted)
            .with_context(|| format!("Failed to write file: {}", path.display()))?;
        eprintln!("✓ {}: formatted", path.display());
        Ok(FileStatus::Formatted)
    } else {
        print!("{}", formatted);
        Ok(FileStatus::Formatted)
    }
}

fn format_stdin(config: &FormatConfig, write_back: bool) -> Result<()> {
    if write_back {
        anyhow::bail!("Cannot use --write with stdin");
    }

    let mut source = String::new();
    io::stdin()
        .read_to_string(&mut source)
        .context("Failed to read from stdin")?;

    let formatted = format_with_config(&source, config).context("Failed to format stdin")?;

    print!("{}", formatted);

    Ok(())
}

fn build_config(cli: &Cli) -> FormatConfig {
    // Start with preset
    let mut config = if cli.compact {
        FormatConfig::compact()
    } else if cli.readable {
        FormatConfig::readable()
    } else {
        FormatConfig::default()
    };

    // Apply overrides
    config.indent_style = match cli.indent {
        IndentType::Spaces => IndentStyle::Spaces(cli.indent_size),
        IndentType::Tabs => IndentStyle::Tabs,
    };

    config.max_width = cli.max_width;

    if cli.no_align_assignments {
        config.align_assignments = false;
    }

    if cli.no_space_operators {
        config.space_around_operators = false;
    }

    config
}
