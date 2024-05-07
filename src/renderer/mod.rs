pub mod webgpu;
pub mod imgui_web;
use futures::Future;
pub use webgpu::Webgpu;
pub mod demo_state;
pub use demo_state::*;
pub mod history;
pub use history::*;
pub mod global_uniform;
pub use global_uniform::*;
mod pipeline_loader;
mod shader_loader;
pub mod demo_stub;
pub mod demo_uv;
pub mod demo_fractal;
pub mod demo_mesh;
mod preprocessor;

use crate::GraphicsLevel;
use wgpu::SurfaceTexture;

use std::{cell::RefCell, pin::Pin, rc::Rc};

use self::webgpu::Premade;

//#[cfg(feature = "web")]
pub mod wasm {

use super::*;
use crate::DemoId;

#[allow(unused)]
pub fn start_loading_demo(id: DemoId, args: LoadingArgs, graphics_level: GraphicsLevel) -> Box<dyn DemoLoadingFuture> {
   // TOOD: remove option, always require it in demos
   match id {
      DemoId::Uv =>
         demo_uv::Demo::start_loading(args, graphics_level),
      DemoId::Fractal =>
         demo_fractal::Demo::start_loading(args, graphics_level),
      DemoId::FrameGeneration =>
         demo_uv::Demo::start_loading(args, graphics_level),
      DemoId::HeadAvatar =>
         demo_uv::Demo::start_loading(args, graphics_level),
      DemoId::FullBodyAvatar =>
         demo_uv::Demo::start_loading(args, graphics_level),
      DemoId::ProceduralGeneration =>
         demo_uv::Demo::start_loading(args, graphics_level),
      _ => demo_stub::Demo::start_loading(),
   }
}

} // mod wasm

pub struct RenderArgs<'a> {
   pub webgpu: &'a Webgpu,
   pub backbuffer: &'a SurfaceTexture,
   pub global_uniform: &'a GlobalUniform,
   pub time_delta_sec: f64,
}

#[derive(Clone)]
pub struct LoadingArgs {
   pub webgpu: Rc<Webgpu>,
   pub color_texture_format: wgpu::TextureFormat,
   pub global_uniform: Rc<RefCell<GlobalUniform>>,
   pub premade: Rc<Premade>,
}

pub trait IDemo {
   fn tick(&mut self, state: &ExternalState);
   fn start_switching_graphics_level(&mut self, args: LoadingArgs, graphics_level: GraphicsLevel) -> Result<(), wgpu::SurfaceError>;
   fn poll_switching_graphics_level(&mut self, webgpu: &Webgpu) -> Result<std::task::Poll<()>, wgpu::SurfaceError>;
   fn progress_switching_graphics_level(&self) -> f32;
   fn render(&mut self, args: RenderArgs) -> Result<(), wgpu::SurfaceError>;
   fn rebuild_pipelines(&mut self, args: LoadingArgs);
   #[cfg(any(feature = "imgui_win", feature = "imgui_web"))]
   fn render_imgui(&mut self, ui: &imgui::Ui, args: imgui_web::ImguiRenderArgs);
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