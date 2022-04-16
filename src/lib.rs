// #![feature(type_alias_impl_trait)]

mod view;
pub use view::*;

mod viewid;
pub use viewid::*;

mod event;
pub use event::*;

mod binding;
pub use binding::*;

mod context;
pub use context::*;

mod views;
pub use views::*;

mod paint;
pub use paint::*;

mod modifiers;
pub use modifiers::*;

mod body;
pub use body::*;

mod colors;
pub use colors::*;

mod align;
pub use align::*;

mod region;
pub use region::*;

use futures::executor::block_on;
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;

use vger::color::*;
use vger::*;

use tao::{
    accelerator::Accelerator,
    dpi::PhysicalSize,
    event::{ElementState, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::ModifiersState,
    menu::{MenuBar as Menu, MenuItem, MenuItemAttributes},
    window::{Window, WindowBuilder},
};

#[macro_use]
extern crate lazy_static;

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

        let app_name = match std::env::current_exe() {
            Ok(exe_path) => exe_path.file_name().unwrap().to_str().unwrap().to_string(),
            Err(_) => "rui".to_string(),
        };
        app_menu.add_native_item(MenuItem::About(app_name));
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

/// Call this function to run your UI.
pub fn rui(view: impl View) {
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

    *GLOBAL_EVENT_LOOP_PROXY.lock().unwrap() = Some(event_loop.create_proxy());

    let mut vger = VGER::new(&device, wgpu::TextureFormat::Bgra8UnormSrgb);
    let mut cx = Context::new(Some(window));
    let mut mouse_position = LocalPoint::zero();

    let mut commands = Vec::new();
    view.commands(cx.root_id, &mut cx, &mut commands);
    let mut command_map = HashMap::new();
    cx.window
        .as_ref()
        .unwrap()
        .set_menu(Some(build_menubar(&commands, &mut command_map)));

    let mut access_nodes = vec![];

    let mut dirty_region = Region::<WorldSpace>::EMPTY;

    event_loop.run(move |event, _, control_flow| {
        // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
        // dispatched any events. This is ideal for games and similar applications.
        // *control_flow = ControlFlow::Poll;

        // ControlFlow::Wait pauses the event loop if no events are available to process.
        // This is ideal for non-game applications that only update in response to user
        // input, and uses significantly less power/CPU time than ControlFlow::Poll.
        *control_flow = ControlFlow::Wait;

        match event {
            tao::event::Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                println!("The close button was pressed; stopping");
                *control_flow = ControlFlow::Exit
            }
            tao::event::Event::WindowEvent {
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
                cx.window.as_ref().unwrap().request_redraw();
            }
            tao::event::Event::UserEvent(_) => {
                // println!("received user event");

                // Process the work queue.
                while let Some(f) = GLOBAL_WORK_QUEUE.lock().unwrap().pop_front() {
                    f(&mut cx);
                }
            }
            tao::event::Event::MainEventsCleared => {
                // Application update code.

                // Queue a RedrawRequested event.
                //
                // You only need to call this if you've determined that you need to redraw, in
                // applications which do not always need to. Applications that redraw continuously
                // can just render here instead.
                if cx.dirty {
                    // Have the commands changed?
                    let mut new_commands = Vec::new();
                    view.commands(cx.root_id, &mut cx, &mut new_commands);

                    if new_commands != commands {
                        print!("commands changed");
                        commands = new_commands;

                        command_map.clear();
                        cx.window
                            .as_ref()
                            .unwrap()
                            .set_menu(Some(build_menubar(&commands, &mut command_map)));
                    }

                    // Clean up state.
                    let mut keep = vec![];
                    view.gc(cx.root_id, &mut cx, &mut keep);
                    let keep_set = HashSet::<ViewId>::from_iter(keep);
                    cx.state_map.retain(|k, _| keep_set.contains(k));

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

                    // Get dirty rectangles.
                    view.dirty(cx.root_id, LocalToWorld::identity(), &mut cx, &mut dirty_region);

                    cx.window.as_ref().unwrap().request_redraw();

                    cx.clear_dirty();
                }
            }
            tao::event::Event::RedrawRequested(_) => {
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

                let window_size = cx.window.as_ref().unwrap().inner_size();
                let scale = cx.window.as_ref().unwrap().scale_factor() as f32;
                // println!("window_size: {:?}", window_size);
                let width = window_size.width as f32 / scale;
                let height = window_size.height as f32 / scale;

                vger.begin(width, height, scale);

                // Disable dirtying the state during layout and rendering
                // to avoid constantly re-rendering if some state is saved.
                cx.enable_dirty = false;
                view.layout(cx.root_id, [width, height].into(), &mut cx, &mut vger);
                view.draw(cx.root_id, &mut cx, &mut vger);
                cx.enable_dirty = true;

                let paint = vger.color_paint(RED_HIGHLIGHT);
                let xf = WorldToLocal::identity();
                for rect in dirty_region.rects() {
                    vger.stroke_rect(xf.transform_point(rect.min()), xf.transform_point(rect.max()), 0.0, 1.0, paint);
                }

                dirty_region.clear();

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
            tao::event::Event::WindowEvent {
                event: WindowEvent::MouseInput { state, button, .. },
                ..
            } => {
                match state {
                    ElementState::Pressed => {
                        cx.mouse_button = Some(button);
                        let event = Event {
                            kind: EventKind::TouchBegin { id: 0 },
                            position: mouse_position,
                        };
                        view.process(&event, cx.root_id, &mut cx, &mut vger)
                    }
                    ElementState::Released => {
                        cx.mouse_button = None;
                        let event = Event {
                            kind: EventKind::TouchEnd { id: 0 },
                            position: mouse_position,
                        };
                        view.process(&event, cx.root_id, &mut cx, &mut vger)
                    }
                    _ => {}
                };
            }
            tao::event::Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                let scale = cx.window.as_ref().unwrap().scale_factor() as f32;
                mouse_position = [
                    position.x as f32 / scale,
                    (config.height as f32 - position.y as f32) / scale,
                ]
                .into();
                let event = Event {
                    kind: EventKind::TouchMove { id: 0 },
                    position: mouse_position,
                };
                view.process(&event, cx.root_id, &mut cx, &mut vger)
            }
            tao::event::Event::WindowEvent {
                event: WindowEvent::KeyboardInput { event, .. },
                ..
            } => {
                if event.state == ElementState::Pressed {
                    let event = Event {
                        kind: EventKind::Key(event.logical_key),
                        position: mouse_position,
                    };
                    view.process(&event, cx.root_id, &mut cx, &mut vger)
                }
            }
            tao::event::Event::WindowEvent {
                event: WindowEvent::ModifiersChanged(mods),
                ..
            } => {
                // println!("modifiers changed: {:?}", mods);
                cx.key_mods = mods;
            }
            tao::event::Event::MenuEvent { menu_id, .. } => {
                //println!("menu event");

                if let Some(command) = command_map.get(&menu_id) {
                    //println!("found command {:?}", command);
                    let event = Event {
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
        let _ = button(text("click me"), |_cx| {
            println!("clicked!");
        });
    }
}
