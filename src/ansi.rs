//! Simple ANSI escape sequence filter.
//!
//! This module provides basic ANSI sequence filtering to extract
//! plain text from shell output without full parser complexity.

/// State machine for filtering ANSI escape sequences.
#[derive(Debug, Clone, Copy, PartialEq)]
enum FilterState {
    /// Normal text output
    Normal,
    /// ESC character received, expecting '['
    Escape,
    /// Inside CSI sequence, reading parameters and final byte
    Csi,
}

/// ANSI escape sequence filter.
///
/// Processes shell output byte-by-byte, filtering out ANSI escape
/// sequences while preserving plain text.
pub struct AnsiFilter {
    state: FilterState,
}

impl AnsiFilter {
    /// Create a new ANSI filter.
    pub fn new() -> Self {
        Self {
            state: FilterState::Normal,
        }
    }

    /// Process a single byte, returning Some(char) if it should be displayed.
    ///
    /// # Returns
    /// - `Some(char)` - Printable character that should be displayed
    /// - `None` - Byte was filtered out (part of ANSI sequence or control char)
    pub fn process(&mut self, byte: u8) -> Option<char> {
        match self.state {
            FilterState::Normal => {
                match byte {
                    0x1b => {
                        // ESC - start of escape sequence
                        self.state = FilterState::Escape;
                        None
                    }
                    0x07 => {
                        // BEL - bell character, ignore
                        None
                    }
                    0x08 => {
                        // BS - backspace, return as control char
                        Some('\x08')
                    }
                    0x09 => {
                        // HT - tab, return as control char
                        Some('\t')
                    }
                    0x0a..=0x0d => {
                        // LF, VT, FF, CR - line breaks, return as control chars
                        Some(byte as char)
                    }
                    0x20..=0x7e => {
                        // Printable ASCII range
                        Some(byte as char)
                    }
                    _ => {
                        // Other control characters, ignore
                        None
                    }
                }
            }
            FilterState::Escape => {
                match byte {
                    b'[' => {
                        // CSI sequence start
                        self.state = FilterState::Csi;
                        None
                    }
                    b'P' | b']' | b'^' | b'_' => {
                        // Other escape sequences (DCS, OSC, PM, APC)
                        // Ignore for now
                        None
                    }
                    _ => {
                        // Simple escape sequence like ESC M, return to normal
                        self.state = FilterState::Normal;
                        None
                    }
                }
            }
            FilterState::Csi => {
                // CSI parameter byte or final byte
                // Parameters: 0x30-0x3F (0-9, :, ;, <, =, >, ?)
                // Intermediate: 0x20-0x2F (space, !"#$%&'()*+,-./)
                // Final: 0x40-0x7E (@A-Z[\]^_`a-z{|}~)
                match byte {
                    0x40..=0x7e => {
                        // Final byte, sequence complete
                        self.state = FilterState::Normal;
                        None
                    }
                    _ => {
                        // Parameter or intermediate byte, stay in CSI
                        None
                    }
                }
            }
        }
    }

    /// Process a byte slice, returning a String of filtered text.
    pub fn process_slice(&mut self, bytes: &[u8]) -> String {
        let mut result = String::new();
        for &byte in bytes {
            if let Some(c) = self.process(byte) {
                result.push(c);
            }
        }
        result
    }

    /// Reset the filter to initial state.
    pub fn reset(&mut self) {
        self.state = FilterState::Normal;
    }
}

impl Default for AnsiFilter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_text() {
        let mut filter = AnsiFilter::new();
        let input = b"Hello, World!";
        let output = filter.process_slice(input);
        assert_eq!(output, "Hello, World!");
    }

    #[test]
    fn test_color_filter() {
        let mut filter = AnsiFilter::new();
        // Red color sequence: ESC[31m
        let input = b"\x1b[31mHello\x1b[0m";
        let output = filter.process_slice(input);
        assert_eq!(output, "Hello");
    }

    #[test]
    fn test_cursor_movement() {
        let mut filter = AnsiFilter::new();
        // Cursor up: ESC[A
        let input = b"Text\x1b[A_more";
        let output = filter.process_slice(input);
        assert_eq!(output, "Text_more");
    }

    #[test]
    fn test_line_breaks() {
        let mut filter = AnsiFilter::new();
        let input = b"Line1\nLine2\rLine3";
        let output = filter.process_slice(input);
        assert_eq!(output, "Line1\nLine2\rLine3");
    }
}
