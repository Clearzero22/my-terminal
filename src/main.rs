use softbuffer::{Context, Surface};
use std::num::NonZeroU32;
use std::rc::Rc;
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{KeyEvent, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{Key, NamedKey},
    window::{Window, WindowAttributes, WindowId},
};

mod pty;
use pty::PtySession;

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
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = WindowAttributes::default()
            .with_title("My Terminal")
            .with_inner_size(LogicalSize::new(800, 600));

        match event_loop.create_window(window_attributes) {
            Ok(window) => {
                log::info!("Window created successfully");

                let window = Rc::new(window);

                // Create softbuffer context and surface
                // The context must be stored to ensure it outlives the surface
                let context = Context::new(window.clone())
                    .expect("Failed to create softbuffer context");
                let surface = Surface::new(&context, window.clone())
                    .expect("Failed to create softbuffer surface");

                self.window = Some(window);
                self.context = Some(context);
                self.surface = Some(surface);

                // Initialize PTY session and start reader thread
                log::info!("Initializing PTY session");
                let pty = PtySession::new();
                pty.spawn_reader_thread();
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
                event: KeyEvent { logical_key, .. },
                ..
            } => {
                // Check if Escape key was pressed
                if logical_key == Key::Named(NamedKey::Escape) {
                    log::info!("Escape key pressed, exiting...");
                    event_loop.exit();
                }
            }
            WindowEvent::Resized(_) => {
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                // Draw to the window with a dark gray color
                if let (Some(window), Some(surface)) = (&self.window, &mut self.surface) {
                    let size = window.inner_size();
                    let (Some(width), Some(height)) =
                        (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
                    else {
                        return;
                    };

                    surface.resize(width, height).expect("Failed to resize surface");

                    let mut buffer = surface.buffer_mut().expect("Failed to get buffer");
                    // Fill with dark gray color (0xFF181818)
                    buffer.fill(0xff181818);
                    buffer.present().expect("Failed to present buffer");
                }
            }
            _ => {}
        }
    }
}

fn main() {
    // Initialize logger
    env_logger::init();

    // Create event loop
    let event_loop = winit::event_loop::EventLoop::new().unwrap();

    // Create and run application
    let mut app = Application {
        window: None,
        context: None,
        surface: None,
        pty: None,
    };
    event_loop.run_app(&mut app).unwrap();
}
