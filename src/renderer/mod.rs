pub mod triangle;

use web_sys::WebGl2RenderingContext as GL;

pub trait IDemo {
   fn tick(&mut self, delta_sec: f32);
   fn render(&mut self, gl: &mut GL, delta_sec: f32);
}