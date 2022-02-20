// #![feature(type_alias_impl_trait)]

mod view;
pub use view::*;

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

use futures::executor::block_on;
use vger::color::*;
use vger::*;

use winit::{
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

struct Setup {
    window: winit::window::Window,
    event_loop: EventLoop<()>,
    instance: wgpu::Instance,
    size: winit::dpi::PhysicalSize<u32>,
    surface: wgpu::Surface,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

async fn setup(title: &str) -> Setup {
    let event_loop = EventLoop::new();
    let mut builder = winit::window::WindowBuilder::new();
    builder = builder.with_title(title);
    let window = builder.build(&event_loop).unwrap();

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
        window,
        event_loop,
        instance,
        size,
        surface,
        adapter,
        device,
        queue,
    }
}

pub fn rui(view: impl View + 'static) {
    let setup = block_on(setup("rui"));
    let window = setup.window;
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
    let mut cx = Context::new();

    setup.event_loop.run(move |event, _, control_flow| {
        // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
        // dispatched any events. This is ideal for games and similar applications.
        // *control_flow = ControlFlow::Poll;

        // ControlFlow::Wait pauses the event loop if no events are available to process.
        // This is ideal for non-game applications that only update in response to user
        // input, and uses significantly less power/CPU time than ControlFlow::Poll.
        *control_flow = ControlFlow::Wait;

        match event {
            winit::event::Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                println!("The close button was pressed; stopping");
                *control_flow = ControlFlow::Exit
            }
            winit::event::Event::WindowEvent {
                event:
                    WindowEvent::Resized(size)
                    | WindowEvent::ScaleFactorChanged {
                        new_inner_size: &mut size,
                        ..
                    },
                ..
            } => {
                println!("Resizing to {:?}", size);
                config.width = size.width.max(1);
                config.height = size.height.max(1);
                surface.configure(&device, &config);
            }
            winit::event::Event::MainEventsCleared => {
                // Application update code.

                // Queue a RedrawRequested event.
                //
                // You only need to call this if you've determined that you need to redraw, in
                // applications which do not always need to. Applications that redraw continuously
                // can just render here instead.
                window.request_redraw();
            }
            winit::event::Event::RedrawRequested(_) => {
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

                vger.begin(config.width as f32, config.height as f32, 1.0);

                let paint = vger.color_paint(Color {
                    r: 0.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                });

                view.layout(
                    ViewID::default(),
                    [config.width as f32, config.height as f32].into(),
                    &mut cx,
                    &mut vger,
                );
                view.draw(ViewID::default(), &mut cx, &mut vger);

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
            _ => (),
        }
    });
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_state_clone() {
        let s = State::new(0);
        let s2 = s.clone();
        s.set(42);
        assert_eq!(*s2.get(), 42);
    }

    #[test]
    fn test_button() {
        let _ = button("click me", || {
            println!("clicked!");
        });
    }

    #[test]
    fn test_state() {
        let _ = state(0, |_s: State<usize>| EmptyView {});
    }

    fn counter0(start: usize) -> impl View {
        state(start, |count: State<usize>| {
            button(format!("{:?}", *count.get()).as_str(), move || {
                *count.get() += 1;
            })
        })
    }

    #[test]
    fn test_state2() {
        let mut cx = Context::new();
        let v = counter(42);
        v.print(ViewID::default(), &mut cx);
    }

    #[test]
    fn test_stack() {
        let mut cx = Context::new();
        let s = stack2(
            EmptyView {},
            button("click me!", || {
                println!("clicked");
            }),
        );
        s.print(ViewID::default(), &mut cx);
    }

    fn counter(start: usize) -> impl View {
        state(start, |count: State<usize>| {
            let count2 = count.clone();
            let value_string = format!("value: {:?}", *count.get());
            vstack! {
                text(value_string.as_str());
                button("increment", move || {
                    *count.get() += 1;
                });
                button("decrement", move || {
                    *count2.get() -= 1;
                })
            }
        })
    }

    #[test]
    fn test_state3() {
        let mut cx = Context::new();
        let v = counter(42);
        println!("\"drawing\" the UI");
        v.print(ViewID::default(), &mut cx);
        println!("ok, now pressing increment button");
        v.process(
            &Event {
                kind: EventKind::PressButton(String::from("increment")),
                position: [0.0, 0.0].into(),
            },
            ViewID::default(),
            &mut cx,
        );
        println!("\"drawing\" the UI again");
        v.print(ViewID::default(), &mut cx);
    }

    fn counter3<B>(count: B) -> impl View
    where
        B: Binding<usize> + Clone + 'static,
    {
        let count2 = count.clone();
        let mut stack = Stack::new(StackOrientation::Vertical);
        stack.push(button("increment", move || {
            *count.get() += 1;
        }));
        stack.push(button("decrement", move || {
            *count2.get() -= 1;
        }));
        stack
    }

    #[test]
    fn test_binding() {
        let _ = state(42, |count: State<usize>| counter3(count));
    }

    fn ok_button<F: Fn() + 'static>(f: F) -> impl View {
        button("ok", f)
    }
}
