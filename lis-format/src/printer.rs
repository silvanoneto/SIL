//! Pretty-printer utilities
//!
//! Handles the low-level string building and indentation management.

use crate::config::FormatConfig;

/// A pretty-printer that manages indentation and line wrapping
pub struct Printer {
    config: FormatConfig,
    buffer: String,
    indent_level: usize,
    current_line_len: usize,
    at_line_start: bool,
}

impl Printer {
    pub fn new(config: FormatConfig) -> Self {
        Self {
            config,
            buffer: String::new(),
            indent_level: 0,
            current_line_len: 0,
            at_line_start: true,
        }
    }

    /// Get the formatted output
    pub fn finish(mut self) -> String {
        // Ensure final newline
        if !self.buffer.ends_with('\n') {
            self.buffer.push('\n');
        }
        self.buffer
    }

    /// Write a string to the output
    pub fn write(&mut self, s: &str) {
        if self.at_line_start && !s.is_empty() {
            self.write_indent();
            self.at_line_start = false;
        }
        self.buffer.push_str(s);
        self.current_line_len += s.len();
    }

    /// Write a line (with newline)
    pub fn writeln(&mut self, s: &str) {
        self.write(s);
        self.newline();
    }

    /// Write a newline
    pub fn newline(&mut self) {
        self.buffer.push('\n');
        self.current_line_len = 0;
        self.at_line_start = true;
    }

    /// Write a space (unless at line start)
    pub fn space(&mut self) {
        if !self.at_line_start {
            self.write(" ");
        }
    }

    /// Write multiple blank lines
    pub fn blank_lines(&mut self, count: usize) {
        for _ in 0..count {
            self.newline();
        }
    }

    /// Increase indentation level
    pub fn indent(&mut self) {
        self.indent_level += 1;
    }

    /// Decrease indentation level
    pub fn dedent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }

    /// Execute a closure with increased indentation
    pub fn indented<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Self),
    {
        self.indent();
        f(self);
        self.dedent();
    }

    /// Write the current indentation
    fn write_indent(&mut self) {
        let indent = self.config.indent_str().repeat(self.indent_level);
        self.buffer.push_str(&indent);
        self.current_line_len += indent.len();
    }

    /// Get current line length
    #[allow(dead_code)]
    pub fn current_line_length(&self) -> usize {
        self.current_line_len
    }

    /// Check if we should wrap to a new line
    #[allow(dead_code)]
    pub fn should_wrap(&self) -> bool {
        self.current_line_len > self.config.max_width
    }

    /// Check if space should be added around operators
    pub fn space_around_operators(&self) -> bool {
        self.config.space_around_operators
    }

    /// Check if space should be added after comma
    pub fn space_after_comma(&self) -> bool {
        self.config.space_after_comma
    }

    /// Check if space should be added before brace
    pub fn space_before_brace(&self) -> bool {
        self.config.space_before_brace
    }

    /// Get the config
    pub fn config(&self) -> &FormatConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_printing() {
        let mut printer = Printer::new(FormatConfig::default());
        printer.write("Hello");
        printer.space();
        printer.write("World");
        let result = printer.finish();
        assert_eq!(result, "Hello World\n");
    }

    #[test]
    fn test_indentation() {
        let mut printer = Printer::new(FormatConfig::default());
        printer.writeln("fn main() {");
        printer.indented(|p| {
            p.writeln("let x = 42;");
        });
        printer.write("}");
        let result = printer.finish();
        assert!(result.contains("    let x = 42;"));
    }

    #[test]
    fn test_blank_lines() {
        let mut printer = Printer::new(FormatConfig::default());
        printer.writeln("line1");
        printer.blank_lines(2);
        printer.writeln("line2");
        let result = printer.finish();
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 4); // line1, blank, blank, line2
    }
}
