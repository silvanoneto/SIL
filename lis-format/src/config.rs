//! Formatter configuration

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndentStyle {
    Spaces(usize),
    Tabs,
}

impl Default for IndentStyle {
    fn default() -> Self {
        IndentStyle::Spaces(4)
    }
}

/// Configuration for the LIS formatter
#[derive(Debug, Clone, PartialEq)]
pub struct FormatConfig {
    /// Indentation style
    pub indent_style: IndentStyle,

    /// Maximum line width before wrapping
    pub max_width: usize,

    /// Align consecutive assignments
    pub align_assignments: bool,

    /// Align function parameters vertically
    pub align_params: bool,

    /// Insert spaces around binary operators
    pub space_around_operators: bool,

    /// Insert space after comma
    pub space_after_comma: bool,

    /// Insert space before block opening brace
    pub space_before_brace: bool,

    /// Add trailing comma in multiline constructs
    pub trailing_comma: bool,

    /// Blank lines between items (functions, transforms)
    pub blank_lines_between_items: usize,

    /// Format comments
    pub format_comments: bool,
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            indent_style: IndentStyle::default(),
            max_width: 100,
            align_assignments: true,
            align_params: false,
            space_around_operators: true,
            space_after_comma: true,
            space_before_brace: true,
            trailing_comma: true,
            blank_lines_between_items: 1,
            format_comments: true,
        }
    }
}

impl FormatConfig {
    /// Create a compact configuration (minimal whitespace)
    pub fn compact() -> Self {
        Self {
            space_around_operators: false,
            space_after_comma: false,
            space_before_brace: false,
            trailing_comma: false,
            blank_lines_between_items: 0,
            ..Default::default()
        }
    }

    /// Create a configuration optimized for readability
    pub fn readable() -> Self {
        Self {
            align_assignments: true,
            align_params: true,
            blank_lines_between_items: 2,
            ..Default::default()
        }
    }

    /// Get the indent string for one level
    pub fn indent_str(&self) -> String {
        match self.indent_style {
            IndentStyle::Spaces(n) => " ".repeat(n),
            IndentStyle::Tabs => "\t".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = FormatConfig::default();
        assert_eq!(config.indent_style, IndentStyle::Spaces(4));
        assert_eq!(config.max_width, 100);
        assert!(config.align_assignments);
    }

    #[test]
    fn test_indent_str() {
        let config_spaces = FormatConfig {
            indent_style: IndentStyle::Spaces(2),
            ..Default::default()
        };
        assert_eq!(config_spaces.indent_str(), "  ");

        let config_tabs = FormatConfig {
            indent_style: IndentStyle::Tabs,
            ..Default::default()
        };
        assert_eq!(config_tabs.indent_str(), "\t");
    }

    #[test]
    fn test_compact_config() {
        let config = FormatConfig::compact();
        assert!(!config.space_around_operators);
        assert!(!config.space_after_comma);
        assert!(!config.trailing_comma);
    }
}
