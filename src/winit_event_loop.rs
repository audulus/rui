use crate::timing::timed;
use crate::*;

use futures::executor::block_on;
use std::sync::Arc;
#[cfg(not(target_arch = "wasm32"))]
use std::{collections::VecDeque, sync::Mutex};

use euclid::Point2D;
#[cfg(not(target_arch = "wasm32"))]
use winit::event_loop::EventLoopProxy;
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{
        DeviceEvent, DeviceId, ElementState as wElementState, KeyEvent as WKeyEvent,
        MouseButton as WMouseButton, MouseScrollDelta, Touch, TouchPhase, WindowEvent,
    },
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard,
    window::{Window, WindowId},
};

/// How far one notch of a mouse wheel scrolls, in logical pixels.
const WHEEL_LINE: f32 = 40.0;

/// The main window of an app run with `rui_with_config`.
#[derive(Clone, Debug)]
pub struct WindowConfig {
    /// The window title.
    pub title: String,
    /// The inner size in logical pixels, or the platform default.
    pub size: Option<[f32; 2]>,
    /// Whether the window has a title bar and borders.
    pub decorations: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: String::from("rui"),
            size: None,
            decorations: true,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
type WorkQueue = VecDeque<Box<dyn FnOnce(&mut Context) + Send>>;

#[cfg(not(target_arch = "wasm32"))]
/// Allows us to wake the event loop whenever we want.
static GLOBAL_EVENT_LOOP_PROXY: Mutex<Option<EventLoopProxy<()>>> = Mutex::new(None);

#[cfg(not(target_arch = "wasm32"))]
static GLOBAL_WORK_QUEUE: Mutex<WorkQueue> = Mutex::new(WorkQueue::new());

/// Runs `f` on the main thread, with the `Context`, as for updating state from a background
/// thread. By the time `f` runs, the view whose state a `StateHandle` refers to may be gone; use
/// `Context::try_get_mut` when it might be.
#[cfg(not(target_arch = "wasm32"))]
pub fn on_main(f: impl FnOnce(&mut Context) + Send + 'static) {
    GLOBAL_WORK_QUEUE.lock().unwrap().push_back(Box::new(f));

    // Wake up the event loop.
    let opt_proxy = GLOBAL_EVENT_LOOP_PROXY.lock().unwrap();
    if let Some(proxy) = &*opt_proxy
        && let Err(err) = proxy.send_event(())
    {
        log::debug!("error waking up event loop: {:?}", err);
    }
}

/// Runs the closures passed to `on_main` so far.
#[cfg(not(target_arch = "wasm32"))]
fn run_work_queue(cx: &mut Context) {
    // Take the queued closures before running them, so the lock isn't held while they run and
    // they can call `on_main` themselves. What they queue runs on the next wake-up, which
    // `on_main` sends, so a closure that queues itself again doesn't keep the event loop from
    // running.
    let batch = std::mem::take(&mut *GLOBAL_WORK_QUEUE.lock().unwrap());
    for f in batch {
        f(cx);
    }
}

struct DrawContext {
    surface: wgpu::Surface<'static>,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    config: wgpu::SurfaceConfiguration,
    vger: Vger,
}

async fn setup(window: Arc<Window>) -> DrawContext {
    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowExtWebSys;
        let query_string = web_sys::window().unwrap().location().search().unwrap();
        let level: log::Level = parse_url_query_string(&query_string, "RUST_LOG")
            .map(|x| x.parse().ok())
            .flatten()
            .unwrap_or(log::Level::Error);
        console_log::init_with_level(level).expect("could not initialize logger");
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        // On wasm, append the canvas to the document body
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.body())
            .and_then(|body| {
                body.append_child(&web_sys::Element::from(window.canvas()?))
                    .ok()
            })
            .expect("couldn't append canvas to document body");
    }

    // log::info!("Initializing the surface...");

    let instance_desc = wgpu::InstanceDescriptor::default();

    let instance = wgpu::Instance::new(instance_desc);
    let size = window.inner_size();
    let surface = instance
        .create_surface(window)
        .expect("Failed to create surface!");
    let adapter = wgpu::util::initialize_adapter_from_env_or_default(&instance, Some(&surface))
        .await
        .expect("No suitable GPU adapters found on the system!");

    #[cfg(not(target_arch = "wasm32"))]
    {
        let adapter_info = adapter.get_info();
        log::debug!("Using {} ({:?})", adapter_info.name, adapter_info.backend);
    }

    let trace_dir = std::env::var("WGPU_TRACE");
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::default(),
                required_limits: wgpu::Limits::default(),
            },
            trace_dir.ok().as_ref().map(std::path::Path::new),
        )
        .await
        .expect("Unable to find a suitable GPU adapter!");
    let device = Arc::new(device);
    let queue = Arc::new(queue);

    // Prefer an sRGB format, so colors look the same on every platform: vger blends in linear
    // space and the surface encodes to sRGB.
    let formats = surface.get_capabilities(&adapter).formats;
    let format = formats
        .iter()
        .copied()
        .find(|format| format.is_srgb())
        .unwrap_or(formats[0]);

    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
        desired_maximum_frame_latency: 2,
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
        view_formats: vec![],
    };
    surface.configure(&device, &config);

    let vger = Vger::new(device.clone(), queue.clone(), config.format);

    DrawContext {
        surface,
        device,
        queue,
        config,
        vger,
    }
}

fn process_event(cx: &mut Context, view: &impl View, event: &Event, window: &Window) {
    let label = match event {
        Event::TouchBegin { .. } => "event:touch-begin",
        Event::TouchMove { .. } => "event:touch-move",
        Event::TouchEnd { .. } => "event:touch-end",
        Event::Scroll { .. } => "event:scroll",
        _ => "event:other",
    };
    timed(label, || cx.process(view, event));

    if cx.grab_cursor && !cx.prev_grab_cursor {
        log::debug!("grabbing cursor");
        window
            .set_cursor_grab(winit::window::CursorGrabMode::Locked)
            .or_else(|_e| window.set_cursor_grab(winit::window::CursorGrabMode::Confined))
            .unwrap();
        window.set_cursor_visible(false);
    }

    if !cx.grab_cursor && cx.prev_grab_cursor {
        log::debug!("releasing cursor");
        window
            .set_cursor_grab(winit::window::CursorGrabMode::None)
            .unwrap();
        window.set_cursor_visible(true);
    }

    cx.prev_grab_cursor = cx.grab_cursor;
}

struct EventHandler<T>
where
    T: View,
{
    title: Arc<str>,
    config: WindowConfig,
    running: bool,
    // The GPU resources, if running.
    context: Option<DrawContext>,
    // The event handling loop is terminated when the main window is closed.
    // We can trigger this by dropping the window, so we wrap it in the Option
    // type.  This is a bit of a hack, but it works.
    window: Option<Arc<Window>>,
    // The event system does not expose the cursor position on-demand.
    // We track all the mouse movement events to make this easier to access
    // by event handlers.
    mouse_position: Point2D<f32, LocalSpace>,
    cx: Context,
    view: T,
    access_nodes: Vec<(accesskit::NodeId, accesskit::Node)>,
}

impl<T> ApplicationHandler for EventHandler<T>
where
    T: View,
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Called when the application is brought into focus.  The window and
        // associated GPU resources need to be reallocated on resume.

        // On some platforms, namely wasm32 + webgl2, the window is not yet
        // ready to create the rendering surface when Event::Resumed is
        // received.  We therefore just record the fact that the we're in the
        // running state.
        self.running = true;

        // Create the main window.
        let mut window_attributes = Window::default_attributes()
            .with_title(self.title.to_string())
            .with_decorations(self.config.decorations);
        if let Some([width, height]) = self.config.size {
            window_attributes = window_attributes.with_inner_size(LogicalSize::new(width, height));
        }
        self.window = match event_loop.create_window(window_attributes) {
            Err(e) => {
                log::error!("Error creating window: {:?}", e);
                return;
            }
            Ok(window) => Some(Arc::new(window)),
        };
        let window = self.window.as_ref().unwrap();

        // Set up the rendering context.
        self.context = Some(block_on(setup(window.clone())));
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
        // dispatched any events. This is ideal for games and similar applications.
        // event_loop.set_control_flow(ControlFlow::Poll);

        // ControlFlow::Wait pauses the event loop if no events are available to process.
        // This is ideal for non-game applications that only update in response to user
        // input, and uses significantly less power/CPU time than ControlFlow::Poll.
        event_loop.set_control_flow(ControlFlow::Wait);

        match event {
            WindowEvent::CloseRequested => {
                log::debug!("The close button was pressed; stopping");
                event_loop.exit()
            }
            WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => {
                if let (Some(window), Some(context)) = (&self.window, &mut self.context) {
                    let size = window.inner_size();
                    // log::debug!("Resizing to {:?}", size);
                    context.config.width = size.width.max(1);
                    context.config.height = size.height.max(1);
                    context.surface.configure(&context.device, &context.config);
                    window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                // Redraw the application.
                //
                // It's preferable for applications that do not render continuously to render in
                // this event rather than in MainEventsCleared, since rendering in here allows
                // the program to gracefully handle redraws requested by the OS.

                if let (Some(window), Some(context)) = (&self.window, &mut self.context) {
                    let window_size = window.inner_size();
                    let scale = window.scale_factor() as f32;
                    // log::debug!("window_size: {:?}", window_size);
                    let width = window_size.width as f32 / scale;
                    let height = window_size.height as f32 / scale;

                    // log::debug!("RedrawRequested");
                    self.cx.render(
                        RenderInfo {
                            device: &context.device,
                            surface: &context.surface,
                            config: &context.config,
                            queue: &context.queue,
                        },
                        &self.view,
                        &mut context.vger,
                        [width, height].into(),
                        scale,
                    );
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                match state {
                    wElementState::Pressed => {
                        self.cx.mouse_button = match button {
                            WMouseButton::Left => Some(MouseButton::Left),
                            WMouseButton::Right => Some(MouseButton::Right),
                            WMouseButton::Middle => Some(MouseButton::Center),
                            _ => None,
                        };

                        match button {
                            WMouseButton::Left => self.cx.mouse_buttons.left = true,
                            WMouseButton::Right => self.cx.mouse_buttons.right = true,
                            WMouseButton::Middle => self.cx.mouse_buttons.middle = true,
                            _ => (),
                        };

                        self.cx.set_dirty();

                        if let Some(window) = &self.window {
                            let event = Event::TouchBegin {
                                id: 0,
                                position: self.mouse_position,
                            };
                            process_event(&mut self.cx, &self.view, &event, window)
                        }
                    }
                    wElementState::Released => {
                        self.cx.mouse_button = None;

                        match button {
                            WMouseButton::Left => self.cx.mouse_buttons.left = false,
                            WMouseButton::Right => self.cx.mouse_buttons.right = false,
                            WMouseButton::Middle => self.cx.mouse_buttons.middle = false,
                            _ => (),
                        };

                        self.cx.set_dirty();

                        if let Some(window) = &self.window {
                            let event = Event::TouchEnd {
                                id: 0,
                                position: self.mouse_position,
                            };
                            process_event(&mut self.cx, &self.view, &event, window)
                        }
                    }
                };
            }
            WindowEvent::Touch(Touch {
                phase, location, ..
            }) => {
                if let (Some(window), Some(context)) = (&self.window, &self.context) {
                    // Do not handle events from other windows.
                    if window_id != window.id() {
                        return;
                    }

                    let scale = window.scale_factor() as f32;
                    let position = [
                        location.x as f32 / scale,
                        (context.config.height as f32 - location.y as f32) / scale,
                    ]
                    .into();

                    let delta = position - self.cx.previous_position[0];

                    // TODO: Multi-Touch management
                    let event = match phase {
                        TouchPhase::Started => Some(Event::TouchBegin { id: 0, position }),
                        TouchPhase::Moved => Some(Event::TouchMove {
                            id: 0,
                            position,
                            delta,
                        }),
                        TouchPhase::Ended | TouchPhase::Cancelled => {
                            Some(Event::TouchEnd { id: 0, position })
                        }
                    };

                    if let Some(event) = event {
                        process_event(&mut self.cx, &self.view, &event, window);
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let (Some(window), Some(context)) = (&self.window, &self.context) {
                    let scale = window.scale_factor() as f32;
                    let position: LocalPoint = [
                        position.x as f32 / scale,
                        (context.config.height as f32 - position.y as f32) / scale,
                    ]
                    .into();
                    let delta = position - self.mouse_position;
                    self.mouse_position = position;

                    // While the cursor is grabbed, motion comes from device events instead.
                    if !self.cx.grab_cursor {
                        let event = Event::TouchMove {
                            id: 0,
                            position,
                            delta,
                        };
                        process_event(&mut self.cx, &self.view, &event, window);
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                if let Some(window) = &self.window {
                    let scale = window.scale_factor() as f32;
                    // winit's y is positive when the content should move down; rui's y is up.
                    let delta: LocalOffset = match delta {
                        MouseScrollDelta::LineDelta(x, y) => [x * WHEEL_LINE, -y * WHEEL_LINE],
                        MouseScrollDelta::PixelDelta(d) => {
                            [d.x as f32 / scale, -(d.y as f32) / scale]
                        }
                    }
                    .into();
                    let event = Event::Scroll {
                        position: self.mouse_position,
                        delta,
                    };
                    process_event(&mut self.cx, &self.view, &event, window);
                }
            }

            WindowEvent::KeyboardInput {
                event: key_event @ WKeyEvent { .. },
                ..
            } => {
                let key = match key_event.logical_key {
                    keyboard::Key::Named(keyboard::NamedKey::Enter) => Some(Key::Enter),
                    keyboard::Key::Named(keyboard::NamedKey::Tab) => Some(Key::Tab),
                    keyboard::Key::Named(keyboard::NamedKey::Space) => Some(Key::Space),
                    keyboard::Key::Named(keyboard::NamedKey::ArrowDown) => Some(Key::ArrowDown),
                    keyboard::Key::Named(keyboard::NamedKey::ArrowLeft) => Some(Key::ArrowLeft),
                    keyboard::Key::Named(keyboard::NamedKey::ArrowRight) => Some(Key::ArrowRight),
                    keyboard::Key::Named(keyboard::NamedKey::ArrowUp) => Some(Key::ArrowUp),
                    keyboard::Key::Named(keyboard::NamedKey::End) => Some(Key::End),
                    keyboard::Key::Named(keyboard::NamedKey::Home) => Some(Key::Home),
                    keyboard::Key::Named(keyboard::NamedKey::PageDown) => Some(Key::PageDown),
                    keyboard::Key::Named(keyboard::NamedKey::PageUp) => Some(Key::PageUp),
                    keyboard::Key::Named(keyboard::NamedKey::Backspace) => Some(Key::Backspace),
                    keyboard::Key::Named(keyboard::NamedKey::Delete) => Some(Key::Delete),
                    keyboard::Key::Named(keyboard::NamedKey::Escape) => Some(Key::Escape),
                    keyboard::Key::Named(keyboard::NamedKey::F1) => Some(Key::F1),
                    keyboard::Key::Named(keyboard::NamedKey::F2) => Some(Key::F2),
                    keyboard::Key::Named(keyboard::NamedKey::F3) => Some(Key::F3),
                    keyboard::Key::Named(keyboard::NamedKey::F4) => Some(Key::F4),
                    keyboard::Key::Named(keyboard::NamedKey::F5) => Some(Key::F5),
                    keyboard::Key::Named(keyboard::NamedKey::F6) => Some(Key::F6),
                    keyboard::Key::Named(keyboard::NamedKey::F7) => Some(Key::F7),
                    keyboard::Key::Named(keyboard::NamedKey::F8) => Some(Key::F8),
                    keyboard::Key::Named(keyboard::NamedKey::F9) => Some(Key::F9),
                    keyboard::Key::Named(keyboard::NamedKey::F10) => Some(Key::F10),
                    keyboard::Key::Named(keyboard::NamedKey::F11) => Some(Key::F11),
                    keyboard::Key::Named(keyboard::NamedKey::F12) => Some(Key::F12),
                    keyboard::Key::Character(str) => str.chars().next().map(Key::Character),
                    _ => None,
                };

                if let (Some(key), wElementState::Pressed) = (key, key_event.state) {
                    timed("event:key", || {
                        self.cx.process(&self.view, &Event::Key(key))
                    })
                }

                if let (Some(key), wElementState::Released) = (key, key_event.state) {
                    timed("event:key-released", || {
                        self.cx.process(&self.view, &Event::KeyReleased(key))
                    })
                }
            }

            WindowEvent::ModifiersChanged(mods) => {
                self.cx.key_mods = KeyboardModifiers {
                    shift: !(mods.state() & keyboard::ModifiersState::SHIFT).is_empty(),
                    control: !(mods.state() & keyboard::ModifiersState::CONTROL).is_empty(),
                    alt: !(mods.state() & keyboard::ModifiersState::ALT).is_empty(),
                    command: !(mods.state() & keyboard::ModifiersState::SUPER).is_empty(),
                };
            }

            _ => (),
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, _event: ()) {
        // log::debug!("received user event");

        // Process the work queue.
        #[cfg(not(target_arch = "wasm32"))]
        {
            run_work_queue(&mut self.cx);
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        // Without a grab, `CursorMoved` reports the motion; device events would also report
        // motion outside the window, and some platforms (such as Wayland compositors without
        // relative pointers) never send them.
        if let DeviceEvent::MouseMotion { delta, .. } = event
            && self.cx.grab_cursor
        {
            // Flip y coordinate.
            let d: LocalOffset = [delta.0 as f32, -delta.1 as f32].into();

            let event = Event::TouchMove {
                id: 0,
                position: self.mouse_position,
                delta: d,
            };

            if let Some(window) = &self.window {
                process_event(&mut self.cx, &self.view, &event, window);
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Application update code.

        // Queue a RedrawRequested event.
        //
        // You only need to call this if you've determined that you need to
        // redraw, in applications which do not always need to. Applications
        // that redraw continuously can just render here instead.

        if let (Some(window), Some(context)) = (&self.window, &mut self.context) {
            let window_size = window.inner_size();
            let scale = window.scale_factor() as f32;
            // log::debug!("window_size: {:?}", window_size);
            let width = window_size.width as f32 / scale;
            let height = window_size.height as f32 / scale;

            // Timed only when it does work, so idle passes print nothing.
            let start = crate::timing::start();
            if self.cx.update(
                &self.view,
                &mut context.vger,
                &mut self.access_nodes,
                [width, height].into(),
            ) {
                crate::timing::stop("update", start);
                window.request_redraw();
            }

            if self.cx.window_title != self.title {
                self.title = self.cx.window_title.clone();
                window.set_title(&self.cx.window_title);
            }
        }
    }
}

/// Call this function to run your UI.
pub fn rui(view: impl View) {
    rui_with_config(view, WindowConfig::default())
}

/// Runs your UI in a window with the given title, size and decorations.
pub fn rui_with_config(view: impl View, config: WindowConfig) {
    let event_loop = EventLoop::new().unwrap();

    let mut app = EventHandler {
        title: config.title.clone().into(),
        config,
        running: false,
        context: None,
        window: None,
        mouse_position: LocalPoint::zero(),
        cx: Context::new(),
        view,
        access_nodes: vec![],
    };
    app.cx.window_title = app.title.clone();

    #[cfg(not(target_arch = "wasm32"))]
    {
        *GLOBAL_EVENT_LOOP_PROXY.lock().unwrap() = Some(event_loop.create_proxy());
    }

    if let Err(e) = event_loop.run_app(&mut app) {
        log::error!("Error exiting event loop: {:?}", e);
    };
}

#[cfg(target_arch = "wasm32")]
/// Parse the query string as returned by `web_sys::window()?.location().search()?` and get a
/// specific key out of it.
pub fn parse_url_query_string<'a>(query: &'a str, search_key: &str) -> Option<&'a str> {
    let query_string = query.strip_prefix('?')?;

    for pair in query_string.split('&') {
        let mut pair = pair.split('=');
        let key = pair.next()?;
        let value = pair.next()?;

        if key == search_key {
            return Some(value);
        }
    }

    None
}

pub trait Run: View + Sized {
    fn run(self) {
        rui(self)
    }
}

impl<V: View> Run for V {}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    /// Held by the tests that use the global `on_main` queue, which test threads share.
    static QUEUE: Mutex<()> = Mutex::new(());

    fn lock_queue() -> std::sync::MutexGuard<'static, ()> {
        let guard = QUEUE.lock().unwrap_or_else(|e| e.into_inner());
        GLOBAL_WORK_QUEUE.lock().unwrap().clear();
        guard
    }

    fn layout(cx: &mut Context, view: &impl View) {
        let root = cx.root_id();
        view.layout(
            root,
            &mut LayoutArgs {
                sz: [100.0, 100.0].into(),
                cx,
                text_bounds: &mut |_, _, _| LocalRect::zero(),
            },
        );
    }

    fn items(ids: Vec<i32>) -> impl View {
        list(ids, |_| state(|| 0i32, |_, _| rectangle()))
    }

    #[test]
    fn test_on_main_across_a_removed_view() {
        let _queue = lock_queue();
        let mut cx = Context::new();
        layout(&mut cx, &items(vec![1]));
        let s = StateHandle::<i32>::new(cx.id_at(&[0, hh(&1)]));

        // A background task holds the handle...
        let (wrote, saw) = (
            std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true)),
        );
        let (w, v) = (wrote.clone(), saw.clone());
        std::thread::spawn(move || {
            on_main(move |cx| {
                v.store(cx.try_get(s).is_some(), std::sync::atomic::Ordering::SeqCst);
                if let Some(x) = cx.try_get_mut(s) {
                    *x = 5;
                    w.store(true, std::sync::atomic::Ordering::SeqCst);
                }
            })
        })
        .join()
        .unwrap();

        // ...while its view is removed and its slot reused by another.
        layout(&mut cx, &items(vec![]));
        layout(&mut cx, &items(vec![2, 3]));
        let others = [2, 3].map(|k| StateHandle::<i32>::new(cx.id_at(&[0, hh(&k)])));
        let reused = (1..10).any(|generation| {
            cx.is_live(ViewId {
                index: s.id.index,
                generation,
            })
        });
        assert!(reused && !cx.is_live(s.id));

        run_work_queue(&mut cx);
        assert!(!saw.load(std::sync::atomic::Ordering::SeqCst));
        assert!(!wrote.load(std::sync::atomic::Ordering::SeqCst));
        assert_eq!(others.map(|o| cx[o]), [0, 0]);
    }

    /// Queues a closure that counts its runs and queues itself again.
    fn tick(count: std::sync::Arc<std::sync::atomic::AtomicUsize>) {
        on_main(move |_| {
            count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            tick(count);
        });
    }

    #[test]
    fn test_a_closure_that_queues_itself_runs_once_per_drain() {
        let _queue = lock_queue();
        let mut cx = Context::new();
        let count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        tick(count.clone());

        // Before the fix, this drain never returned.
        run_work_queue(&mut cx);
        assert_eq!(count.load(std::sync::atomic::Ordering::SeqCst), 1);
        run_work_queue(&mut cx);
        assert_eq!(count.load(std::sync::atomic::Ordering::SeqCst), 2);
    }
}
