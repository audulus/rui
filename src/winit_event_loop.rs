use crate::*;

use futures::executor::block_on;
use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
};

use winit::{
    dpi::PhysicalSize,
    event::{
        ElementState, Event as WEvent, MouseButton as WMouseButton, Touch, TouchPhase,
        VirtualKeyCode, WindowEvent,
    },
    event_loop::{ControlFlow, EventLoop, EventLoopProxy},
    window::{Window, WindowBuilder},
};

type WorkQueue = VecDeque<Box<dyn FnOnce(&mut Context) + Send>>;

#[cfg(not(target_arch = "wasm32"))]
lazy_static! {
    /// Allows us to wake the event loop whenever we want.
    static ref GLOBAL_EVENT_LOOP_PROXY: Mutex<Option<EventLoopProxy<()>>> = Mutex::new(None);

    static ref GLOBAL_WORK_QUEUE: Mutex<WorkQueue> = Mutex::new(WorkQueue::new());
}

#[cfg(not(target_arch = "wasm32"))]
pub fn on_main(f: impl FnOnce(&mut Context) + Send + 'static) {
    GLOBAL_WORK_QUEUE.lock().unwrap().push_back(Box::new(f));

    // Wake up the event loop.
    let opt_proxy = GLOBAL_EVENT_LOOP_PROXY.lock().unwrap();
    if let Some(proxy) = &*opt_proxy {
        if let Err(err) = proxy.send_event(()) {
            println!("error waking up event loop: {:?}", err);
        }
    }
}

struct Setup {
    size: PhysicalSize<u32>,
    surface: wgpu::Surface,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

async fn setup(window: &Window) -> Setup {
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
                body.append_child(&web_sys::Element::from(window.canvas()))
                    .ok()
            })
            .expect("couldn't append canvas to document body");
    }

    // log::info!("Initializing the surface...");

    let backend = wgpu::util::backend_bits_from_env().unwrap_or_else(wgpu::Backends::all);
    let instance_desc = wgpu::InstanceDescriptor::default();

    let instance = wgpu::Instance::new(instance_desc);
    let (size, surface) = unsafe {
        let size = window.inner_size();
        let surface = instance.create_surface(&window);
        (size, surface.unwrap())
    };
    let adapter =
        wgpu::util::initialize_adapter_from_env_or_default(&instance, backend, Some(&surface))
            .await
            .expect("No suitable GPU adapters found on the system!");

    #[cfg(not(target_arch = "wasm32"))]
    {
        let adapter_info = adapter.get_info();
        println!("Using {} ({:?})", adapter_info.name, adapter_info.backend);
    }

    let trace_dir = std::env::var("WGPU_TRACE");
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::default(),
                limits: wgpu::Limits::default(),
            },
            trace_dir.ok().as_ref().map(std::path::Path::new),
        )
        .await
        .expect("Unable to find a suitable GPU adapter!");

    Setup {
        size,
        surface,
        adapter,
        device,
        queue,
    }
}

fn process_event(cx: &mut Context, view: &impl View, event: &Event, window: &Window) {
    cx.process(view, event);

    if cx.grab_cursor && !cx.prev_grab_cursor {
        println!("grabbing cursor");
        window
            .set_cursor_grab(winit::window::CursorGrabMode::Locked)
            .or_else(|_e| window.set_cursor_grab(winit::window::CursorGrabMode::Confined))
            .unwrap();
        window.set_cursor_visible(false);
    }

    if !cx.grab_cursor && cx.prev_grab_cursor {
        println!("releasing cursor");
        window
            .set_cursor_grab(winit::window::CursorGrabMode::None)
            .unwrap();
        window.set_cursor_visible(true);
    }

    cx.prev_grab_cursor = cx.grab_cursor;
}

/// Call this function to run your UI.
pub fn rui(view: impl View) {
    let event_loop = EventLoop::new();

    let mut window_title = String::from("rui");
    let builder = WindowBuilder::new().with_title(&window_title);
    let window = builder.build(&event_loop).unwrap();

    let setup = block_on(setup(&window));
    let surface = setup.surface;
    let device = Arc::new(setup.device);
    let size = setup.size;
    let adapter = setup.adapter;
    let queue = Arc::new(setup.queue);

    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface.get_capabilities(&adapter).formats[0],
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
        view_formats: vec![],
    };
    surface.configure(&device, &config);

    #[cfg(not(target_arch = "wasm32"))]
    {
        *GLOBAL_EVENT_LOOP_PROXY.lock().unwrap() = Some(event_loop.create_proxy());
    }

    let mut vger = Vger::new(device.clone(), queue.clone(), config.format);
    let mut cx = Context::new();
    let mut mouse_position = LocalPoint::zero();

    let mut commands: Vec<CommandInfo> = Vec::new();
    let mut command_map = HashMap::new();
    cx.commands(&view, &mut commands);

    {
        // So we can infer a type for CommandMap when winit is enabled.
        command_map.insert("", "");
    }

    let mut access_nodes = vec![];

    event_loop.run(move |event, _, control_flow| {
        // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
        // dispatched any events. This is ideal for games and similar applications.
        // *control_flow = ControlFlow::Poll;

        // ControlFlow::Wait pauses the event loop if no events are available to process.
        // This is ideal for non-game applications that only update in response to user
        // input, and uses significantly less power/CPU time than ControlFlow::Poll.
        *control_flow = ControlFlow::Wait;

        match event {
            WEvent::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                println!("The close button was pressed; stopping");
                *control_flow = ControlFlow::Exit
            }
            WEvent::WindowEvent {
                event:
                    WindowEvent::Resized(size)
                    | WindowEvent::ScaleFactorChanged {
                        new_inner_size: &mut size,
                        ..
                    },
                ..
            } => {
                // println!("Resizing to {:?}", size);
                config.width = size.width.max(1);
                config.height = size.height.max(1);
                surface.configure(&device, &config);
                window.request_redraw();
            }
            WEvent::UserEvent(_) => {
                // println!("received user event");

                // Process the work queue.
                #[cfg(not(target_arch = "wasm32"))]
                {
                    while let Some(f) = GLOBAL_WORK_QUEUE.lock().unwrap().pop_front() {
                        f(&mut cx);
                    }
                }
            }
            WEvent::MainEventsCleared => {
                // Application update code.

                // Queue a RedrawRequested event.
                //
                // You only need to call this if you've determined that you need to redraw, in
                // applications which do not always need to. Applications that redraw continuously
                // can just render here instead.

                let window_size = window.inner_size();
                let scale = window.scale_factor() as f32;
                // println!("window_size: {:?}", window_size);
                let width = window_size.width as f32 / scale;
                let height = window_size.height as f32 / scale;

                if cx.update(&view, &mut vger, &mut access_nodes, [width, height].into()) {
                    window.request_redraw();
                }

                if cx.window_title != window_title {
                    window_title = cx.window_title.clone();
                    window.set_title(&cx.window_title);
                }
            }
            WEvent::RedrawRequested(_) => {
                // Redraw the application.
                //
                // It's preferable for applications that do not render continuously to render in
                // this event rather than in MainEventsCleared, since rendering in here allows
                // the program to gracefully handle redraws requested by the OS.

                let window_size = window.inner_size();
                let scale = window.scale_factor() as f32;
                // println!("window_size: {:?}", window_size);
                let width = window_size.width as f32 / scale;
                let height = window_size.height as f32 / scale;

                // println!("RedrawRequested");
                cx.render(
                    RenderInfo {
                        device: &device,
                        surface: &surface,
                        config: &config,
                        queue: &queue,
                    },
                    &view,
                    &mut vger,
                    [width, height].into(),
                    scale,
                );
            }
            WEvent::WindowEvent {
                event: WindowEvent::MouseInput { state, button, .. },
                ..
            } => {
                match state {
                    ElementState::Pressed => {
                        cx.mouse_button = match button {
                            WMouseButton::Left => Some(MouseButton::Left),
                            WMouseButton::Right => Some(MouseButton::Right),
                            WMouseButton::Middle => Some(MouseButton::Center),
                            _ => None,
                        };
                        let event = Event::TouchBegin {
                            id: 0,
                            position: mouse_position,
                        };
                        process_event(&mut cx, &view, &event, &window)
                    }
                    ElementState::Released => {
                        cx.mouse_button = None;
                        let event = Event::TouchEnd {
                            id: 0,
                            position: mouse_position,
                        };
                        process_event(&mut cx, &view, &event, &window)
                    }
                };
            }
            WEvent::WindowEvent {
                window_id,
                event:
                    WindowEvent::Touch(Touch {
                        phase, location, ..
                    }),
                ..
            } => {
                // Do not handle events from other windows.
                if window_id != window.id() {
                    return;
                }

                let scale = window.scale_factor() as f32;
                let position = [
                    location.x as f32 / scale,
                    (config.height as f32 - location.y as f32) / scale,
                ]
                .into();

                let delta = position - cx.previous_position[0];

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
                    process_event(&mut cx, &view, &event, &window);
                }
            }
            WEvent::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                let scale = window.scale_factor() as f32;
                mouse_position = [
                    position.x as f32 / scale,
                    (config.height as f32 - position.y as f32) / scale,
                ]
                .into();
                // let event = Event::TouchMove {
                //     id: 0,
                //     position: mouse_position,
                // };
                // process_event(&mut cx, &view, &event, &window)
            }

            WEvent::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } => {
                if input.state == ElementState::Pressed {
                    if let Some(code) = input.virtual_keycode {
                        let key = match code {
                            // VirtualKeyCode::Character(c) => Some(Key::Character(c)),
                            VirtualKeyCode::Key1 => Some(Key::Character('1')),
                            VirtualKeyCode::Key2 => Some(Key::Character('2')),
                            VirtualKeyCode::Key3 => Some(Key::Character('3')),
                            VirtualKeyCode::Key4 => Some(Key::Character('4')),
                            VirtualKeyCode::Key5 => Some(Key::Character('5')),
                            VirtualKeyCode::Key6 => Some(Key::Character('6')),
                            VirtualKeyCode::Key7 => Some(Key::Character('7')),
                            VirtualKeyCode::Key8 => Some(Key::Character('8')),
                            VirtualKeyCode::Key9 => Some(Key::Character('9')),
                            VirtualKeyCode::Key0 => Some(Key::Character('0')),
                            VirtualKeyCode::A => {
                                Some(Key::Character(if cx.key_mods.shift { 'A' } else { 'a' }))
                            }
                            VirtualKeyCode::B => {
                                Some(Key::Character(if cx.key_mods.shift { 'B' } else { 'b' }))
                            }
                            VirtualKeyCode::C => {
                                Some(Key::Character(if cx.key_mods.shift { 'C' } else { 'c' }))
                            }
                            VirtualKeyCode::D => {
                                Some(Key::Character(if cx.key_mods.shift { 'D' } else { 'd' }))
                            }
                            VirtualKeyCode::E => {
                                Some(Key::Character(if cx.key_mods.shift { 'E' } else { 'e' }))
                            }
                            VirtualKeyCode::F => {
                                Some(Key::Character(if cx.key_mods.shift { 'F' } else { 'f' }))
                            }
                            VirtualKeyCode::G => {
                                Some(Key::Character(if cx.key_mods.shift { 'G' } else { 'g' }))
                            }
                            VirtualKeyCode::H => {
                                Some(Key::Character(if cx.key_mods.shift { 'H' } else { 'h' }))
                            }
                            VirtualKeyCode::I => {
                                Some(Key::Character(if cx.key_mods.shift { 'I' } else { 'i' }))
                            }
                            VirtualKeyCode::J => {
                                Some(Key::Character(if cx.key_mods.shift { 'J' } else { 'j' }))
                            }
                            VirtualKeyCode::K => {
                                Some(Key::Character(if cx.key_mods.shift { 'K' } else { 'k' }))
                            }
                            VirtualKeyCode::L => {
                                Some(Key::Character(if cx.key_mods.shift { 'L' } else { 'l' }))
                            }
                            VirtualKeyCode::M => {
                                Some(Key::Character(if cx.key_mods.shift { 'M' } else { 'm' }))
                            }
                            VirtualKeyCode::N => {
                                Some(Key::Character(if cx.key_mods.shift { 'N' } else { 'n' }))
                            }
                            VirtualKeyCode::O => {
                                Some(Key::Character(if cx.key_mods.shift { 'O' } else { 'o' }))
                            }
                            VirtualKeyCode::P => {
                                Some(Key::Character(if cx.key_mods.shift { 'P' } else { 'p' }))
                            }
                            VirtualKeyCode::Q => {
                                Some(Key::Character(if cx.key_mods.shift { 'Q' } else { 'q' }))
                            }
                            VirtualKeyCode::R => {
                                Some(Key::Character(if cx.key_mods.shift { 'R' } else { 'r' }))
                            }
                            VirtualKeyCode::S => {
                                Some(Key::Character(if cx.key_mods.shift { 'S' } else { 's' }))
                            }
                            VirtualKeyCode::T => {
                                Some(Key::Character(if cx.key_mods.shift { 'T' } else { 't' }))
                            }
                            VirtualKeyCode::U => {
                                Some(Key::Character(if cx.key_mods.shift { 'U' } else { 'u' }))
                            }
                            VirtualKeyCode::V => {
                                Some(Key::Character(if cx.key_mods.shift { 'V' } else { 'v' }))
                            }
                            VirtualKeyCode::W => {
                                Some(Key::Character(if cx.key_mods.shift { 'W' } else { 'w' }))
                            }
                            VirtualKeyCode::X => {
                                Some(Key::Character(if cx.key_mods.shift { 'X' } else { 'x' }))
                            }
                            VirtualKeyCode::Y => {
                                Some(Key::Character(if cx.key_mods.shift { 'Y' } else { 'y' }))
                            }
                            VirtualKeyCode::Z => {
                                Some(Key::Character(if cx.key_mods.shift { 'Z' } else { 'z' }))
                            }
                            VirtualKeyCode::Semicolon => Some(Key::Character(';')),
                            VirtualKeyCode::Colon => Some(Key::Character(';')),
                            VirtualKeyCode::Caret => Some(Key::Character('^')),
                            VirtualKeyCode::Asterisk => Some(Key::Character('*')),
                            VirtualKeyCode::Period => Some(Key::Character('.')),
                            VirtualKeyCode::Comma => Some(Key::Character(',')),
                            VirtualKeyCode::Return => Some(Key::Enter),
                            VirtualKeyCode::Tab => Some(Key::Tab),
                            VirtualKeyCode::Space => Some(Key::Space),
                            VirtualKeyCode::Down => Some(Key::ArrowDown),
                            VirtualKeyCode::Left => Some(Key::ArrowLeft),
                            VirtualKeyCode::Right => Some(Key::ArrowRight),
                            VirtualKeyCode::Up => Some(Key::ArrowUp),
                            VirtualKeyCode::End => Some(Key::End),
                            VirtualKeyCode::Home => Some(Key::Home),
                            VirtualKeyCode::PageDown => Some(Key::PageDown),
                            VirtualKeyCode::PageUp => Some(Key::PageUp),
                            VirtualKeyCode::Back => Some(Key::Backspace),
                            VirtualKeyCode::Delete => Some(Key::Delete),
                            VirtualKeyCode::Escape => Some(Key::Escape),
                            VirtualKeyCode::F1 => Some(Key::F1),
                            VirtualKeyCode::F2 => Some(Key::F2),
                            VirtualKeyCode::F3 => Some(Key::F3),
                            VirtualKeyCode::F4 => Some(Key::F4),
                            VirtualKeyCode::F5 => Some(Key::F5),
                            VirtualKeyCode::F6 => Some(Key::F6),
                            VirtualKeyCode::F7 => Some(Key::F7),
                            VirtualKeyCode::F8 => Some(Key::F8),
                            VirtualKeyCode::F9 => Some(Key::F9),
                            VirtualKeyCode::F10 => Some(Key::F10),
                            VirtualKeyCode::F11 => Some(Key::F11),
                            VirtualKeyCode::F12 => Some(Key::F12),
                            _ => None,
                        };

                        if let Some(key) = key {
                            cx.process(&view, &Event::Key(key))
                        }
                    }
                }
            }

            WEvent::WindowEvent {
                event: WindowEvent::ModifiersChanged(mods),
                ..
            } => {
                cx.key_mods = KeyboardModifiers {
                    shift: mods.shift(),
                    control: mods.ctrl(),
                    alt: mods.alt(),
                    command: mods.logo(),
                };
            }

            WEvent::DeviceEvent {
                event: winit::event::DeviceEvent::MouseMotion { delta },
                ..
            } => {
                // Flip y coordinate.
                let d: LocalOffset = [delta.0 as f32, -delta.1 as f32].into();

                let event = Event::TouchMove {
                    id: 0,
                    position: mouse_position,
                    delta: d,
                };

                process_event(&mut cx, &view, &event, &window);
            }
            _ => (),
        }
    });
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
