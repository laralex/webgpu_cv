use super::{ExternalState, IDemo, GraphicsLevel};
use web_sys::{WebGl2RenderingContext as GL};

pub struct StubDemo;

impl IDemo for StubDemo {
   fn tick(&mut self, input: &ExternalState) {

   }

   fn render(&mut self, gl: &GL, delta_sec: f32) {

   }

   fn set_graphics_level(&mut self, level: GraphicsLevel) {

   }
}