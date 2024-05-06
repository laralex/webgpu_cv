use std::task::Poll;
use futures::Future;

use super::{webgpu::Utils, DemoLoadingFuture, DemoLoadingSimpleFuture, Dispose, ExternalState, GraphicsLevel, IDemo, LoadingArgs, Progress, RenderArgs, SimpleFuture, Webgpu};

pub struct Demo;

impl Drop for Demo {
   fn drop(&mut self) { self.dispose(); }
}

impl Dispose for Demo {
   fn dispose(&mut self) { }
}

impl IDemo for Demo {
   fn tick(&mut self, _input: &ExternalState) { }

   fn render(&mut self, args: RenderArgs) -> Result<(), wgpu::SurfaceError> {
      let view = Utils::surface_view(args.backbuffer);
      let mut encoder = args.webgpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
         label: Some("Render Encoder"),
      });

      {
         #[allow(unused_variables)]
         let color = wgpu::Color::WHITE;
         #[cfg(debug_assertions)]
         let color = wgpu::Color{r: 0.9, g: 0.0, b: 0.9, a: 1.0};
         let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
               label: Some("Render Pass"),
               color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                  view: &view,
                  resolve_target: None,
                  ops: wgpu::Operations {
                     load: wgpu::LoadOp::Clear(color),
                     store: wgpu::StoreOp::Store,
                  },
               })],
               depth_stencil_attachment: None,
               occlusion_query_set: None,
               timestamp_writes: None,
         });
      }

      // submit will accept anything that implements IntoIter
      args.webgpu.queue.submit(std::iter::once(encoder.finish()));
      Ok(())
   }

   fn rebuild_pipelines(&mut self, _args: LoadingArgs) {

   }

   #[cfg(any(feature = "imgui_win", feature = "imgui_web"))]
   fn render_imgui(&mut self, _ui: &imgui::Ui, _args: super::imgui_web::ImguiRenderArgs) {

   }

   fn start_switching_graphics_level(&mut self, _args: LoadingArgs, _level: GraphicsLevel) -> Result<(), wgpu::SurfaceError> {
      Ok(())
   }

   fn poll_switching_graphics_level(&mut self, _webgpu: &Webgpu) -> Result<std::task::Poll<()>, wgpu::SurfaceError> {
      Ok(std::task::Poll::Ready(()))
   }

   fn progress_switching_graphics_level(&self) -> f32 {
      0.0
   }

   fn drop_demo(&mut self, _webgpu: &Webgpu) {
      log::info!("Rust demo drop {}", std::module_path!());
   }
}

struct DemoLoadingProcess { }

impl Dispose for DemoLoadingProcess {
   fn dispose(&mut self) { }
}

impl Drop for DemoLoadingProcess {
   fn drop(&mut self) { self.dispose(); }
}

impl Progress for DemoLoadingProcess {
    fn progress(&self) -> f32 { 1.0 }
}

impl SimpleFuture for DemoLoadingProcess {
   type Output = Box<dyn IDemo>;
   type Context = ();

   fn simple_poll(self: std::pin::Pin<&mut Self>, _cx: &mut Self::Context) -> Poll<Self::Output> {
      Poll::Ready(Box::new(Demo{}))
   }
}

impl Future for DemoLoadingProcess {
   type Output = Box<dyn IDemo>;
   
   fn poll(self: std::pin::Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
      Poll::Ready(Box::new(Demo{}))
   }
}

impl DemoLoadingSimpleFuture for DemoLoadingProcess {}

impl DemoLoadingFuture for DemoLoadingProcess {}

impl Demo {
   pub fn start_loading() -> Box<dyn DemoLoadingFuture> {
      Box::new(DemoLoadingProcess{})
   }
}