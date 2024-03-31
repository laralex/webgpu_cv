pub mod triangle;

use std::{cell::Cell, rc::Rc};
use web_sys::WebGl2RenderingContext as GL;

pub struct MouseState {
   pub is_down: Rc<Cell<bool>>,
   pub screen_position: Rc<Cell<(i32, i32)>>,
}

pub trait IDemo {
   fn tick(&mut self, delta_sec: f32, mouse_state: &MouseState);
   fn render(&mut self, gl: &mut GL, delta_sec: f32);
}