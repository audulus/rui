use winit::{
   event::{Event, WindowEvent},
   event_loop::{ControlFlow, EventLoop},
   window::WindowBuilder,
};

fn main() {
   let event_loop = EventLoop::new();
   let window = WindowBuilder::new().build(&event_loop).unwrap();

   event_loop.run(move |event, _, control_flow| {
      // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
      // dispatched any events. This is ideal for games and similar applications.
      // *control_flow = ControlFlow::Poll;

      // ControlFlow::Wait pauses the event loop if no events are available to process.
      // This is ideal for non-game applications that only update in response to user
      // input, and uses significantly less power/CPU time than ControlFlow::Poll.
      *control_flow = ControlFlow::Wait;

      match event {
         Event::WindowEvent {
               event: WindowEvent::CloseRequested,
               ..
         } => {
               println!("The close button was pressed; stopping");
               *control_flow = ControlFlow::Exit
         },
         Event::MainEventsCleared => {
               // Application update code.

               // Queue a RedrawRequested event.
               //
               // You only need to call this if you've determined that you need to redraw, in
               // applications which do not always need to. Applications that redraw continuously
               // can just render here instead.
               window.request_redraw();
         },
         Event::RedrawRequested(_) => {
               // Redraw the application.
               //
               // It's preferable for applications that do not render continuously to render in
               // this event rather than in MainEventsCleared, since rendering in here allows
               // the program to gracefully handle redraws requested by the OS.
         },
         _ => ()
      }
   });
}
