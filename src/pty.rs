//! PTY (Pseudo-Terminal) session management for shell integration.
//!
//! This module handles creating a pseudo-terminal, spawning a shell process,
//! and reading its output in a separate thread.

use portable_pty::{native_pty_system, CommandBuilder, Child, PtySize};
use std::io::{self, Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;

/// PTY session that manages a shell process.
///
/// The session spawns a shell (zsh) and provides access to its
/// input/output streams. The reader runs in a separate thread to
/// continuously read shell output.
pub struct PtySession {
    /// The child shell process (kept alive to prevent premature termination)
    _child: Box<dyn Child>,
    /// Reader for shell output (wrapped in Arc<Mutex<>> for thread sharing)
    reader: Arc<Mutex<Box<dyn Read + Send>>>,
    /// Writer for shell input (unused in this step, but kept for future use)
    _writer: Box<dyn Write + Send>,
}

impl PtySession {
    /// Create a new PTY session with a zsh shell.
    ///
    /// # Returns
    /// A `PtySession` instance ready to read shell output.
    ///
    /// # Panics
    /// Panics if:
    /// - PTY system cannot be created
    /// - Shell process cannot be spawned
    /// - Reader/writer cannot be obtained
    pub fn new() -> Self {
        log::info!("Creating PTY session");

        // Get the native PTY system for the current platform
        let pty_system = native_pty_system();

        // Create a new PTY pair with reasonable size
        let pty_size = PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        };

        let pty_pair = pty_system
            .openpty(pty_size)
            .expect("Failed to create PTY pair");

        log::debug!("PTY pair created successfully");

        // Build the command to spawn zsh
        let mut cmd = CommandBuilder::new("zsh");

        // Set up a clean environment for the shell
        cmd.env("TERM", "xterm-256color");

        // Spawn the shell process
        let child = pty_pair
            .slave
            .spawn_command(cmd)
            .expect("Failed to spawn zsh");

        log::info!("Zsh spawned successfully");

        // Split the PTY master into reader and writer
        let reader = pty_pair.master.try_clone_reader().unwrap();
        let writer = pty_pair.master.take_writer().unwrap();

        log::debug!("Reader and writer obtained from PTY master");

        Self {
            _child: child,
            reader: Arc::new(Mutex::new(reader)),
            _writer: writer,
        }
    }

    /// Spawn a background thread that continuously reads shell output.
    ///
    /// The thread reads from the PTY reader and prints all output to stdout.
    /// This runs indefinitely until:
    /// - The shell exits (EOF)
    /// - A read error occurs
    pub fn spawn_reader_thread(&self) {
        let reader = Arc::clone(&self.reader);

        thread::spawn(move || {
            log::info!("PTY reader thread started");

            let mut buffer = vec![0u8; 8192];
            let mut total_bytes = 0;

            loop {
                // Lock the mutex to get access to the reader
                let mut reader_guard = reader.lock().unwrap();
                match reader_guard.read(&mut buffer) {
                    Ok(0) => {
                        // EOF - shell has exited
                        log::info!("PTY reader reached EOF (shell exited)");
                        break;
                    }
                    Ok(n) => {
                        total_bytes += n;
                        // Release the lock before printing
                        drop(reader_guard);

                        // Print the output to console
                        let output = String::from_utf8_lossy(&buffer[..n]);
                        print!("{}", output);

                        // Ensure output is flushed immediately
                        if let Err(e) = io::stdout().flush() {
                            log::error!("Failed to flush stdout: {}", e);
                        }
                    }
                    Err(e) => {
                        log::error!("PTY read error: {}", e);
                        break;
                    }
                }
            }

            log::info!("PTY reader thread exiting (total bytes read: {})", total_bytes);
        });
    }
}

// When the PtySession is dropped, the child process will be killed.
impl Drop for PtySession {
    fn drop(&mut self) {
        log::info!("PtySession dropped, shell process will be terminated");
    }
}
