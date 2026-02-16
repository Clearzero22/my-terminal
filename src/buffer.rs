//! Terminal output buffer for storing and rendering text.

use crate::ansi::AnsiFilter;
use std::sync::{Arc, Mutex};

/// Terminal buffer that stores filtered text output.
///
/// This buffer stores PTY output after ANSI filtering and provides
/// access for rendering to the window.
#[derive(Clone)]
pub struct TerminalBuffer {
    /// The filtered text content
    content: Arc<Mutex<String>>,
    /// ANSI filter for processing escape sequences
    filter: Arc<Mutex<AnsiFilter>>,
}

impl TerminalBuffer {
    /// Create a new terminal buffer.
    pub fn new() -> Self {
        Self {
            content: Arc::new(Mutex::new(String::new())),
            filter: Arc::new(Mutex::new(AnsiFilter::new())),
        }
    }

    /// Process raw PTY output and add to buffer.
    ///
    /// This filters ANSI escape sequences and stores the plain text.
    pub fn write(&self, bytes: &[u8]) {
        let mut filter = self.filter.lock().unwrap();
        let filtered = filter.process_slice(bytes);
        drop(filter);

        if !filtered.is_empty() {
            let mut content = self.content.lock().unwrap();
            content.push_str(&filtered);
        }
    }

    /// Get a clone of the content for rendering.
    pub fn content(&self) -> String {
        let content = self.content.lock().unwrap();
        content.clone()
    }

    /// Clear the buffer content.
    pub fn clear(&self) {
        let mut content = self.content.lock().unwrap();
        content.clear();
    }

    /// Get the content Arc for shared access.
    pub fn content_arc(&self) -> Arc<Mutex<String>> {
        Arc::clone(&self.content)
    }

    /// Get the filter Arc for shared access.
    pub fn filter_arc(&self) -> Arc<Mutex<AnsiFilter>> {
        Arc::clone(&self.filter)
    }
}

impl Default for TerminalBuffer {
    fn default() -> Self {
        Self::new()
    }
}
