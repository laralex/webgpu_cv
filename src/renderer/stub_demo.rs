type GL = web_sys::WebGl2RenderingContext;
use std::task::Poll;

use super::{DemoLoadingFuture, Dispose, ExternalState, GraphicsLevel, IDemo, Progress, SimpleFuture};

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

struct DemoLoadingProcess { }

impl Dispose for DemoLoadingProcess {
   fn dispose(&mut self) { }
}

impl Drop for DemoLoadingProcess {
   fn drop(&mut self) { self.dispose(); }
}

impl Progress for DemoLoadingProcess {
    fn progress(&self) -> f32 { 1.0 }
}

impl SimpleFuture for DemoLoadingProcess {
   type Output = Box<dyn IDemo>;
   type Context = ();

   fn simple_poll(mut self: std::pin::Pin<&mut Self>, _cx: &mut Self::Context) -> Poll<Self::Output> {
      Poll::Ready(Box::new(StubDemo{}))
   }
}

impl DemoLoadingFuture for DemoLoadingProcess {}

impl StubDemo {
   pub fn start_loading() -> Box<dyn DemoLoadingFuture> {
      Box::new(DemoLoadingProcess{})
   }
}