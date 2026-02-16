//! PTY (Pseudo-Terminal) session management for shell integration.
//!
//! This module handles creating a pseudo-terminal, spawning a shell process,
//! and reading/writing its output in a separate thread.

use portable_pty::{native_pty_system, CommandBuilder, Child, PtySize};
use std::io::{self, Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Callback type for handling PTY output.
///
/// This callback receives raw bytes from the PTY and can process
/// them (e.g., write to a buffer, parse ANSI sequences, etc.).
pub type OutputCallback = Arc<Mutex<Box<dyn Fn(&[u8]) + Send + Sync>>>;

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
    /// Writer for shell input (wrapped in Arc<Mutex<>> for thread safety)
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    /// Handle to the reader thread (stored to prevent thread leak)
    _reader_thread: Option<thread::JoinHandle<()>>,
}

impl PtySession {
    /// Create a new PTY session with a zsh shell.
    ///
    /// This automatically spawns the reader thread to begin reading
    /// shell output. Output will be printed directly to stdout.
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
        Self::with_output_callback(None)
    }

    /// Create a new PTY session with a custom output callback.
    ///
    /// # Arguments
    /// * `callback` - Optional callback to handle PTY output. If None, output
    ///               is printed directly to stdout.
    ///
    /// # Returns
    /// A `PtySession` instance ready to read shell output.
    ///
    /// # Panics
    /// Panics if:
    /// - PTY system cannot be created
    /// - Shell process cannot be spawned
    /// - Reader/writer cannot be obtained
    pub fn with_output_callback(callback: Option<OutputCallback>) -> Self {
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

        // Wrap both in Arc<Mutex<>> for thread-safe access
        let reader = Arc::new(Mutex::new(reader));
        let writer = Arc::new(Mutex::new(writer));

        // Spawn the reader thread and store the JoinHandle
        let reader_clone = Arc::clone(&reader);
        let handle = thread::Builder::new()
            .name("pty-reader".to_string())
            .spawn(move || {
                log::info!("PTY reader thread started");

                let mut buffer = vec![0u8; 8192];
                let mut total_bytes = 0;

                loop {
                    // Lock the mutex to get access to the reader
                    let mut reader_guard = reader_clone.lock().unwrap();
                    match reader_guard.read(&mut buffer) {
                        Ok(0) => {
                            // EOF - shell has exited
                            log::info!("PTY reader reached EOF (shell exited)");
                            break;
                        }
                        Ok(n) => {
                            total_bytes += n;
                            let data = &buffer[..n];
                            // Release the lock before processing
                            drop(reader_guard);

                            // Use callback if provided, otherwise print to stdout
                            if let Some(cb) = &callback {
                                let cb = cb.lock().unwrap();
                                cb(data);
                            } else {
                                // Print the output to console
                                let output = String::from_utf8_lossy(data);
                                print!("{}", output);

                                // Ensure output is flushed immediately
                                if let Err(e) = io::stdout().flush() {
                                    log::error!("Failed to flush stdout: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            log::error!("PTY read error: {}", e);
                            break;
                        }
                    }
                }

                log::info!(
                    "PTY reader thread exiting (total bytes read: {})",
                    total_bytes
                );
            })
            .expect("Failed to spawn reader thread");

        Self {
            _child: child,
            reader,
            writer,
            _reader_thread: Some(handle),
        }
    }

    /// Write a byte to the PTY.
    ///
    /// This sends the byte to the shell process as input.
    pub fn write(&self, byte: u8) {
        let mut writer = self.writer.lock().unwrap();
        if let Err(e) = writer.write_all(&[byte]) {
            log::error!("Failed to write to PTY: {}", e);
        } else {
            if let Err(e) = writer.flush() {
                log::error!("Failed to flush PTY writer: {}", e);
            }
        }
    }

    /// Write a string to the PTY.
    ///
    /// This sends the string to the shell process as input.
    pub fn write_str(&self, s: &str) {
        let mut writer = self.writer.lock().unwrap();
        if let Err(e) = writer.write_all(s.as_bytes()) {
            log::error!("Failed to write to PTY: {}", e);
        } else {
            if let Err(e) = writer.flush() {
                log::error!("Failed to flush PTY writer: {}", e);
            }
        }
    }

    /// Write a byte slice to the PTY.
    ///
    /// This sends the bytes to the shell process as input.
    pub fn write_all(&self, bytes: &[u8]) {
        let mut writer = self.writer.lock().unwrap();
        if let Err(e) = writer.write_all(bytes) {
            log::error!("Failed to write to PTY: {}", e);
        } else {
            if let Err(e) = writer.flush() {
                log::error!("Failed to flush PTY writer: {}", e);
            }
        }
    }

    /// Get a clone of the writer for direct access if needed.
    pub fn writer(&self) -> Arc<Mutex<Box<dyn Write + Send>>> {
        Arc::clone(&self.writer)
    }
}

// When the PtySession is dropped, wait for the reader thread to finish.
impl Drop for PtySession {
    fn drop(&mut self) {
        log::info!("PtySession dropped, cleaning up resources");

        // Wait for the reader thread to finish with a timeout
        if let Some(handle) = self._reader_thread.take() {
            log::debug!("Waiting for PTY reader thread to finish...");

            // Wait with a timeout to avoid hanging indefinitely
            let timeout = Duration::from_secs(2);
            let start = std::time::Instant::now();

            loop {
                if handle.is_finished() {
                    log::debug!("Reader thread has finished");
                    let _ = handle.join();
                    break;
                }

                if start.elapsed() >= timeout {
                    log::warn!("Reader thread did not finish within timeout, continuing cleanup");
                    // Thread will be detached when handle is dropped
                    break;
                }

                // Small sleep to avoid busy-waiting
                std::thread::sleep(Duration::from_millis(50));
            }
        }

        // Child process will be automatically terminated when _child is dropped
        log::debug!("PtySession cleanup complete");
    }
}
