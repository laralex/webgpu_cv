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
   pub canvas_position_px: (i32, i32), // origin at top-left
}

#[derive(Default)]
struct DerivedState {
   pub aspect_ratio: f32,
   pub time_now_sec:   f64,
   pub time_prev_sec:  f64,
   pub time_delta_sec: f64,
   pub frame_rate: f32,
   pub mouse_viewport_position_px: (i32, i32), // origin at bottom-left
}

pub struct ExternalState {
   pub mouse: Rc<Cell<MouseState>>,
   pub screen_size: (u32, u32),
   pub time_now_ms:    f64,
   pub time_prev_ms:   f64,
   pub time_delta_ms:  f64,
   pub time_delta_limit_ms: f64,
   pub frame_idx: usize,
   pub graphics_level: GraphicsLevel,
   pub debug_mode: Option<u16>,
   derived: DerivedState,
}

impl ExternalState {
   pub fn mouse_unit_position(&self) -> (f32, f32) {
      let px_pos = self.mouse_viewport_position_px();
      return (
         px_pos.0 as f32 / self.screen_size.0 as f32,
         px_pos.1 as f32 / self.screen_size.1 as f32,
      )
   }

   pub fn update_derived_state(&mut self) {
      let now = self.time_now_ms * 0.001;
      let then = self.time_prev_ms * 0.001;
      let delta = self.derived.time_now_sec - self.derived.time_prev_sec;
      
      let current_mouse = self.mouse.get();
      self.derived = DerivedState {
         aspect_ratio: self.screen_size.0 as f32 / self.screen_size.1 as f32,
         time_now_sec: now,
         time_prev_sec: then,
         time_delta_sec: delta,
         frame_rate: (1.0 / delta) as f32,
         mouse_viewport_position_px: (
            current_mouse.canvas_position_px.0,
            self.screen_size.1 as i32 - current_mouse.canvas_position_px.1
         ),
      }

   }

   pub fn aspect_ratio(&self) -> f32 { self.derived.aspect_ratio }
   pub fn time_now_sec(&self) -> f64 { self.derived.time_now_sec }
   pub fn time_prev_sec(&self) -> f64 { self.derived.time_prev_sec }
   pub fn time_delta_sec(&self) -> f64 { self.derived.time_delta_sec }
   pub fn frame_rate(&self) -> f32 { self.derived.frame_rate }
   pub fn mouse_viewport_position_px(&self) -> (i32, i32) { self.derived.mouse_viewport_position_px }

   pub fn screen_resize(&mut self, (width_px, height_px): (u32, u32)) {
      self.screen_size = (width_px, height_px);
   }

   pub fn override_time(&mut self, timestamp_ms: f64, frame_idx: usize) {
      self.frame_idx = frame_idx;
      self.time_delta_ms = 0.0; // .max(1)
      self.time_prev_ms  = timestamp_ms;
      self.time_now_ms   = timestamp_ms;
      self.update_derived_state();
   }

   pub fn tick(&mut self, tick_timestamp_ms: f64) {
      self.frame_idx += 1;
      self.time_delta_ms = tick_timestamp_ms - self.time_prev_ms;
      self.time_prev_ms  = self.time_now_ms;
      self.time_now_ms   = tick_timestamp_ms;
      self.update_derived_state();
   }

   pub fn tick_from_delta(&mut self, tick_delta_ms: f64) {
      self.tick(self.time_prev_ms + tick_delta_ms);
   }

   pub fn dismiss_events(&mut self) {
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
            canvas_position_px: Default::default(),
         })),
         screen_size: (1, 1),
         time_delta_ms: Default::default(),
         time_delta_limit_ms: Default::default(),
         time_now_ms: Default::default(),
         time_prev_ms: Default::default(),
         frame_idx: Default::default(),
         graphics_level: Default::default(),
         debug_mode: Default::default(),
         derived: Default::default(),
       }
    }
}

pub trait IDemo : Drop {
   fn tick(&mut self, state: &ExternalState);
   fn start_switching_graphics_level(&mut self, webgpu: &Webgpu, level: GraphicsLevel) -> Result<(), wgpu::SurfaceError>;
   fn poll_switching_graphics_level(&mut self, webgpu: &Webgpu) -> Result<std::task::Poll<()>, wgpu::SurfaceError>;
   fn progress_switching_graphics_level(&self) -> f32;
   fn render(&mut self, webgpu: &Webgpu, backbuffer: &SurfaceTexture, delta_sec: f64) -> Result<(), wgpu::SurfaceError>;
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