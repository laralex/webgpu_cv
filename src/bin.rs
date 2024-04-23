mod env;
use env::log_init;

use winit::{
   event::*, event_loop::EventLoop, keyboard::*, window::WindowBuilder
};

fn main() {
   log_init();
   let event_loop = EventLoop::new()
      .expect("Winit failed to initialize");
   let window = WindowBuilder::new().build(&event_loop).unwrap();

   event_loop.run(move |event, control_flow| match event {
       Event::WindowEvent {
           ref event,
           window_id,
       } if window_id == window.id() => match event {
           WindowEvent::CloseRequested
           | WindowEvent::KeyboardInput {
               event:
                   KeyEvent {
                       state: ElementState::Pressed,
                       logical_key: Key::Named(NamedKey::Escape),
                       ..
                   },
               ..
           } => control_flow.exit(),
           _ => {}
       },
       _ => {}
   })
   .expect("Winit failed to start event loop");
}