use softbuffer::{Context, Surface};
use std::num::NonZeroU32;
use std::rc::Rc;
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoopProxy},
    keyboard::{Key, NamedKey},
    window::{Window, WindowAttributes, WindowId},
};

mod ansi;
mod buffer;
mod font;
mod grid;
mod pty;
use buffer::TerminalBuffer;
use font::FontRenderer;
use pty::PtySession;

/// Custom event type for triggering redraws.
#[derive(Debug, Clone, Copy)]
enum AppEvent {
    /// New PTY output available, trigger redraw
    NewOutput,
}

/// Application state with proper softbuffer resource management.
///
/// The `Context` must be kept alive for the entire lifetime of the `Surface`,
/// as the surface internally holds a reference to the context. Dropping the
/// context before the surface would cause undefined behavior.
struct Application {
    window: Option<Rc<Window>>,
    context: Option<Context<Rc<Window>>>,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
    pty: Option<PtySession>,
    buffer: Option<TerminalBuffer>,
    font: Option<FontRenderer>,
    proxy: Option<EventLoopProxy<AppEvent>>,
}

impl ApplicationHandler<AppEvent> for Application {
    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: AppEvent) {
        match event {
            AppEvent::NewOutput => {
                // Trigger redraw when new PTY output is available
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Same implementation as above
        let window_attributes = WindowAttributes::default()
            .with_title("My Terminal")
            .with_inner_size(LogicalSize::new(800, 600));

        match event_loop.create_window(window_attributes) {
            Ok(window) => {
                log::info!("Window created successfully");

                let window = Rc::new(window);

                // Create softbuffer context and surface
                let context = Context::new(window.clone())
                    .expect("Failed to create softbuffer context");
                let surface = Surface::new(&context, window.clone())
                    .expect("Failed to create softbuffer surface");

                self.window = Some(window);
                self.context = Some(context);
                self.surface = Some(surface);

                // Initialize font renderer
                log::info!("Initializing font renderer");
                let font = FontRenderer::with_size(14.0);
                self.font = Some(font);

                // Initialize terminal buffer
                log::info!("Initializing terminal buffer");
                let buffer = TerminalBuffer::new();
                let buffer_clone = buffer.clone();
                self.buffer = Some(buffer);

                // Initialize PTY session with buffer callback
                log::info!("Initializing PTY session");
                let proxy = self.proxy.clone().unwrap();
                let callback = std::sync::Arc::new(std::sync::Mutex::new(Box::new(move |data: &[u8]| {
                    buffer_clone.write(data);
                    // Trigger redraw when new data arrives
                    let _ = proxy.send_event(AppEvent::NewOutput);
                }) as Box<dyn Fn(&[u8]) + Send + Sync>));
                let pty = PtySession::with_output_callback(Some(callback));
                self.pty = Some(pty);

                // Request initial redraw
                self.window.as_ref().unwrap().request_redraw();
            }
            Err(err) => {
                log::error!("Error creating window: {err}");
                event_loop.exit();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                log::info!("Close requested");
                event_loop.exit();
            }
            WindowEvent::KeyboardInput {
                event: KeyEvent { logical_key, state, repeat, .. },
                ..
            } => {
                // Only handle key press events (not release or repeat)
                if state != ElementState::Pressed || repeat {
                    return;
                }

                // Check if Escape key was pressed
                if logical_key == Key::Named(NamedKey::Escape) {
                    log::info!("Escape key pressed, exiting...");
                    event_loop.exit();
                } else if let Some(pty) = &self.pty {
                    // Handle special keys
                    match &logical_key {
                        Key::Named(NamedKey::Enter) => {
                            // Send carriage return for Enter key
                            pty.write(b'\r');
                            log::debug!("Sent Enter key to PTY");
                        }
                        Key::Named(NamedKey::Backspace) => {
                            // Send backspace for Backspace key
                            pty.write(0x08); // ASCII backspace
                            log::debug!("Sent Backspace key to PTY");
                        }
                        Key::Named(NamedKey::Tab) => {
                            // Send tab for Tab key
                            pty.write(b'\t');
                            log::debug!("Sent Tab key to PTY");
                        }
                        Key::Character(c) => {
                            // Send character input
                            pty.write_str(c.as_str());
                            log::debug!("Sent '{}' to PTY", c);
                        }
                        _ => {
                            log::debug!("Unhandled key: {:?}", logical_key);
                        }
                    }
                }
            }
            WindowEvent::Resized(_) => {
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                // Draw terminal buffer to window
                if let (Some(window), Some(surface), Some(buffer), Some(font)) =
                    (&self.window, &mut self.surface, &self.buffer, &self.font)
                {
                    let size = window.inner_size();
                    let (Some(width), Some(height)) =
                        (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
                    else {
                        return;
                    };

                    surface.resize(width, height).expect("Failed to resize surface");

                    let mut buffer_surface = surface.buffer_mut().expect("Failed to get buffer");

                    // Fill with dark background (ARGB format)
                    buffer_surface.fill(0xff181818);

                    // Get buffer content and split into lines
                    let content = buffer.content();
                    let lines = font::split_lines(&content);

                    let width_val = width.get() as usize;
                    let height_val = height.get() as usize;

                    // Calculate visible lines
                    let char_height = font.char_height();
                    let visible_lines = height_val / char_height;

                    // Use a bright green color for text (ARGB)
                    let text_color = 0xff00ff00;

                    // Render the text lines
                    font.render_lines(
                        lines.into_iter(),
                        8, // Left padding
                        8, // Top padding
                        &mut buffer_surface,
                        width_val,
                        height_val,
                        text_color,
                        visible_lines,
                    );

                    buffer_surface.present().expect("Failed to present buffer");
                }
            }
            _ => {}
        }
    }
}

fn main() {
    // Initialize logger
    env_logger::init();

    // Create event loop with user event support
    let event_loop = winit::event_loop::EventLoop::with_user_event().build().unwrap();

    // Get proxy for sending custom events
    let proxy = event_loop.create_proxy();

    // Create and run application
    let mut app = Application {
        window: None,
        context: None,
        surface: None,
        pty: None,
        buffer: None,
        font: None,
        proxy: Some(proxy),
    };
    event_loop.run_app(&mut app).unwrap();
}
