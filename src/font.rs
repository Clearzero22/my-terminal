//! Font rendering for terminal text display.
//!
//! This module provides text rendering using fontdue for
//! simple, dependency-free font rasterization.

use fontdue::{Font, FontSettings};
use std::sync::Arc;

/// Terminal font renderer.
///
/// Provides text rasterization with configurable size and colors.
pub struct FontRenderer {
    /// The font face for rendering text
    font: Font,
    /// Font size in pixels
    font_size: f32,
    /// Character width in pixels (monospace)
    char_width: usize,
    /// Character height in pixels
    char_height: usize,
}

impl FontRenderer {
    /// Create a new font renderer with default settings.
    ///
    /// Uses Roboto font as the default terminal font.
    ///
    /// # Returns
    /// A `FontRenderer` instance ready for text rendering.
    ///
    /// # Panics
    /// Panics if the embedded font data cannot be loaded.
    pub fn new() -> Self {
        Self::with_size(14.0)
    }

    /// Create a new font renderer with specified font size.
    ///
    /// # Arguments
    /// * `font_size` - Font size in pixels
    ///
    /// # Returns
    /// A `FontRenderer` instance with the specified font size.
    ///
    /// # Panics
    /// Panics if the embedded font data cannot be loaded.
    pub fn with_size(font_size: f32) -> Self {
        // Load a simple monospace-ish font from fontdue's examples
        // For now, we'll use the built-in fontdue font
        let font_data = include_bytes!("fonts/Roboto-Regular.ttf");
        let font = Font::from_bytes(
            font_data.as_slice(),
            FontSettings::default(),
        ).expect("Failed to load font");

        // Calculate character dimensions
        let char_width = (font_size * 0.6) as usize; // Approximate monospace width
        let char_height = font_size.ceil() as usize;

        Self {
            font,
            font_size,
            char_width,
            char_height,
        }
    }

    /// Get the character width in pixels.
    pub fn char_width(&self) -> usize {
        self.char_width
    }

    /// Get the character height in pixels.
    pub fn char_height(&self) -> usize {
        self.char_height
    }

    /// Get the font size in pixels.
    pub fn font_size(&self) -> f32 {
        self.font_size
    }

    /// Render a single character to a pixel buffer.
    ///
    /// # Arguments
    /// * `c` - The character to render
    /// * `x` - X position in the buffer
    /// * `y` - Y position in the buffer
    /// * `buffer` - The pixel buffer to draw to
    /// * `buffer_width` - Width of the buffer in pixels
    /// * `buffer_height` - Height of the buffer in pixels
    /// * `color` - ARGB color value (0xAARRGGBB)
    pub fn render_char(
        &self,
        c: char,
        x: usize,
        y: usize,
        buffer: &mut [u32],
        buffer_width: usize,
        buffer_height: usize,
        color: u32,
    ) {
        // Rasterize the character
        let (metrics, bitmap) = self.font.rasterize(c, self.font_size);

        // Extract color components
        let a = ((color >> 24) & 0xFF) as u32;
        let r = ((color >> 16) & 0xFF) as u32;
        let g = ((color >> 8) & 0xFF) as u32;
        let b = (color & 0xFF) as u32;

        // Draw each pixel of the glyph
        for (i, &alpha) in bitmap.iter().enumerate() {
            if alpha == 0 {
                continue;
            }

            let glyph_x = x + (i % metrics.width);
            let glyph_y = y + (i / metrics.width);

            if glyph_x >= buffer_width || glyph_y >= buffer_height {
                continue;
            }

            let idx = glyph_y * buffer_width + glyph_x;

            // Blend the pixel with the existing buffer content
            let existing = buffer[idx];
            let existing_a = (existing >> 24) & 0xFF;
            let existing_r = (existing >> 16) & 0xFF;
            let existing_g = (existing >> 8) & 0xFF;
            let existing_b = existing & 0xFF;

            // Simple alpha blending
            let alpha_value = alpha as u32;
            let inv_alpha = 255 - alpha_value;

            let new_a = ((a * alpha_value + existing_a * inv_alpha) / 255) & 0xFF;
            let new_r = ((r * alpha_value + existing_r * inv_alpha) / 255) & 0xFF;
            let new_g = ((g * alpha_value + existing_g * inv_alpha) / 255) & 0xFF;
            let new_b = ((b * alpha_value + existing_b * inv_alpha) / 255) & 0xFF;

            buffer[idx] = (new_a << 24) | (new_r << 16) | (new_g << 8) | new_b;
        }
    }

    /// Render a string to a pixel buffer.
    ///
    /// # Arguments
    /// * `text` - The text to render
    /// * `x` - X position in the buffer
    /// * `y` - Y position in the buffer
    /// * `buffer` - The pixel buffer to draw to
    /// * `buffer_width` - Width of the buffer in pixels
    /// * `buffer_height` - Height of the buffer in pixels
    /// * `color` - ARGB color value (0xAARRGGBB)
    pub fn render_text(
        &self,
        text: &str,
        mut x: usize,
        y: usize,
        buffer: &mut [u32],
        buffer_width: usize,
        buffer_height: usize,
        color: u32,
    ) {
        for c in text.chars() {
            match c {
                '\n' => {
                    // Line break: move to next line
                    x = 0;
                    // Note: y increment should be handled by caller for multi-line rendering
                }
                '\r' => {
                    // Carriage return: move to start of line
                    x = 0;
                }
                '\t' => {
                    // Tab: move to next tab stop (every 8 characters)
                    x = ((x / self.char_width / 8) + 1) * 8 * self.char_width;
                }
                _ if c.is_control() => {
                    // Other control characters: skip
                }
                _ => {
                    // Regular character: render it
                    self.render_char(c, x, y, buffer, buffer_width, buffer_height, color);
                    x += self.char_width;
                }
            }

            // Stop if we've gone past the buffer width
            if x >= buffer_width {
                break;
            }
        }
    }

    /// Render multiple lines of text to a pixel buffer.
    ///
    /// # Arguments
    /// * `lines` - Iterator over lines of text
    /// * `x` - X position in the buffer
    /// * `y` - Y position in the buffer (first line)
    /// * `buffer` - The pixel buffer to draw to
    /// * `buffer_width` - Width of the buffer in pixels
    /// * `buffer_height` - Height of the buffer in pixels
    /// * `color` - ARGB color value (0xAARRGGBB)
    /// * `visible_lines` - Maximum number of lines to render
    pub fn render_lines(
        &self,
        lines: impl Iterator<Item = String>,
        mut x: usize,
        mut y: usize,
        buffer: &mut [u32],
        buffer_width: usize,
        buffer_height: usize,
        color: u32,
        visible_lines: usize,
    ) {
        for (line_num, line) in lines.enumerate() {
            if line_num >= visible_lines {
                break;
            }
            if y + self.char_height > buffer_height {
                break;
            }

            self.render_text(&line, x, y, buffer, buffer_width, buffer_height, color);
            y += self.char_height;
        }
    }
}

impl Default for FontRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Split text into lines suitable for terminal display.
///
/// Handles both \n and \r\n line endings.
pub fn split_lines(text: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\r' => {
                // Check for CRLF
                if chars.peek() == Some(&'\n') {
                    chars.next(); // Skip the \n
                }
                // Carriage return: end current line
                if !current_line.is_empty() {
                    lines.push(current_line.clone());
                }
                current_line.clear();
            }
            '\n' => {
                // Line feed: end current line
                if !current_line.is_empty() {
                    lines.push(current_line.clone());
                }
                lines.push(String::new()); // Empty line for explicit line breaks
                current_line.clear();
            }
            _ => {
                current_line.push(c);
            }
        }
    }

    // Add the last line if it has content
    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_lines_basic() {
        let text = "line1\nline2\nline3";
        let lines = split_lines(text);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "line1");
        assert_eq!(lines[1], "line2");
        assert_eq!(lines[2], "line3");
    }

    #[test]
    fn test_split_lines_crlf() {
        let text = "line1\r\nline2\r\nline3";
        let lines = split_lines(text);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "line1");
        assert_eq!(lines[1], "line2");
        assert_eq!(lines[2], "line3");
    }
}
