use std::{cell::RefCell, pin::Pin, rc::Rc};

use super::{Dispose, ExternalState, GraphicsLevel, GraphicsSwitchingFuture, IDemo, Progress, SimpleFuture};
use web_sys::{WebGl2RenderingContext as GL};

pub struct StubDemo;
pub struct GraphicsSwitchingProcess {
   demo: StubDemo,
}

impl Dispose for GraphicsSwitchingProcess {
   fn dispose(&mut self) { }
}

impl Drop for GraphicsSwitchingProcess {
   fn drop(&mut self) {
      self.dispose();
   }
}

impl Progress for GraphicsSwitchingProcess {
    fn progress(&self) -> f32 { 1.0 }
}

impl SimpleFuture for GraphicsSwitchingProcess {
   type Output = Box<dyn IDemo>;
   type Context = ();

   fn simple_poll(mut self: std::pin::Pin<&mut Self>, cx: &mut Self::Context) -> std::task::Poll<Self::Output> {
      std::task::Poll::Ready(Box::new(self.demo))
   }
}

impl GraphicsSwitchingFuture for GraphicsSwitchingProcess {}

impl Drop for StubDemo {
    fn drop(&mut self) { }
}

impl IDemo for StubDemo {
   fn tick(&mut self, input: &ExternalState) { }

   fn render(&mut self, gl: &GL, delta_sec: f32) { }

   fn start_switching_graphics_level(&mut self, gl: Rc<GL>, level: GraphicsLevel) -> Box<dyn GraphicsSwitchingFuture> {
      web_sys::console::log_1(&"Rust start_switching_graphics_level: StubDemo".into());
      Box::new(GraphicsSwitchingProcess{demo: StubDemo{}})
   }

   // fn start_switching_graphics_level_static(this: std::pin::Pin<Box<&mut Self>>, gl: Rc<GL>, level: GraphicsLevel) -> Pin<Box<dyn GraphicsSwitchingFuture>> {
   //    web_sys::console::log_1(&"Rust start_switching_graphics_level_static: StubDemo".into());
   //    Box::pin(GraphicsSwitchingProcess{})
   // }

   fn drop_demo(&mut self, gl: &GL) {
      web_sys::console::log_1(&"Rust demo drop: StubDemo".into());
   }
}