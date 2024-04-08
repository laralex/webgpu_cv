type GL = web_sys::WebGl2RenderingContext;
use super::{Dispose, ExternalState, GraphicsLevel, IDemo};

pub struct StubDemo;

impl Drop for StubDemo {
   fn drop(&mut self) { self.dispose(); }
}

impl Dispose for StubDemo {
   fn dispose(&mut self) { }
}

impl IDemo for StubDemo {
   fn tick(&mut self, _input: &ExternalState) { }

   fn render(&mut self, _gl: &GL, _delta_sec: f32) { }

   fn start_switching_graphics_level(&mut self, _gl: &GL, _level: GraphicsLevel) {
      web_sys::console::log_1(&"Rust start_switching_graphics_level: StubDemo".into());
   }

   fn poll_switching_graphics_level(&mut self, _gl: &GL) -> std::task::Poll<()> {
      std::task::Poll::Ready(())
   }

   fn progress_switching_graphics_level(&self) -> f32 {
      0.0
   }

   fn drop_demo(&mut self, _gl: &GL) {
      web_sys::console::log_1(&"Rust demo drop: StubDemo".into());
   }
}