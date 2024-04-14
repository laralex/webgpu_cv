use std::rc::Rc;
use wgpu::{ShaderStages, SurfaceTexture};
use bytemuck;

use crate::renderer::webgpu_utils::WebgpuUtils;
use crate::GraphicsLevel;

use super::{webgpu_utils::{PipelineLayoutBuilder, PushConstantsCompatibility, UniformGroup}, DemoLoadingFuture, Dispose, ExternalState, IDemo, Progress, SimpleFuture, Webgpu};

const VERTEX_SHADER_SRC:   &str = include_str!("shaders/triangle_fullscreen.vs.wgsl");
const FRAGMENT_SHADER_SRC: &str = include_str!("shaders/mandelbrot.fs.wgsl");
// const FRAGMENT_SHADER_SRC: &str = include_str!("shaders/uv.fs.wgsl");

#[derive(Default)]
enum DemoLoadingStage {
   Ready = 0,
   #[default] CompileShaders,
   CreatePipelines,
   CreateUniforms,
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
   fractal_uniform: Option<PushConstantsCompatibility>,
   demo_uniform: Option<PushConstantsCompatibility>,
   webgpu: Rc<Webgpu>,
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
            self.fractal_uniform.take();
            self.render_pipeline.take();
            self.vertex_shader.take();
            self.fragment_shader.take();
            self.stage = DemoLoadingStage::Ready;
            self.loaded_demo.take();
            web_sys::console::log_3(&"Rust loading drop".into(), &std::module_path!().into(), &self.stage_percent.into());
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
            self.compile_shaders();
            self.stage_percent += 0.1;
            self.stage = CreateUniforms;
            std::task::Poll::Pending
         },
         CreateUniforms => {
            self.create_uniforms();
            self.stage_percent += 0.1;
            self.stage = CreatePipelines;
            std::task::Poll::Pending
         }
         CreatePipelines => {
            self.create_pipelines();
            self.stage_percent += 0.1;
            self.stage = StartSwitchingGraphicsLevel;
            std::task::Poll::Pending
         }
         StartSwitchingGraphicsLevel => {
            self.switch_graphics_level();
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
                  eprintln!("Error when switching graphics level: {}: {}", std::module_path!(), e);
                  std::task::Poll::Pending // hopefully will work next frame
               }
            }
         }
         Ready => unreachable!("Should not poll the task again after std::task::Poll::Ready was polled"),
      }
   }
}

impl DemoLoadingFuture for DemoLoadingProcess {}

impl DemoLoadingProcess {
   fn compile_shaders(&mut self) {
      let device = &self.webgpu.as_ref().device;
      let vertex_shader = WebgpuUtils::make_vertex_shader(device, VERTEX_SHADER_SRC);
      let fragment_shader = WebgpuUtils::make_fragment_shader(device, FRAGMENT_SHADER_SRC);
      std::mem::drop(device);
      self.vertex_shader = Some(vertex_shader);
      self.fragment_shader = Some(fragment_shader);
   }

   fn create_uniforms(&mut self) {
      const DEMO_UNIFORM_BIND_GROUP: u32 = 0;
      const FRACTAL_BIND_GROUP_INDEX: u32 = 1;

      self.demo_uniform = Some(WebgpuUtils::make_compatible_push_constant::<DemoUniformData>(
         &self.webgpu.device,  wgpu::ShaderStages::FRAGMENT,
         DEMO_UNIFORM_BIND_GROUP)
      );

      self.fractal_uniform = Some(WebgpuUtils::make_compatible_push_constant::<FractalUniformData>(
         &self.webgpu.device,  wgpu::ShaderStages::FRAGMENT,
         FRACTAL_BIND_GROUP_INDEX)
      );
   }

   fn create_pipelines(&mut self) {
      let render_pipeline_layout = PipelineLayoutBuilder::new()
         .with(self.demo_uniform.as_ref().unwrap())
         .with(self.fractal_uniform.as_ref().unwrap())
         .build(&self.webgpu.device, Some("Render Pipeline Layout"));
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
   }

   fn switch_graphics_level(&mut self) {
      self.loaded_demo = Some(Demo {
         render_pipeline: self.render_pipeline.take().unwrap(),
         num_rendered_vertices: 3,
         pending_graphics_level_switch: None,
         fractal_uniform_data: FractalUniformData {
            fractal_center: [-1.1900443, 0.3043895],
            fractal_zoom: 2.0,
            num_iterations: 1000,
         },
         fractal_uniform: self.fractal_uniform.take().unwrap(),
         demo_uniform_data: DemoUniformData {
            mouse_position: [0.0, 0.0],
            aspect_ratio: 1.0,
            is_debug: 0.0,
         },
         demo_uniform: self.demo_uniform.take().unwrap(),
      });
      let webgpu = self.webgpu.clone();
      self.loaded_demo.as_mut().unwrap()
            .start_switching_graphics_level(webgpu.as_ref(), self.graphics_level);
   }
}

pub struct Demo {
   render_pipeline: wgpu::RenderPipeline,
   num_rendered_vertices: i32,
   pending_graphics_level_switch: Option<GraphicsSwitchingProcess>,
   fractal_uniform_data: FractalUniformData,
   fractal_uniform: PushConstantsCompatibility,
   demo_uniform_data: DemoUniformData,
   demo_uniform: PushConstantsCompatibility,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct FractalUniformData {
   fractal_center: [f32; 2],
   fractal_zoom: f32,
   num_iterations: i32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct DemoUniformData {
   mouse_position: [f32; 2],
   aspect_ratio: f32,
   is_debug: f32,
}

impl IDemo for Demo {
   fn tick(&mut self, input: &ExternalState) {
      self.fractal_uniform_data.fractal_zoom -= self.fractal_uniform_data.fractal_zoom * 0.25 * input.time_delta_sec() as f32;
      self.demo_uniform_data.aspect_ratio = input.aspect_ratio();
      self.demo_uniform_data.mouse_position = input.mouse_unit_position().into();
      self.demo_uniform_data.is_debug = input.debug_mode.map_or(0.0, f32::from);
   }

   fn render(&mut self, webgpu: &Webgpu, backbuffer: &SurfaceTexture, _delta_sec: f64) -> Result<(), wgpu::SurfaceError> {
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
                     load: wgpu::LoadOp::Load,
                     store: wgpu::StoreOp::Store,
                  },
               })],
               depth_stencil_attachment: None,
               occlusion_query_set: None,
               timestamp_writes: None,
         });

         render_pass.set_pipeline(&self.render_pipeline);
         WebgpuUtils::bind_compatible_push_constant(&mut render_pass, &webgpu.queue,
            &self.fractal_uniform, bytemuck::cast_slice(&[self.fractal_uniform_data]));
         WebgpuUtils::bind_compatible_push_constant(&mut render_pass, &webgpu.queue,
            &self.demo_uniform, bytemuck::cast_slice(&[self.demo_uniform_data]));
         render_pass.draw(0..3, 0..1); // self.num_rendered>vertices
      }
   
      // submit will accept anything that implements IntoIter
      webgpu.queue.submit(std::iter::once(encoder.finish()));
      Ok(())
   }

   fn start_switching_graphics_level(&mut self, _webgpu: &Webgpu, graphics_level: GraphicsLevel) -> Result<(), wgpu::SurfaceError> {
      web_sys::console::log_3(&"Rust start_switching_graphics_level".into(), &std::module_path!().into(), &graphics_level.into());
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
      web_sys::console::log_2(&"Rust demo drop".into(), &std::module_path!().into());
   }
}


impl Drop for Demo {
   fn drop(&mut self) {
      //std::mem::drop(self.render_pipeline);
      self.pending_graphics_level_switch.take();
   }
}

impl Demo {
   pub fn start_loading<'a>(webgpu: Rc<Webgpu>, color_target_format: wgpu::TextureFormat, graphics_level: GraphicsLevel) -> Box<dyn DemoLoadingFuture> {
      Box::new(DemoLoadingProcess {
         stage: Default::default(),
         stage_percent: 0.0,
         graphics_level,
         color_target_format,
         render_pipeline: Default::default(),
         vertex_shader: Default::default(),
         fragment_shader: Default::default(),
         fractal_uniform: Default::default(),
         demo_uniform: Default::default(),
         loaded_demo: Default::default(),
         webgpu,
      })
   }
}

pub struct GraphicsSwitchingProcess {
   progress: f32,
   graphics_level: GraphicsLevel,
}

impl Dispose for GraphicsSwitchingProcess {
   fn dispose(&mut self) {
      web_sys::console::log_2(&"Rust graphics switching drop".into(), &std::module_path!().into());
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
      demo.fractal_uniform_data.num_iterations = match self_.graphics_level {
        GraphicsLevel::Minimal => 25,
        GraphicsLevel::Low => 250,
        GraphicsLevel::Medium => 500,
        GraphicsLevel::High => 1000,
        GraphicsLevel::Ultra => 1500,
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
        WebgpuUtils::make_vertex_shader(&device, VERTEX_SHADER_SRC);
        WebgpuUtils::make_fragment_shader(&device, FRAGMENT_SHADER_SRC);
    }
}