use crate::*;

use futures::executor::block_on;
use std::{
    collections::{HashMap, VecDeque},
    sync::Mutex,
};

#[cfg(feature = "tao")]
use tao::{
    accelerator::Accelerator,
    dpi::PhysicalSize,
    event::{ElementState, Event as WEvent, MouseButton as WMouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopProxy},
    keyboard::{Key as KeyPress, KeyCode, ModifiersState},
    menu::{MenuBar as Menu, MenuItem, MenuItemAttributes},
    window::{Window, WindowBuilder},
};

#[cfg(feature = "winit")]
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event as WEvent, MouseButton as WMouseButton, WindowEvent, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop, EventLoopProxy},
    window::{Window, WindowBuilder},
};

type WorkQueue = VecDeque<Box<dyn FnOnce(&mut Context) + Send>>;

lazy_static! {
    /// Allows us to wake the event loop whenever we want.
    static ref GLOBAL_EVENT_LOOP_PROXY: Mutex<Option<EventLoopProxy<()>>> = Mutex::new(None);

    static ref GLOBAL_WORK_QUEUE: Mutex<WorkQueue> = Mutex::new(WorkQueue::new());
}

fn wake_event_loop() {
    // Wake up the event loop.
    let opt_proxy = GLOBAL_EVENT_LOOP_PROXY.lock().unwrap();
    if let Some(proxy) = &*opt_proxy {
        if let Err(err) = proxy.send_event(()) {
            println!("error waking up event loop: {:?}", err);
        }
    }
}

pub fn on_main(f: impl FnOnce(&mut Context) + Send + 'static) {
    GLOBAL_WORK_QUEUE.lock().unwrap().push_back(Box::new(f));
    wake_event_loop();
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

    let instance = wgpu::Instance::new(backend);
    let (size, surface) = unsafe {
        let size = window.inner_size();
        let surface = instance.create_surface(&window);
        (size, surface)
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

#[cfg(feature = "tao")]
mod menus {
    use super::*;

    struct MenuItem2 {
        name: String,
        submenu: Vec<usize>,
        command: CommandInfo,
    }

    type CommandMap = HashMap<tao::menu::MenuId, String>;

    fn make_menu_rec(items: &Vec<MenuItem2>, i: usize, command_map: &mut CommandMap) -> Menu {
        let mut menu = Menu::new();

        if i == 0 {
            let mut app_menu = Menu::new();

            let app_name = match std::env::current_exe() {
                Ok(exe_path) => exe_path.file_name().unwrap().to_str().unwrap().to_string(),
                Err(_) => "rui".to_string(),
            };

            app_menu.add_native_item(MenuItem::About(app_name, Default::default()));
            app_menu.add_native_item(MenuItem::Quit);
            menu.add_submenu("rui", true, app_menu);
        }

        for j in &items[i].submenu {
            let item = &items[*j];
            if !item.submenu.is_empty() {
                menu.add_submenu(
                    item.name.as_str(),
                    true,
                    make_menu_rec(items, *j, command_map),
                );
            } else {
                let mut attrs = MenuItemAttributes::new(item.name.as_str());
                if let Some(key) = item.command.key {
                    let key_code = match key {
                        HotKey::KeyA => KeyCode::KeyA,
                        HotKey::KeyB => KeyCode::KeyB,
                        HotKey::KeyC => KeyCode::KeyC,
                        HotKey::KeyD => KeyCode::KeyD,
                        HotKey::KeyE => KeyCode::KeyE,
                        HotKey::KeyF => KeyCode::KeyF,
                        HotKey::KeyG => KeyCode::KeyG,
                        HotKey::KeyH => KeyCode::KeyH,
                        HotKey::KeyI => KeyCode::KeyI,
                        HotKey::KeyJ => KeyCode::KeyJ,
                        HotKey::KeyK => KeyCode::KeyK,
                        HotKey::KeyL => KeyCode::KeyL,
                        HotKey::KeyM => KeyCode::KeyM,
                        HotKey::KeyN => KeyCode::KeyN,
                        HotKey::KeyO => KeyCode::KeyO,
                        HotKey::KeyP => KeyCode::KeyP,
                        HotKey::KeyQ => KeyCode::KeyQ,
                        HotKey::KeyR => KeyCode::KeyR,
                        HotKey::KeyS => KeyCode::KeyS,
                        HotKey::KeyT => KeyCode::KeyT,
                        HotKey::KeyU => KeyCode::KeyU,
                        HotKey::KeyV => KeyCode::KeyV,
                        HotKey::KeyW => KeyCode::KeyW,
                        HotKey::KeyX => KeyCode::KeyX,
                        HotKey::KeyY => KeyCode::KeyY,
                        HotKey::KeyZ => KeyCode::KeyZ,
                    };

                    let accel = Accelerator::new(ModifiersState::SUPER, key_code);
                    attrs = attrs.with_accelerators(&accel);
                }
                let id = menu.add_item(attrs).id();
                command_map.insert(id, item.command.path.clone());
            }
        }

        menu
    }

    pub(crate) fn build_menubar(commands: &Vec<CommandInfo>, command_map: &mut CommandMap) -> Menu {
        let mut items: Vec<MenuItem2> = vec![MenuItem2 {
            name: "root".into(),
            submenu: vec![],
            command: CommandInfo {
                path: "".into(),
                key: None,
            },
        }];

        for command in commands {
            let mut v = 0;
            for name in command.path.split(':') {
                if let Some(item) = items[v].submenu.iter().find(|x| items[**x].name == name) {
                    v = *item;
                } else {
                    let n = items.len();
                    items[v].submenu.push(n);
                    v = n;
                    items.push(MenuItem2 {
                        name: name.into(),
                        submenu: vec![],
                        command: command.clone(),
                    });
                }
            }
        }

        make_menu_rec(&items, 0, command_map)
    }
}

/// Call this function to run your UI.
pub fn rui(view: impl View) {
    let event_loop = EventLoop::new();

    let mut window_title = String::from("rui");
    let builder = WindowBuilder::new().with_title(&window_title);
    let window = builder.build(&event_loop).unwrap();

    let setup = block_on(setup(&window));
    let surface = setup.surface;
    let device = setup.device;
    let size = setup.size;
    let adapter = setup.adapter;
    let queue = setup.queue;

    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface.get_preferred_format(&adapter).unwrap(),
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Mailbox,
    };
    surface.configure(&device, &config);

    *GLOBAL_EVENT_LOOP_PROXY.lock().unwrap() = Some(event_loop.create_proxy());

    let mut vger = Vger::new(&device, wgpu::TextureFormat::Bgra8UnormSrgb);
    let mut cx = Context::new();
    let mut mouse_position = LocalPoint::zero();

    let mut commands: Vec<CommandInfo> = Vec::new();
    let mut command_map = HashMap::new();
    cx.commands(&view, &mut commands);
    #[cfg(feature = "tao")]
    {
        window.set_menu(Some(menus::build_menubar(&commands, &mut command_map)));
    }

    #[cfg(feature = "winit")]
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
                while let Some(f) = GLOBAL_WORK_QUEUE.lock().unwrap().pop_front() {
                    f(&mut cx);
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

                #[cfg(feature = "tao")]
                {
                    let mut new_commands = vec![];
                    cx.commands(&view, &mut new_commands);

                    if new_commands != *commands {
                        print!("commands changed");
                        commands = new_commands;

                        command_map.clear();
                        window.set_menu(Some(menus::build_menubar(&commands, &mut command_map)));
                    }
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
                    &device,
                    &surface,
                    &config,
                    &queue,
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
                        cx.process(&view, &event, &mut vger)
                    }
                    ElementState::Released => {
                        cx.mouse_button = None;
                        let event = Event::TouchEnd {
                            id: 0,
                            position: mouse_position,
                        };
                        cx.process(&view, &event, &mut vger)
                    }
                    #[cfg(feature = "tao")]
                    _ => {}
                };
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
                let event = Event::TouchMove {
                    id: 0,
                    position: mouse_position,
                };
                cx.process(&view, &event, &mut vger)
            }

            #[cfg(feature = "tao")]
            WEvent::WindowEvent {
                event: WindowEvent::KeyboardInput { event, .. },
                ..
            } => {
                if event.state == ElementState::Pressed {
                    let key = match event.logical_key {
                        KeyPress::Character(c) => Some(Key::Character(c)),
                        KeyPress::Enter => Some(Key::Enter),
                        KeyPress::Tab => Some(Key::Tab),
                        KeyPress::Space => Some(Key::Space),
                        KeyPress::ArrowDown => Some(Key::ArrowDown),
                        KeyPress::ArrowLeft => Some(Key::ArrowLeft),
                        KeyPress::ArrowRight => Some(Key::ArrowRight),
                        KeyPress::ArrowUp => Some(Key::ArrowUp),
                        KeyPress::End => Some(Key::End),
                        KeyPress::Home => Some(Key::Home),
                        KeyPress::PageDown => Some(Key::PageDown),
                        KeyPress::PageUp => Some(Key::PageUp),
                        KeyPress::Backspace => Some(Key::Backspace),
                        KeyPress::Delete => Some(Key::Delete),
                        KeyPress::Escape => Some(Key::Escape),
                        KeyPress::F1 => Some(Key::F1),
                        KeyPress::F2 => Some(Key::F2),
                        KeyPress::F3 => Some(Key::F3),
                        KeyPress::F4 => Some(Key::F4),
                        KeyPress::F5 => Some(Key::F5),
                        KeyPress::F6 => Some(Key::F6),
                        KeyPress::F7 => Some(Key::F7),
                        KeyPress::F8 => Some(Key::F8),
                        KeyPress::F9 => Some(Key::F9),
                        KeyPress::F10 => Some(Key::F10),
                        KeyPress::F11 => Some(Key::F11),
                        KeyPress::F12 => Some(Key::F12),
                        _ => None,
                    };

                    if let Some(key) = key {
                        cx.process(&view, &Event::Key(key), &mut vger)
                    }
                }
            }

            #[cfg(feature = "winit")]
            WEvent::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } => {
                if input.state == ElementState::Pressed {
                    if let Some(code) = input.virtual_keycode {
                        let key = match code {
                            // VirtualKeyCode::Character(c) => Some(Key::Character(c)),
                            VirtualKeyCode::Key1 => Some(Key::Character("1")),
                            VirtualKeyCode::Key2 => Some(Key::Character("2")),
                            VirtualKeyCode::Key3 => Some(Key::Character("3")),
                            VirtualKeyCode::Key4 => Some(Key::Character("4")),
                            VirtualKeyCode::Key5 => Some(Key::Character("5")),
                            VirtualKeyCode::Key6 => Some(Key::Character("6")),
                            VirtualKeyCode::Key7 => Some(Key::Character("7")),
                            VirtualKeyCode::Key8 => Some(Key::Character("8")),
                            VirtualKeyCode::Key9 => Some(Key::Character("9")),
                            VirtualKeyCode::Key0 => Some(Key::Character("0")),
                            VirtualKeyCode::A => Some(Key::Character(if cx.key_mods.shift { "A"} else { "a" })),
                            VirtualKeyCode::B => Some(Key::Character(if cx.key_mods.shift { "B"} else { "b" })),
                            VirtualKeyCode::C => Some(Key::Character(if cx.key_mods.shift { "C"} else { "c" })),
                            VirtualKeyCode::D => Some(Key::Character(if cx.key_mods.shift { "D"} else { "d" })),
                            VirtualKeyCode::E => Some(Key::Character(if cx.key_mods.shift { "E"} else { "e" })),
                            VirtualKeyCode::F => Some(Key::Character(if cx.key_mods.shift { "F"} else { "f" })),
                            VirtualKeyCode::G => Some(Key::Character(if cx.key_mods.shift { "G"} else { "g" })),
                            VirtualKeyCode::H => Some(Key::Character(if cx.key_mods.shift { "H"} else { "h" })),
                            VirtualKeyCode::I => Some(Key::Character(if cx.key_mods.shift { "I"} else { "i" })),
                            VirtualKeyCode::J => Some(Key::Character(if cx.key_mods.shift { "J"} else { "j" })),
                            VirtualKeyCode::K => Some(Key::Character(if cx.key_mods.shift { "K"} else { "k" })),
                            VirtualKeyCode::L => Some(Key::Character(if cx.key_mods.shift { "L"} else { "l" })),
                            VirtualKeyCode::M => Some(Key::Character(if cx.key_mods.shift { "M"} else { "m" })),
                            VirtualKeyCode::N => Some(Key::Character(if cx.key_mods.shift { "N"} else { "n" })),
                            VirtualKeyCode::O => Some(Key::Character(if cx.key_mods.shift { "O"} else { "o" })),
                            VirtualKeyCode::P => Some(Key::Character(if cx.key_mods.shift { "P"} else { "p" })),
                            VirtualKeyCode::Q => Some(Key::Character(if cx.key_mods.shift { "Q"} else { "q" })),
                            VirtualKeyCode::R => Some(Key::Character(if cx.key_mods.shift { "R"} else { "r" })),
                            VirtualKeyCode::S => Some(Key::Character(if cx.key_mods.shift { "S"} else { "s" })),
                            VirtualKeyCode::T => Some(Key::Character(if cx.key_mods.shift { "T"} else { "t" })),
                            VirtualKeyCode::U => Some(Key::Character(if cx.key_mods.shift { "U"} else { "u" })),
                            VirtualKeyCode::V => Some(Key::Character(if cx.key_mods.shift { "V"} else { "v" })),
                            VirtualKeyCode::W => Some(Key::Character(if cx.key_mods.shift { "W"} else { "w" })),
                            VirtualKeyCode::X => Some(Key::Character(if cx.key_mods.shift { "X"} else { "x" })),
                            VirtualKeyCode::Y => Some(Key::Character(if cx.key_mods.shift { "Y"} else { "y" })),
                            VirtualKeyCode::Z => Some(Key::Character(if cx.key_mods.shift { "Z"} else { "z" })),
                            VirtualKeyCode::Period => Some(Key::Character(".")),
                            VirtualKeyCode::Comma => Some(Key::Character(",")),
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
                            cx.process(&view, &Event::Key(key), &mut vger)
                        }
                    }
                }
            }

            WEvent::WindowEvent {
                event: WindowEvent::ModifiersChanged(mods),
                ..
            } => {

                #[cfg(feature = "tao")]
                {
                    cx.key_mods = KeyboardModifiers {
                        shift: mods.shift_key(),
                        control: mods.control_key(),
                        alt: mods.alt_key(),
                        command: mods.super_key(),
                    };
                }

                #[cfg(feature = "winit")]
                {
                    cx.key_mods = KeyboardModifiers {
                        shift: mods.shift(),
                        control: mods.ctrl(),
                        alt: mods.alt(),
                        command: mods.logo(),
                    };
                }
            }

            #[cfg(feature = "tao")]
            WEvent::MenuEvent { menu_id, .. } => {
                //println!("menu event");

                if let Some(command) = command_map.get(&menu_id) {
                    //println!("found command {:?}", command);
                    let event = Event::Command(command.clone());
                    cx.process(&view, &event, &mut vger)
                }
            }
            _ => (),
        }
    });
}
