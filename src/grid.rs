//! Terminal grid data structure - simplified version.
//!
//! This module defines a simple grid to store terminal output
//! without complex ANSI parsing for now.

use std::fmt;

/// Terminal grid storing plain text output.
pub struct Grid {
    /// Number of rows in the grid.
    pub rows: usize,
    /// Number of columns in the grid.
    pub cols: usize,
    /// The grid cells as plain text.
    pub cells: Vec<Vec<String>>,
}

impl Grid {
    /// Create a new grid with the specified dimensions.
    pub fn new(rows: usize, cols: usize) -> Self {
        log::debug!("Creating {}x{} grid", rows, cols);

        let cells = vec![vec![String::new(); cols]; rows];

        Self { rows, cols, cells }
    }

    /// Write a character at the specified position.
    pub fn write_char(&mut self, row: usize, col: usize, c: char) {
        if row < self.rows && col < self.cols {
            self.cells[row][col].push(c);
        }
    }

    /// Write a string at the specified position.
    pub fn write_str(&mut self, row: usize, col: usize, s: &str) {
        if row < self.rows && col < self.cols {
            self.cells[row][col].push_str(s);
        }
    }
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in &self.cells {
            for cell in row {
                write!(f, "{}", cell)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
