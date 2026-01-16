//! LIS Code Formatter
//!
//! A pretty-printer and code formatter for the LIS language that ensures
//! consistent code style with configurable options.

mod config;
mod formatter;
mod printer;

pub use config::{FormatConfig, IndentStyle};
pub use formatter::format_program;

use lis_core::parse;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FormatError {
    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Format LIS source code with default configuration
pub fn format(source: &str) -> Result<String, FormatError> {
    format_with_config(source, &FormatConfig::default())
}

/// Format LIS source code with custom configuration
pub fn format_with_config(source: &str, config: &FormatConfig) -> Result<String, FormatError> {
    let program = parse(source)
        .map_err(|e| FormatError::ParseError(format!("{:?}", e)))?;

    Ok(format_program(&program, config))
}

/// Check if source code is already formatted
pub fn is_formatted(source: &str) -> Result<bool, FormatError> {
    let formatted = format(source)?;
    Ok(source == formatted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_formatting() {
        let input = r#"fn main(){let x=42;}"#;
        let result = format(input);
        assert!(result.is_ok());
        let formatted = result.unwrap();
        assert!(formatted.contains("fn main()"));
        assert!(formatted.contains("let x = 42;"));
    }

    #[test]
    fn test_is_formatted() {
        let formatted_code = r#"fn main() {
    let x = 42;
}
"#;
        assert!(is_formatted(formatted_code).unwrap_or(false));
    }
}
