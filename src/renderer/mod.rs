pub mod stub_demo;
pub use stub_demo::StubDemo;
pub mod triangle;
pub use triangle::TriangleDemo;
pub mod webgpu;
pub use webgpu::Webgpu;
pub mod webgpu_utils;

use std::{cell::Cell, pin::Pin, rc::Rc};

use crate::{DemoId, GraphicsLevel};

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
       }
    }
}

impl ExternalState {
   pub fn begin_frame(&mut self, timestamp_ms: usize) {
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
   fn render(&mut self, webgpu: &Webgpu, delta_sec: f32) -> Result<(), wgpu::SurfaceError>;
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

pub fn start_loading_demo<'a>(id: DemoId, webgpu: Rc<Webgpu>, color_target_format: wgpu::TextureFormat, graphics_level: GraphicsLevel) -> Box<dyn DemoLoadingFuture> {
   match id {
      DemoId::Triangle =>
         TriangleDemo::start_loading(webgpu, color_target_format, graphics_level),
      DemoId::Fractal =>
         TriangleDemo::start_loading(webgpu, color_target_format, GraphicsLevel::High),
      DemoId::FrameGeneration =>
         TriangleDemo::start_loading(webgpu, color_target_format, GraphicsLevel::Low),
      DemoId::HeadAvatar =>
         TriangleDemo::start_loading(webgpu, color_target_format, GraphicsLevel::Medium),
      DemoId::FullBodyAvatar =>
         TriangleDemo::start_loading(webgpu, color_target_format, GraphicsLevel::Minimal),
      DemoId::ProceduralGeneration =>
         TriangleDemo::start_loading(webgpu, color_target_format, GraphicsLevel::Ultra),
      _ => StubDemo::start_loading(),
   }
}