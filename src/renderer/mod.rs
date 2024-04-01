pub mod triangle;

use std::{cell::Cell, rc::Rc};
use web_sys::WebGl2RenderingContext as GL;

#[derive(Default, Clone, Copy)]
pub enum GraphicsLevel {
   Minimal = 0x00,
   Low = 0x10,
   #[default] Medium = 0x20,
   High = 0x30,
   Ultra = 0xFF,
}

impl From<u32> for GraphicsLevel {
    fn from(level_code: u32) -> Self {
      match level_code {
         0x00 => GraphicsLevel::Minimal,
         0x10 => GraphicsLevel::Low,
         0x20 => GraphicsLevel::Medium,
         0x30 => GraphicsLevel::High,
         0xFF => GraphicsLevel::Ultra,
         _ => Default::default(),
      }
    }
}


#[derive(Clone, Copy)]
pub struct MouseState {
   pub left: f32,
   pub middle: f32,
   pub right: f32,
   pub wheel: f32,
   pub viewport_position: (i32, i32),
}

pub struct ExternalState {
   pub mouse: Rc<Cell<MouseState>>,
   pub screen_size: (u32, u32),
   pub time_sec: f32,
   pub time_delta_sec: f32,
   pub time_delta_limit_ms: i32,
   pub frame_idx: usize,
   pub frame_rate: f32,
   pub date: chrono::NaiveDate,
   #[allow(unused)] pub sound_sample_rate: f32,
   pub graphics_level: GraphicsLevel,
}

impl ExternalState {
   pub fn mouse_unit_position(&self) -> (f32, f32) {
      let px_pos = self.mouse.get().viewport_position;
      return (
         px_pos.0 as f32 / self.screen_size.0 as f32,
         px_pos.1 as f32 / self.screen_size.1 as f32,
      )
   }
}

impl Default for ExternalState {
    fn default() -> Self {
        Self {
         mouse: Rc::new(Cell::new(MouseState {
            left: Default::default(),
            middle: Default::default(),
            right: Default::default(),
            wheel: Default::default(), /* TODO: not populated */
            viewport_position: Default::default(),
         })),
         screen_size: (1, 1),
         time_delta_sec: Default::default(),
         time_delta_limit_ms: Default::default(),
         time_sec: Default::default(),
         frame_idx: Default::default(),
         frame_rate: 1.0,
         date: Default::default(),
         sound_sample_rate: Default::default(),
         graphics_level: Default::default(),
       }
    }
}

impl ExternalState {
   pub fn begin_frame(&mut self, elapsed_sec: f32) {
      self.time_delta_sec = elapsed_sec.max(1e-6);
      self.time_sec += self.time_delta_sec;
      self.frame_rate = 1.0 / self.time_delta_sec;
   }

   pub fn end_frame(&mut self) {
      let mut current_mouse_state = self.mouse.get();
      if current_mouse_state.left < 0.0 {
         current_mouse_state.left = 0.0;
      }
      if current_mouse_state.middle < 0.0 {
         current_mouse_state.middle = 0.0;
      }
      if current_mouse_state.right < 0.0 {
         current_mouse_state.right = 0.0;
      }
      self.mouse.set(current_mouse_state);
      self.frame_idx += 1;
   }
}

pub trait IDemo {
   fn tick(&mut self, state: &ExternalState);
   fn render(&mut self, gl: &mut GL, delta_sec: f32);
   fn set_graphics_level(&mut self, level: GraphicsLevel);
}