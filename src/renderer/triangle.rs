use std::rc::Rc;
use wgpu::SurfaceTexture;

use crate::renderer::webgpu_utils::WebgpuUtils;

use super::{DemoLoadingFuture, Dispose, ExternalState, GraphicsLevel, IDemo, Progress, SimpleFuture, Webgpu};

const VERTEX_SHADER_SRC:   &str = include_str!("shaders/no_vao_triangle.vert.wgsl");
const FRAGMENT_SHADER_SRC: &str = include_str!("shaders/vertex_color.frag.wgsl");

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
   color_target_format: wgpu::TextureFormat,
   render_pipeline: Option<wgpu::RenderPipeline>,
   vertex_shader: Option<wgpu::ShaderModule>,
   fragment_shader: Option<wgpu::ShaderModule>,
   webgpu: Rc<Webgpu>,
   loaded_demo: Option<TriangleDemo>,
}

impl Dispose for DemoLoadingProcess {
   fn dispose(&mut self) {
      match self.stage {
         DemoLoadingStage::Ready => {
            // demo is fully loaded, its lifetime is now separate, 
            // shouldn't free its resources
         },
         _ => {
            // self.gl.delete_shader(self.vert_shader.as_ref());
            // self.gl.delete_shader(self.frag_shader.as_ref());
            // self.gl.delete_program(self.main_program.as_ref());
            self.stage = DemoLoadingStage::Ready;
            self.loaded_demo.take();
            web_sys::console::log_2(&"Rust loading drop: TriangleDemo".into(), &self.stage_percent.into());
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
            
            let device = &self.webgpu.as_ref().device;
            let vertex_shader = WebgpuUtils::make_vertex_shader(device, VERTEX_SHADER_SRC);
            let fragment_shader = WebgpuUtils::make_fragment_shader(device, FRAGMENT_SHADER_SRC);
            std::mem::drop(device);
            self.vertex_shader = Some(vertex_shader);
            self.fragment_shader = Some(fragment_shader);
            self.stage_percent = 0.6;
            self.stage = LinkPrograms;
            std::task::Poll::Pending
         },
         LinkPrograms => {
            let render_pipeline_layout =
               self.webgpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                  label: Some("Render Pipeline Layout"),
                  bind_group_layouts: &[],
                  push_constant_ranges: &[],
               });
            let vs = self.vertex_shader.take().unwrap();
            let fs = self.fragment_shader.take().unwrap();
            self.render_pipeline = Some(
               self.webgpu.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
                        format: self.color_target_format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                     })],
               }),
               primitive: WebgpuUtils::default_primitive_state(),
               depth_stencil: None, // 1.
               multisample: wgpu::MultisampleState {
                  count: 1,
                  mask: !0,
                  alpha_to_coverage_enabled: false,
               },
               multiview: None,
            }));
            self.stage_percent = 0.7;
            self.stage = StartSwitchingGraphicsLevel;
            std::task::Poll::Pending
         }
         StartSwitchingGraphicsLevel => {
            self.loaded_demo = Some(TriangleDemo {
               render_pipeline: self.render_pipeline.take().unwrap(),
               clear_color: [0.0; 4],
               num_rendered_vertices: 3,
               pending_graphics_level_switch: None,
            });
            let graphics_level = self.graphics_level;
            let webgpu = self.webgpu.clone();
            self.loaded_demo.as_mut().unwrap()
                  .start_switching_graphics_level(webgpu.as_ref(), graphics_level);
            self.stage_percent = 0.75;
            self.stage = SwitchGraphicsLevel;
            std::task::Poll::Pending
         }
         SwitchGraphicsLevel => {
            let webgpu = self.webgpu.clone();
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

impl DemoLoadingFuture for DemoLoadingProcess {}

pub struct TriangleDemo {
   // gl: Rc<GL>,
   // main_program: WebGlProgram,
   render_pipeline: wgpu::RenderPipeline,
   clear_color: [f32; 4],
   num_rendered_vertices: i32,
   pending_graphics_level_switch: Option<GraphicsSwitchingProcess>,
}

impl Drop for TriangleDemo {
   fn drop(&mut self) {
      // self.gl.delete_program(Some(&self.main_program));
   }
}

impl TriangleDemo {
   pub fn start_loading<'a>(webgpu: Rc<Webgpu>, color_target_format: wgpu::TextureFormat, graphics_level: GraphicsLevel) -> Box<dyn DemoLoadingFuture> {
      Box::new(DemoLoadingProcess {
         stage: Default::default(),
         stage_percent: 0.0,
         graphics_level,
         color_target_format,
         render_pipeline: Default::default(),
         vertex_shader: Default::default(),
         fragment_shader: Default::default(),
         loaded_demo: Default::default(),
         webgpu,
      })
   }
}


impl IDemo for TriangleDemo {
   fn tick(&mut self, input: &ExternalState) {
      let mouse_pos = input.mouse_unit_position();
      self.clear_color[0] = input.time_now_sec.sin() * 0.5 + 0.5 * mouse_pos.0;
      self.clear_color[1] = (input.time_now_sec * 1.2).sin() * 0.5 + 0.5;
      self.clear_color[2] = input.mouse.get().left;
      self.clear_color[3] = 1.0;
   }

   fn render(&mut self, webgpu: &Webgpu, backbuffer: &SurfaceTexture, _delta_sec: f32) -> Result<(), wgpu::SurfaceError> {
      let view = WebgpuUtils::surface_view(backbuffer);
      let mut encoder = webgpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
         label: Some("Render Encoder"),
      });

      {
         let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
               label: Some("Render Pass"),
               color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                  view: &view,
                  resolve_target: None,
                  ops: wgpu::Operations {
                     load: wgpu::LoadOp::Clear(
                        wgpu::Color { r: 0.2, g: 0.5, b: 0.3, a: 1.0 }),
                     store: wgpu::StoreOp::Store,
                  },
               })],
               depth_stencil_attachment: None,
               occlusion_query_set: None,
               timestamp_writes: None,
         });

         render_pass.set_pipeline(&self.render_pipeline);
         render_pass.draw(0..3, 0..1); // self.num_rendered>vertices
      }
   
      // submit will accept anything that implements IntoIter
      webgpu.queue.submit(std::iter::once(encoder.finish()));
      Ok(())
   }

   fn start_switching_graphics_level(&mut self, webgpu: &Webgpu, graphics_level: GraphicsLevel) -> Result<(), wgpu::SurfaceError> {
      web_sys::console::log_1(&"Rust start_switching_graphics_level: TriangleDemo".into());
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

   fn drop_demo(&mut self, webgpu: &Webgpu) {
      // gl.delete_program(Some(&self.main_program));
      web_sys::console::log_1(&"Rust demo drop: TriangleDemo".into());
   }
}

pub struct GraphicsSwitchingProcess {
   progress: f32,
   graphics_level: GraphicsLevel,
}

impl Dispose for GraphicsSwitchingProcess {
   fn dispose(&mut self) {
      web_sys::console::log_1(&"Rust graphics switching drop: TriangleDemo".into());
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
   pub fn poll(demo: &mut TriangleDemo, _webgpu: &Webgpu) -> std::task::Poll<()> {
      let self_ = demo.pending_graphics_level_switch.as_mut().unwrap();
      demo.num_rendered_vertices = match self_.graphics_level {
         GraphicsLevel::Minimal => 0,
         GraphicsLevel::Low => 3,
         GraphicsLevel::Medium => 6,
         GraphicsLevel::High => 9,
         GraphicsLevel::Ultra => 12,
      };
      self_.progress = 1.0;
      std::task::Poll::Ready(())
   }
}


#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn shaders_compile() {
        let (device, _) = futures::executor::block_on(Webgpu::new_offscreen());
      //   let result = std::panic::catch_unwind(||
        WebgpuUtils::make_vertex_shader(&device, VERTEX_SHADER_SRC);
        WebgpuUtils::make_fragment_shader(&device, FRAGMENT_SHADER_SRC);
    }
}