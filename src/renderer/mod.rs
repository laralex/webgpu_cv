pub mod triangle;

use std::{cell::Cell, rc::Rc};
use web_sys::WebGl2RenderingContext as GL;

#[derive(Clone, Copy)]
pub struct MouseState {
   pub left: f32,
   pub middle: f32,
   pub right: f32,
   pub viewport_position: (i32, i32),
   pub unit_position: (f32, f32),
}

pub struct ExternalState {
   pub mouse: Rc<Cell<MouseState>>,
   pub screen_size: (u32, u32),
   pub delta_sec: f32,
   pub frame_idx: usize,
}

pub trait IDemo {
   fn tick(&mut self, state: &ExternalState);
   fn render(&mut self, gl: &mut GL, delta_sec: f32);
}