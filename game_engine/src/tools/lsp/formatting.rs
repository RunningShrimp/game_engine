//! # Code Formatting for LSP
//!
//! Provides code formatting functionality.

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

/// Code formatter
pub struct CodeFormatter {
    /// Indent size (spaces)
    indent_size: u32,

    /// Use tabs instead of spaces
    use_tabs: bool,
}

impl CodeFormatter {
    /// Create a new code formatter
    pub fn new() -> Self {
        Self {
            indent_size: 4,
            use_tabs: false,
        }
    }

    /// Format code
    pub fn format(&self, text: &str, options: &FormattingOptions) -> String {
        // Simple formatting implementation
        // In production, use rustfmt for Rust, prettier for JS/TS, etc.

        let mut formatted = String::new();
        let lines: Vec<&str> = text.lines().collect();
        let mut indent_level: i32 = 0;
        let indent_char = if options.insert_spaces {
            " ".repeat(options.tab_size as usize)
        } else {
            "\t".to_string()
        };

        for line in lines {
            let trimmed = line.trim();

            // Skip empty lines
            if trimmed.is_empty() {
                formatted.push('\n');
                continue;
            }

            // Decrease indent for closing braces
            if trimmed.starts_with('}') || trimmed.starts_with(']') || trimmed.starts_with(')') {
                indent_level = indent_level.saturating_sub(1);
            }

            // Add indentation
            for _ in 0..indent_level {
                formatted.push_str(&indent_char);
            }

            formatted.push_str(trimmed);
            formatted.push('\n');

            // Increase indent for opening braces
            if trimmed.ends_with('{') || trimmed.ends_with('[') || trimmed.ends_with('(') {
                indent_level += 1;
            }
        }

        formatted
    }

    /// Format a range of code
    pub fn format_range(
        &self,
        text: &str,
        range: &Range,
        options: &FormattingOptions,
    ) -> Vec<TextEdit> {
        let lines: Vec<&str> = text.lines().collect();
        let start_line = range.start.line as usize;
        let end_line = range.end.line as usize;

        if start_line >= lines.len() || end_line >= lines.len() {
            return Vec::new();
        }

        // Extract the range
        let range_text: String = lines[start_line..=end_line].join("\n");
        let formatted = self.format(&range_text, options);

        vec![TextEdit {
            range: Range {
                start: Position {
                    line: range.start.line,
                    character: 0,
                },
                end: Position {
                    line: range.end.line,
                    character: lines[end_line].len() as u32,
                },
            },
            new_text: formatted,
        }]
    }
}

impl Default for CodeFormatter {
    fn default() -> Self {
        Self::new()
    }
}
