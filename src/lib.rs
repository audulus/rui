// #![feature(type_alias_impl_trait)]

mod view;
pub use view::*;

mod binding;
pub use binding::*;

mod state;
pub use state::*;

mod text;
pub use text::*;

mod button;
pub use button::*;

mod stack;
pub use stack::*;

mod context;
pub use context::*;

mod padding;
pub use padding::*;

mod shapes;
pub use shapes::*;

mod paint;
pub use paint::*;

mod gestures;
pub use gestures::*;

mod background;
pub use background::*;

mod modifiers;
pub use modifiers::*;

mod geom;
pub use geom::*;

mod offset;
pub use offset::*;

mod canvas;
pub use canvas::*;

mod slider;
pub use slider::*;

mod list;
pub use list::*;

mod size;
pub use size::*;

mod body;
pub use body::*;

mod knob;
pub use knob::*;

mod command;
pub use command::*;

mod colors;
pub use colors::*;

mod key;
pub use key::*;

mod text_editor;
pub use text_editor::*;

mod focus;
pub use focus::*;

mod cond;
pub use cond::*;

mod toggle;
pub use toggle::*;

mod align;
pub use align::*;

mod role;
pub use role::*;

mod window;
pub use window::*;

use futures::executor::block_on;
use std::collections::HashMap;
use vger::color::*;
use vger::*;

use tao::{
    accelerator::Accelerator,
    dpi::PhysicalSize,
    event,
    event::{ElementState, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::ModifiersState,
    menu::{MenuBar as Menu, MenuItem, MenuItemAttributes},
    window::{Window, WindowBuilder},
};

use std::env;

// See https://rust-lang.github.io/api-guidelines/future-proofing.html
pub(crate) mod private {
    pub trait Sealed {}
}

pub type KeyCode = tao::keyboard::KeyCode;
pub type KeyPress = tao::keyboard::Key<'static>;

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

#[derive(Clone, Eq, PartialEq)]
pub struct CommandInfo {
    path: String,
    key: Option<KeyCode>,
}

struct MenuItem2 {
    name: String,
    submenu: Vec<usize>,
    command: CommandInfo,
}

fn make_menu_rec(
    items: &Vec<MenuItem2>,
    i: usize,
    command_map: &mut HashMap<tao::menu::MenuId, String>,
) -> Menu {
    let mut menu = Menu::new();

    if i == 0 {
        let mut app_menu = Menu::new();

        let app_name = match env::current_exe() {
            Ok(exe_path) => exe_path.file_name().unwrap().to_str().unwrap().to_string(),
            Err(_) => "rui".to_string(),
        };
        app_menu.add_native_item(MenuItem::About(app_name));
        app_menu.add_native_item(MenuItem::Quit);
        menu.add_submenu("rui", true, app_menu);
    }

    for j in &items[i].submenu {
        let item = &items[*j];
        if item.submenu.len() > 0 {
            menu.add_submenu(
                item.name.as_str(),
                true,
                make_menu_rec(items, *j, command_map),
            );
        } else {
            let mut attrs = MenuItemAttributes::new(item.name.as_str());
            if let Some(key) = item.command.key {
                let accel = Accelerator::new(ModifiersState::SUPER, key);
                attrs = attrs.with_accelerators(&accel);
            }
            let id = menu.add_item(attrs).id();
            command_map.insert(id, item.command.path.clone());
        }
    }

    menu
}

fn build_menubar(
    commands: &Vec<CommandInfo>,
    command_map: &mut HashMap<tao::menu::MenuId, String>,
) -> Menu {
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
        for name in command.path.split(":") {
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

/// Call this function to run your UI.
pub fn rui(view: impl View + 'static) {
    let event_loop = EventLoop::new();

    let builder = WindowBuilder::new().with_title("rui");
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

    let mut vger = VGER::new(&device, wgpu::TextureFormat::Bgra8UnormSrgb);
    let mut cx = Context::new(Some(event_loop.create_proxy()), window);
    let mut mouse_position = LocalPoint::zero();

    let mut commands = Vec::new();
    view.commands(cx.root_id, &mut cx, &mut commands);
    let mut command_map = HashMap::new();
    cx.window
        .set_menu(Some(build_menubar(&commands, &mut command_map)));

    let mut modifiers = ModifiersState::default();

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
            event::Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                println!("The close button was pressed; stopping");
                *control_flow = ControlFlow::Exit
            }
            event::Event::WindowEvent {
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
                cx.window.request_redraw();
            }
            event::Event::UserEvent(_) => {
                // println!("received user event");
            }
            event::Event::MainEventsCleared => {
                // Application update code.

                // Queue a RedrawRequested event.
                //
                // You only need to call this if you've determined that you need to redraw, in
                // applications which do not always need to. Applications that redraw continuously
                // can just render here instead.
                if cx.dirty.lock().unwrap().dirty {
                    // Have the commands changed?
                    let mut new_commands = Vec::new();
                    view.commands(cx.root_id, &mut cx, &mut new_commands);

                    if new_commands != commands {
                        print!("commands changed");
                        commands = new_commands;

                        command_map.clear();
                        cx.window
                            .set_menu(Some(build_menubar(&commands, &mut command_map)));
                    }

                    // Clean up state.
                    let mut new_map = StateMap::new();
                    view.gc(cx.root_id, &mut cx, &mut new_map);
                    // println!("collected {} states", cx.state_map.len() - new_map.len());
                    cx.state_map = new_map;

                    // Get a new accesskit tree.
                    let mut nodes = vec![];
                    view.access(cx.root_id, &mut cx, &mut nodes);

                    if nodes != access_nodes {
                        println!("access nodes:");
                        for node in &nodes {
                            println!(
                                "  id: {:?}, role: {:?}, children: {:?}",
                                node.id, node.role, node.children
                            );
                        }
                        access_nodes = nodes;
                    } else {
                        // println!("access nodes unchanged");
                    }

                    cx.window.request_redraw();

                    cx.dirty.lock().unwrap().dirty = false;
                }
            }
            event::Event::RedrawRequested(_) => {
                // Redraw the application.
                //
                // It's preferable for applications that do not render continuously to render in
                // this event rather than in MainEventsCleared, since rendering in here allows
                // the program to gracefully handle redraws requested by the OS.

                // println!("RedrawRequested");

                let frame = match surface.get_current_texture() {
                    Ok(frame) => frame,
                    Err(_) => {
                        surface.configure(&device, &config);
                        surface
                            .get_current_texture()
                            .expect("Failed to acquire next surface texture!")
                    }
                };

                let window_size = cx.window.inner_size();
                let scale = cx.window.scale_factor() as f32;
                // println!("window_size: {:?}", window_size);
                let width = window_size.width as f32 / scale;
                let height = window_size.height as f32 / scale;

                vger.begin(width, height, scale);

                view.layout(cx.root_id, [width, height].into(), &mut cx, &mut vger);
                view.draw(cx.root_id, &mut cx, &mut vger);

                // After rendering, clearly the dirty flag in case rendering
                // updated some state (for example, saving off geometric information
                // when drawing with Canvas).
                cx.dirty.lock().unwrap().dirty = false;

                let texture_view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let desc = wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[wgpu::RenderPassColorAttachment {
                        view: &texture_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: true,
                        },
                    }],
                    depth_stencil_attachment: None,
                };

                vger.encode(&device, &desc, &queue);

                frame.present();
            }
            event::Event::WindowEvent {
                event: WindowEvent::MouseInput { state, .. },
                ..
            } => {
                match state {
                    ElementState::Pressed => {
                        let event = view::Event {
                            kind: EventKind::TouchBegin { id: 0 },
                            position: mouse_position,
                        };
                        view.process(&event, cx.root_id, &mut cx, &mut vger)
                    }
                    ElementState::Released => {
                        let event = view::Event {
                            kind: EventKind::TouchEnd { id: 0 },
                            position: mouse_position,
                        };
                        view.process(&event, cx.root_id, &mut cx, &mut vger)
                    }
                    _ => {}
                };
            }
            event::Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                let scale = cx.window.scale_factor() as f32;
                mouse_position = [
                    position.x as f32 / scale,
                    (config.height as f32 - position.y as f32) / scale,
                ]
                .into();
                let event = view::Event {
                    kind: EventKind::TouchMove { id: 0 },
                    position: mouse_position,
                };
                view.process(&event, cx.root_id, &mut cx, &mut vger)
            }
            event::Event::WindowEvent {
                event: WindowEvent::KeyboardInput { event, .. },
                ..
            } => {
                if event.state == ElementState::Pressed {
                    let event = view::Event {
                        kind: EventKind::Key(event.logical_key, modifiers),
                        position: mouse_position,
                    };
                    view.process(&event, cx.root_id, &mut cx, &mut vger)
                }
            }
            event::Event::WindowEvent {
                event: WindowEvent::ModifiersChanged(mods),
                ..
            } => {
                modifiers = mods;
                // println!("modifiers changed: {:?}", modifiers);
            }
            event::Event::MenuEvent { menu_id, .. } => {
                //println!("menu event");

                if let Some(command) = command_map.get(&menu_id) {
                    //println!("found command {:?}", command);
                    let event = view::Event {
                        kind: EventKind::Command(command.clone()),
                        position: mouse_position,
                    };
                    view.process(&event, cx.root_id, &mut cx, &mut vger)
                }
            }
            _ => (),
        }
    });
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_button() {
        let _ = button(text("click me"), || {
            println!("clicked!");
        });
    }

    // #[test]
    // fn test_state2() {
    //     let mut cx = Context::new(None);
    //     let v = counter(42);
    //     v.print(ViewID::default(), &mut cx);
    // }

    fn counter(start: usize) -> impl View {
        state(
            move || start,
            |count: State<usize>| {
                let count2 = count.clone();
                let value_string = format!("value: {:?}", count.get());
                vstack((
                    text(value_string.as_str()),
                    button(text("increment"), move || {
                        let value = count.get();
                        count.set(value + 1);
                    }),
                    button(text("decrement"), move || {
                        let value = count2.get();
                        count2.set(value - 1);
                    }),
                ))
            },
        )
    }

    fn counter3<B>(count: B) -> impl View
    where
        B: Binding<usize> + Clone + 'static,
    {
        let count2 = count.clone();
        vstack((
            button(text("increment"), move || {
                let value = count.get();
                count.set(value + 1);
            }),
            button(text("decrement"), move || {
                let value = count2.get();
                count2.set(value - 1);
            }),
        ))
    }

    #[test]
    fn test_binding() {
        let _ = state(|| 42, |count: State<usize>| counter3(count));
    }
}
