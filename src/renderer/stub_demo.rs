use std::task::Poll;
use crate::Webgpu;

use super::{DemoLoadingFuture, Dispose, ExternalState, GraphicsLevel, IDemo, Progress, SimpleFuture};

pub struct StubDemo;

impl Drop for StubDemo {
   fn drop(&mut self) { self.dispose(); }
}

impl Dispose for StubDemo {
   fn dispose(&mut self) { }
}

impl IDemo for StubDemo {
   fn tick(&mut self, _input: &ExternalState) { }

   fn render(&mut self, webgpu: &Webgpu, delta_sec: f32) -> Result<(), wgpu::SurfaceError> {
      let output = webgpu.surface.get_current_texture()?;
      let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
      let mut encoder = webgpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
         label: Some("Render Encoder"),
      });

      {
         let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
               label: Some("Render Pass"),
               color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                  view: &view,
                  resolve_target: None,
                  ops: wgpu::Operations {
                     load: wgpu::LoadOp::Clear(
                        wgpu::Color {r: 0.9, g: 0.0, b: 0.9, a: 1.0,}),
                     store: wgpu::StoreOp::Store,
                  },
               })],
               depth_stencil_attachment: None,
               occlusion_query_set: None,
               timestamp_writes: None,
         });
      }

      // submit will accept anything that implements IntoIter
      webgpu.queue.submit(std::iter::once(encoder.finish()));
      output.present();

      Ok(())
   }

   fn start_switching_graphics_level(&mut self, _webgpu: &Webgpu, _level: GraphicsLevel) -> Result<(), wgpu::SurfaceError> {
      web_sys::console::log_1(&"Rust start_switching_graphics_level: StubDemo".into());
      Ok(())
   }

   fn poll_switching_graphics_level(&mut self, _webgpu: &Webgpu) -> Result<std::task::Poll<()>, wgpu::SurfaceError> {
      Ok(std::task::Poll::Ready(()))
   }

   fn progress_switching_graphics_level(&self) -> f32 {
      0.0
   }

   fn drop_demo(&mut self, _webgpu: &Webgpu) {
      web_sys::console::log_1(&"Rust demo drop: StubDemo".into());
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

   fn simple_poll(mut self: std::pin::Pin<&mut Self>, _cx: &mut Self::Context) -> Poll<Self::Output> {
      Poll::Ready(Box::new(StubDemo{}))
   }
}

impl DemoLoadingFuture for DemoLoadingProcess {}

impl StubDemo {
   pub fn start_loading() -> Box<dyn DemoLoadingFuture> {
      Box::new(DemoLoadingProcess{})
   }
}