pub mod stub_demo;
pub use stub_demo::Demo as StubDemo;
pub mod webgpu;
pub use webgpu::Webgpu;
pub mod webgpu_utils;
mod triangle;
mod fractal;

use crate::GraphicsLevel;
use wgpu::SurfaceTexture;

use std::{cell::Cell, pin::Pin, rc::Rc};

#[cfg(feature = "web")]
pub mod wasm {

use super::*;
use crate::wasm::DemoId;

pub fn start_loading_demo<'a>(id: DemoId, webgpu: Rc<Webgpu>, color_target_format: wgpu::TextureFormat, graphics_level: GraphicsLevel) -> Box<dyn DemoLoadingFuture> {
   match id {
      DemoId::Triangle =>
         triangle::Demo::start_loading(webgpu, color_target_format, graphics_level),
      DemoId::Fractal =>
         fractal::Demo::start_loading(webgpu, color_target_format, graphics_level),
      DemoId::FrameGeneration =>
         triangle::Demo::start_loading(webgpu, color_target_format, graphics_level),
      DemoId::HeadAvatar =>
         triangle::Demo::start_loading(webgpu, color_target_format, graphics_level),
      DemoId::FullBodyAvatar =>
         triangle::Demo::start_loading(webgpu, color_target_format, graphics_level),
      DemoId::ProceduralGeneration =>
         triangle::Demo::start_loading(webgpu, color_target_format,graphics_level),
      _ => stub_demo::Demo::start_loading(),
   }
}

} // mod wasm

#[derive(Clone, Copy)]
pub struct MouseState {
   pub left: f32,
   pub middle: f32,
   pub right: f32,
   pub wheel: f32,
   pub viewport_position: (i32, i32), // origin at bottom-left
   pub canvas_position: (i32, i32), // origin at top-left
}

pub struct ExternalState {
   pub mouse: Rc<Cell<MouseState>>,
   pub screen_size: (u32, u32),
   pub aspect_ratio: f32,
   pub time_now_sec:   f32,
   pub time_now_ms:    u32,
   time_prev_sec:  f32,
   time_prev_ms:   u32,
   pub time_delta_sec: f32,
   pub time_delta_ms:  u32,
   pub time_delta_limit_ms: i32,
   pub frame_idx: usize,
   pub frame_rate: f32,
   #[allow(unused)] pub sound_sample_rate: f32,
   pub graphics_level: GraphicsLevel,
   pub debug_mode: Option<u16>,
}

impl ExternalState {
   pub fn mouse_unit_position(&self) -> (f32, f32) {
      let px_pos = self.mouse.get().viewport_position;
      return (
         px_pos.0 as f32 / self.screen_size.0 as f32,
         px_pos.1 as f32 / self.screen_size.1 as f32,
      )
   }

   pub fn screen_resize(&mut self, (width_px, height_px): (u32, u32)) {
      self.screen_size = (width_px, height_px);
      self.aspect_ratio = width_px as f32 / height_px as f32;
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
            canvas_position: Default::default(),
            viewport_position: Default::default(),
         })),
         screen_size: (1, 1),
         aspect_ratio: 1.0,
         time_delta_sec: Default::default(),
         time_delta_ms: Default::default(),
         time_delta_limit_ms: Default::default(),
         time_now_sec: Default::default(),
         time_now_ms: Default::default(),
         time_prev_sec: Default::default(),
         time_prev_ms: Default::default(),
         frame_idx: Default::default(),
         frame_rate: 1.0,
         sound_sample_rate: Default::default(),
         graphics_level: Default::default(),
         debug_mode: Default::default(),
       }
    }
}

impl ExternalState {
   pub fn begin_frame(&mut self, timestamp_ms: usize) {
      let current_mouse = self.mouse.get();
      self.mouse.set(MouseState {
         viewport_position: (
            current_mouse.canvas_position.0,
            self.screen_size.1 as i32 - current_mouse.canvas_position.1
         ), // NOTE: origin at bottom-left
         ..current_mouse
      });
      let time_now_ms  = timestamp_ms as u32;
      let time_now_sec = timestamp_ms as f32 * 0.001;
      let elapsed_ms  = time_now_ms - self.time_prev_ms;
      let elapsed_sec = time_now_sec - self.time_prev_sec;
      self.time_prev_ms  = time_now_ms;
      self.time_prev_sec = time_now_sec;
      self.time_delta_ms = elapsed_ms.max(1);
      self.time_delta_sec = elapsed_sec.max(1e-6);
      self.time_now_sec += self.time_delta_sec;
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

pub trait IDemo : Drop {
   fn tick(&mut self, state: &ExternalState);
   fn start_switching_graphics_level(&mut self, webgpu: &Webgpu, level: GraphicsLevel) -> Result<(), wgpu::SurfaceError>;
   fn poll_switching_graphics_level(&mut self, webgpu: &Webgpu) -> Result<std::task::Poll<()>, wgpu::SurfaceError>;
   fn progress_switching_graphics_level(&self) -> f32;
   fn render(&mut self, webgpu: &Webgpu, backbuffer: &SurfaceTexture, delta_sec: f32) -> Result<(), wgpu::SurfaceError>;
   fn drop_demo(&mut self, webgpu: &Webgpu);
}

pub trait SimpleFuture {
   type Output;
   type Context;
   // std::future::Future uses std::task::Context<'_>
   // we use a mock argument        Self::Context
   fn simple_poll(self: Pin<&mut Self>, cx: &mut Self::Context) -> std::task::Poll<Self::Output>;
}

impl<T,C> SimpleFuture for Box<dyn SimpleFuture<Output=T, Context=C>> {
    type Output=T;
    type Context=C;

    fn simple_poll(mut self: Pin<&mut Self>, cx: &mut Self::Context) -> std::task::Poll<Self::Output> {
        self.as_mut().simple_poll(cx)
    }
}

pub trait Progress {
   // normalized progress 0.0 - 1.0
   fn progress(&self) -> f32;
}

pub trait Dispose {
   fn dispose(&mut self);
}

pub trait DemoLoadingFuture : SimpleFuture<Output=Box<dyn IDemo>, Context=()> + Dispose + Progress {}