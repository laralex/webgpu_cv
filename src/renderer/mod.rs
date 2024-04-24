pub mod stub_demo;
pub mod webgpu;
use futures::Future;
pub use webgpu::Webgpu;
pub mod demo_state;
pub use demo_state::*;
mod pipeline_loader;
mod shader_loader;
pub mod triangle;
pub mod fractal;
mod preprocessor;

use crate::GraphicsLevel;
use wgpu::SurfaceTexture;

use std::{pin::Pin, rc::Rc};

//#[cfg(feature = "web")]
pub mod wasm {

use super::*;
use crate::DemoId;

#[allow(unused)]
pub fn start_loading_demo<'window>(id: DemoId, webgpu: Rc<Webgpu<'window>>, color_target_format: wgpu::TextureFormat, graphics_level: GraphicsLevel) -> Box<dyn DemoLoadingFuture + 'window> {
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

pub trait IDemo {
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

pub trait Progress {
   // normalized progress 0.0 - 1.0
   fn progress(&self) -> f32;
}

pub trait Dispose {
   fn dispose(&mut self);
}

pub trait DemoLoadingSimpleFuture : SimpleFuture<Output=Box<dyn IDemo>, Context=()> + Dispose + Progress {}
pub trait DemoLoadingFuture : Future<Output=Box<dyn IDemo>> + Unpin + DemoLoadingSimpleFuture {}