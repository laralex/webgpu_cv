use std::rc::Rc;
use futures::Future;

use crate::renderer::pipeline_loader::RenderPipelineFlatDescriptor;
use crate::renderer::webgpu::Utils;

use super::shader_loader::{FragmentShaderVariant, VertexShaderVariant};
use super::{DemoLoadingFuture, DemoLoadingSimpleFuture, Dispose, ExternalState, GraphicsLevel, IDemo, LoadingArgs, Progress, RenderArgs, SimpleFuture, Webgpu};

const VERTEX_SHADER_VARIANT:   VertexShaderVariant   = VertexShaderVariant::TriangleFullscreen;
const FRAGMENT_SHADER_VARIANT: FragmentShaderVariant = FragmentShaderVariant::Uv;

#[derive(Default)]
enum DemoLoadingStage {
   Ready = 0,
   #[default] CompileShaders,
   LinkPrograms,
   StartSwitchingGraphicsLevel,
   SwitchGraphicsLevel,
}

struct DemoLoadingProcess {
   stage: DemoLoadingStage,
   stage_percent: f32,
   graphics_level: GraphicsLevel,
   loading_args: LoadingArgs,
   render_pipeline: Option<Rc<wgpu::RenderPipeline>>,
   vertex_shader: Option<Rc<wgpu::ShaderModule>>,
   fragment_shader: Option<Rc<wgpu::ShaderModule>>,
   loaded_demo: Option<Demo>,
}

impl Dispose for DemoLoadingProcess {
   fn dispose(&mut self) {
      match self.stage {
         DemoLoadingStage::Ready => {
            // demo is fully loaded, its lifetime is now separate, 
            // shouldn't free its resources
         },
         _ => {
            self.render_pipeline.take();
            self.vertex_shader.take();
            self.fragment_shader.take();
            self.stage = DemoLoadingStage::Ready;
            self.loaded_demo.take();
            log::info!("Rust loading drop: TriangleDemo {}", self.stage_percent);
         },
      }
   }
}

impl Drop for DemoLoadingProcess {
   fn drop(&mut self) {
      self.dispose();
   }
}

impl Progress for DemoLoadingProcess {
    fn progress(&self) -> f32 {
        self.stage_percent
    }
}

impl SimpleFuture for DemoLoadingProcess {
   type Output = Box<dyn IDemo>;
   type Context = ();

   fn simple_poll(mut self: std::pin::Pin<&mut Self>, _cx: &mut Self::Context) -> std::task::Poll<Self::Output> {
      use DemoLoadingStage::*;
      match self.stage {
         CompileShaders => {
            let vertex_shader = self.loading_args.webgpu.get_vertex_shader(VERTEX_SHADER_VARIANT, None);
            let fragment_shader = self.loading_args.webgpu.get_fragment_shader(FRAGMENT_SHADER_VARIANT, None);
            self.vertex_shader = Some(vertex_shader);
            self.fragment_shader = Some(fragment_shader);
            self.stage_percent = 0.6;
            self.stage = LinkPrograms;
            std::task::Poll::Pending
         },
         LinkPrograms => {
            let global_uniform = self.loading_args.global_uniform.clone();
            let layout_descriptor = wgpu::PipelineLayoutDescriptor {
               label: Some("Render Pipeline Layout"),
               bind_group_layouts: &[&global_uniform.borrow().bind_group_info.bind_group_layout],
               push_constant_ranges: &[],
            };
            let render_pipeline_layout = self.loading_args.webgpu.device.create_pipeline_layout(
               &layout_descriptor);
            let vs = self.vertex_shader.take().unwrap();
            let fs = self.fragment_shader.take().unwrap();
            self.render_pipeline = Some(self.loading_args.webgpu.get_pipeline(
               &RenderPipelineFlatDescriptor::new(
               &layout_descriptor,
               &wgpu::RenderPipelineDescriptor {
               label: Some("Render Pipeline"),
               layout: Some(&render_pipeline_layout),
               vertex: wgpu::VertexState {
                     module: &vs,
                     entry_point: "vs_main",
                     buffers: &[],
               },
               fragment: Some(wgpu::FragmentState {
                     module: &fs,
                     entry_point: "fs_main",
                     targets: &[Some(wgpu::ColorTargetState {
                        format: self.loading_args.color_texture_format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                     })],
               }),
               primitive: Utils::default_primitive_state(),
               depth_stencil: None, // 1.
               multisample: wgpu::MultisampleState {
                  count: 1,
                  mask: !0,
                  alpha_to_coverage_enabled: false,
               },
               multiview: None,
            })));
            self.stage_percent = 0.7;
            self.stage = StartSwitchingGraphicsLevel;
            std::task::Poll::Pending
         }
         StartSwitchingGraphicsLevel => {
            self.loaded_demo = Some(Demo {
               render_pipeline: self.render_pipeline.take().unwrap(),
               pending_graphics_level_switch: None,
            });
            let graphics_level = self.graphics_level;
            let loading_args = self.loading_args.clone();
            self.loaded_demo.as_mut().unwrap()
                  .start_switching_graphics_level(loading_args, graphics_level)
                  .expect("WebGPU surface error");
            self.stage_percent = 0.75;
            self.stage = SwitchGraphicsLevel;
            std::task::Poll::Pending
         }
         SwitchGraphicsLevel => {
            let webgpu = self.loading_args.webgpu.clone();
            match self.loaded_demo.as_mut().unwrap().poll_switching_graphics_level(webgpu.as_ref()) {
               Ok(std::task::Poll::Pending)  => {
                  self.stage_percent = 0.75 + 0.25 * self.loaded_demo.as_ref().unwrap().progress_switching_graphics_level();
                  self.stage = SwitchGraphicsLevel;
                  std::task::Poll::Pending
               }
               Ok(std::task::Poll::Ready(())) => {
                  self.stage_percent = 1.0;
                  self.stage = Ready;
                  std::task::Poll::Ready(Box::new(
                     self.loaded_demo.take().unwrap()
                  ))
               },
               Err(e) => {
                  eprintln!("Error when switching graphics level: TriangleDemo: {}", e);
                  std::task::Poll::Pending // hopefully will work next frame
               }
            }
         }
         Ready => unreachable!("Should not poll the task again after std::task::Poll::Ready was polled"),
      }
   }
}

impl Future for DemoLoadingProcess {
   type Output = Box<dyn IDemo>;
   
   fn poll(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
      match self.as_mut().simple_poll(&mut ()) {
         std::task::Poll::Pending => {
            cx.waker().wake_by_ref();
            std::task::Poll::Pending
         }
         poll => poll,
      }
   }
}

impl DemoLoadingSimpleFuture for DemoLoadingProcess{}

impl DemoLoadingFuture for DemoLoadingProcess {}

pub struct Demo {
   render_pipeline: Rc<wgpu::RenderPipeline>,
   pending_graphics_level_switch: Option<GraphicsSwitchingProcess>,
}

impl IDemo for Demo {
   fn tick(&mut self, _input: &ExternalState) {

   }

   fn render(&mut self, args: RenderArgs) -> Result<(), wgpu::SurfaceError> {
      let view = Utils::surface_view(args.backbuffer);
      let mut encoder = args.webgpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
         label: Some("Render Encoder"),
      });

      {
         let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
               label: Some("Render Pass"),
               color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                  view: &view,
                  resolve_target: None,
                  ops: wgpu::Operations {
                     load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                     store: wgpu::StoreOp::Store,
                  },
               })],
               depth_stencil_attachment: None,
               occlusion_query_set: None,
               timestamp_writes: None,
         });
         const DEMO_UNIFORM_BIND_GROUP_INDEX: u32 = 0;
         render_pass.set_bind_group(DEMO_UNIFORM_BIND_GROUP_INDEX, &args.global_uniform.bind_group_info.bind_group, &[]);
         render_pass.set_pipeline(&self.render_pipeline);
         render_pass.draw(0..3, 0..1);
      }
   
      // submit will accept anything that implements IntoIter
      args.webgpu.queue.submit(std::iter::once(encoder.finish()));
      Ok(())
   }

   fn rebuild_pipelines(&mut self, _loading_args: LoadingArgs) {

   }

   #[cfg(any(feature = "imgui_win", feature = "imgui_web"))]
   fn render_imgui(&mut self, ui: &imgui::Ui, args: super::imgui_web::ImguiRenderArgs) {
      use imgui::*;
      let window = ui.window("Uv Sandbox Demo");
      window
         .size(args.size, Condition::FirstUseEver)
         .position(args.position, Condition::FirstUseEver)
         .build(|| {
         });
   }

   fn start_switching_graphics_level(&mut self, _args: LoadingArgs, graphics_level: GraphicsLevel) -> Result<(), wgpu::SurfaceError> {
      log::info!("Rust start_switching_graphics_level {}", std::module_path!());
      self.pending_graphics_level_switch = Some(GraphicsSwitchingProcess{
         progress: 0.0,
         graphics_level,
      });
      Ok(())
   }

   fn poll_switching_graphics_level(&mut self, webgpu: &Webgpu) -> Result<std::task::Poll<()>, wgpu::SurfaceError> {
      if self.pending_graphics_level_switch.is_some() {
         Ok(GraphicsSwitchingProcess::poll(self, webgpu))
      } else {
         self.pending_graphics_level_switch.take();
         Ok(std::task::Poll::Ready(()))
      }
   }

   fn progress_switching_graphics_level(&self) -> f32 {
      self.pending_graphics_level_switch
         .as_ref()
         .map(|s| s.progress())
         .unwrap_or_default()
   }

   fn drop_demo(&mut self, _webgpu: &Webgpu) {
      log::info!("Rust demo drop {}", std::module_path!());
   }
}

impl Drop for Demo {
   fn drop(&mut self) {
      // std::mem::drop(self.render_pipeline);
      self.pending_graphics_level_switch.take();
   }
}

impl Demo {
   pub fn start_loading(args: LoadingArgs, graphics_level: GraphicsLevel) -> Box<dyn DemoLoadingFuture> {
      Box::new(DemoLoadingProcess {
         stage: Default::default(),
         stage_percent: 0.0,
         graphics_level,
         loading_args: args,
         render_pipeline: Default::default(),
         vertex_shader: Default::default(),
         fragment_shader: Default::default(),
         loaded_demo: Default::default(),
      })
   }
}

pub struct GraphicsSwitchingProcess {
   progress: f32,
   #[allow(unused)] graphics_level: GraphicsLevel,
}

impl Dispose for GraphicsSwitchingProcess {
   fn dispose(&mut self) {
      log::info!("Rust graphics switching drop {}", std::module_path!());
   }
}

impl Drop for GraphicsSwitchingProcess {
   fn drop(&mut self) {
      self.dispose();
   }
}

impl Progress for GraphicsSwitchingProcess {
    fn progress(&self) -> f32 {
      self.progress
   }
}

impl GraphicsSwitchingProcess {
   pub fn poll(demo: &mut Demo, _webgpu: &Webgpu) -> std::task::Poll<()> {
      let self_ = demo.pending_graphics_level_switch.as_mut().unwrap();
      self_.progress = 1.0;
      std::task::Poll::Ready(())
   }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shaders_compile() {
        let webgpu = futures::executor::block_on(Webgpu::new_offscreen());
        webgpu.get_vertex_shader(VERTEX_SHADER_VARIANT, None);
        webgpu.get_fragment_shader(FRAGMENT_SHADER_VARIANT, None);
    }

}