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
use vger::*;

use winit::{
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

async fn setup(window: &winit::window::Window) -> (wgpu::Device, wgpu::Queue) {
    let backend = wgpu::Backends::all();
    let instance = wgpu::Instance::new(backend);

    let (size, surface) = unsafe {
        let size = window.inner_size();
        let surface = instance.create_surface(window);
        (size, surface)
    };

    let adapter =
        wgpu::util::initialize_adapter_from_env_or_default(&instance, backend, Some(&surface))
            .await
            .expect("No suitable GPU adapters found on the system!");

    let adapter_info = adapter.get_info();
    println!("Using {} ({:?})", adapter_info.name, adapter_info.backend);

    let trace_dir = std::env::var("WGPU_TRACE");
    adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::default(),
                limits: wgpu::Limits::default(),
            },
            trace_dir.ok().as_ref().map(std::path::Path::new),
        )
        .await
        .expect("Unable to find a suitable GPU adapter!")
}

pub fn rui(view: impl View + 'static) {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("rui")
        .build(&event_loop)
        .unwrap();

    let (device, queue) = block_on(setup(&window));

    let vger = VGER::new(&device);
    let mut cx = Context::new();
    let mut window_size = [0.0, 0.0];

    event_loop.run(move |event, _, control_flow| {
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
                window_size[0] = size.width as f32;
                window_size[1] = size.height as f32;
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

                println!("RedrawRequested");

                // vger.begin(window_size[0], window_size[1], 1.0);
                
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
        v.draw(ViewID::default(), &mut cx);
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
        s.draw(ViewID::default(), &mut cx);
    }

    fn counter(start: usize) -> impl View {
        state(start, |count: State<usize>| {
            let count2 = count.clone();
            let value_string = format!("value: {:?}", *count.get());
            stack! {
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
        v.draw(ViewID::default(), &mut cx);
        println!("ok, now pressing increment button");
        v.process(
            &Event::PressButton(String::from("increment")),
            ViewID::default(),
            &mut cx,
        );
        println!("\"drawing\" the UI again");
        v.draw(ViewID::default(), &mut cx);
    }

    fn counter3<B>(count: B) -> impl View
    where
        B: Binding<usize> + Clone + 'static,
    {
        let count2 = count.clone();
        let mut stack = Stack::new();
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
