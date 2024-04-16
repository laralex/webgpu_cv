use std::rc::Rc;
use wgpu::{ShaderStages, SurfaceTexture};
use bytemuck;

use crate::renderer::webgpu::Utils;
use crate::GraphicsLevel;

use super::webgpu::buffer::{Buffer, UniformBuffer};
use super::webgpu::utils::PipelineLayoutBuilder;
use super::webgpu::uniform::{BindGroup, BindGroupBuilfer, PushConstantsCompatibility};
use super::{DemoLoadingFuture, Dispose, ExternalState, IDemo, Progress, SimpleFuture, Webgpu};

const VERTEX_SHADER_SRC:   &str = include_str!("shaders/triangle_fullscreen.vs.wgsl");
const FRAGMENT_SHADER_SRC: &str = include_str!("shaders/mandelbrot.fs.wgsl");
const FRAGMENT_SHADER_AA_SRC: &str = include_str!("shaders/mandelbrot_aa.fs.wgsl");

#[derive(Default)]
enum DemoLoadingStage {
   Ready = 0,
   #[default] CreateShaders,
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
   render_pipelines: Option<FractalRenderPipelines>,
   vertex_shader: Option<wgpu::ShaderModule>,
   fragment_shader_default: Option<wgpu::ShaderModule>,
   fragment_shader_antialiasing: Option<wgpu::ShaderModule>,
   fractal_uniform: Option<BindGroup>,
   fractal_uniform_buffer: Option<UniformBuffer>,
   demo_uniform: Option<BindGroup>,
   demo_uniform_buffer: Option<UniformBuffer>,
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
            self.render_pipelines.take();
            self.vertex_shader.take();
            self.fragment_shader_default.take();
            self.fragment_shader_antialiasing.take();
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
         CreateShaders => {
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
   const MIN_UNIFORM_BUFFER_OFFSET_ALIGNMENT: u64 = 256;

   fn compile_shaders(&mut self) {
      let device = &self.webgpu.as_ref().device;
      let vertex_shader = Utils::make_vertex_shader(device, VERTEX_SHADER_SRC);
      let fragment_shader_default = Utils::make_fragment_shader(device, FRAGMENT_SHADER_SRC);
      let fragment_shader_antialiasing = Utils::make_fragment_shader(device, FRAGMENT_SHADER_AA_SRC);
      std::mem::drop(device);
      self.vertex_shader = Some(vertex_shader);
      self.fragment_shader_default = Some(fragment_shader_default);
      self.fragment_shader_antialiasing = Some(fragment_shader_antialiasing);
   }

   fn create_uniforms(&mut self) {
      let uniform_visibility = ShaderStages::FRAGMENT;
      let uniform_usage = wgpu::BufferUsages::COPY_DST;

      let demo_static_uniform_size = std::mem::size_of::<DemoStaticUniformData>() as u64;
      let demo_dynamic_uniform_size = std::mem::size_of::<DemoDynamicUniformData>() as u64;
      let demo_buffer = Buffer::new_uniform_size(
         &self.webgpu.device, Self::MIN_UNIFORM_BUFFER_OFFSET_ALIGNMENT + demo_dynamic_uniform_size,
          uniform_usage, Some("Demo Bind Buffer"));
      let fractal_buffer = Buffer::new_uniform::<DemoDynamicUniformData>(
         &self.webgpu.device, uniform_usage, Some("Fractal Bind Buffer"));
      
      self.demo_uniform = Some(BindGroup::builder()
         .with_uniform_buffer_range(0, uniform_visibility,
            &demo_buffer.buffer, (0, demo_static_uniform_size))
         .with_uniform_buffer_range(1, uniform_visibility,
            &demo_buffer.buffer, (Self::MIN_UNIFORM_BUFFER_OFFSET_ALIGNMENT, demo_dynamic_uniform_size))
         .build(&self.webgpu.device, Some("Demo Bind Group"), None)
      );
      self.demo_uniform_buffer = Some(demo_buffer);

      self.fractal_uniform = Some(BindGroup::builder()
         .with_uniform_buffer(0, uniform_visibility, &fractal_buffer.buffer)
         .build(&self.webgpu.device, Some("Fractal Bind Group"), None)
      );
      self.fractal_uniform_buffer = Some(fractal_buffer);
   }

   fn create_pipelines(&mut self) {
      let render_pipeline_layout = PipelineLayoutBuilder::new()
         .with(self.demo_uniform.as_ref().unwrap())
         .with(self.fractal_uniform.as_ref().unwrap())
         .build(&self.webgpu.device, Some("Render Pipeline Layout"));
      let vs = self.vertex_shader.take().unwrap();
      let fs = self.fragment_shader_default.take().unwrap();
      let fs_aa = self.fragment_shader_antialiasing.take().unwrap();
      self.render_pipelines = Some(FractalRenderPipelines{
         default: self.create_render_pipeline("Render Pipeline - Default", &render_pipeline_layout, &vs, &fs),
         antiialiasing: self.create_render_pipeline("Render Pipeline - AA", &render_pipeline_layout, &vs, &fs_aa),
      })
   }

   fn create_render_pipeline(&self, label: &str, layout: &wgpu::PipelineLayout, vs: &wgpu::ShaderModule, fs: &wgpu::ShaderModule) -> wgpu::RenderPipeline {
      self.webgpu.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
         label: Some(label),
         layout: Some(&layout),
         vertex: wgpu::VertexState {
               module: vs,
               entry_point: "vs_main",
               buffers: &[],
         },
         fragment: Some(wgpu::FragmentState {
               module: fs,
               entry_point: "fs_main",
               targets: &[Some(wgpu::ColorTargetState {
                  format: self.color_target_format,
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
      })
   }

   fn switch_graphics_level(&mut self) {
      self.loaded_demo = Some(Demo {
         render_pipelines: self.render_pipelines.take().unwrap(),
         use_antialiasing: false,
         num_rendered_vertices: 3,
         pending_graphics_level_switch: None,
         pending_write_static_uniform: false,
         fractal_uniform_data: FractalUniformData {
            fractal_center: [-1.1900443, 0.3043895],
            fractal_zoom: 2.0,
            num_iterations: 1000,
         },
         fractal_buffer_offset: 0,
         fractal_uniform_buffer: self.fractal_uniform_buffer.take().unwrap(),
         fractal_uniform: self.fractal_uniform.take().unwrap(),
         demo_static_uniform_data: DemoStaticUniformData {
            color_attachment_size: [1, 1],
            aspect_ratio: 1.0,
            is_debug: 0.0,
         },
         demo_static_buffer_offset: 0,
         demo_dynamic_uniform_data: DemoDynamicUniformData {
            mouse_position: [0.0, 0.0],
            __padding: Default::default(),
         },
         demo_dynamic_buffer_offset: Self::MIN_UNIFORM_BUFFER_OFFSET_ALIGNMENT,
         demo_uniform_buffer: self.demo_uniform_buffer.take().unwrap(),
         demo_uniform: self.demo_uniform.take().unwrap(),
      });
      let webgpu = self.webgpu.clone();
      self.loaded_demo.as_mut().unwrap()
            .start_switching_graphics_level(webgpu.as_ref(), self.graphics_level);
   }
}

struct FractalRenderPipelines {
   default: wgpu::RenderPipeline,
   antiialiasing: wgpu::RenderPipeline
}
pub struct Demo {
   render_pipelines: FractalRenderPipelines,
   use_antialiasing: bool,
   num_rendered_vertices: i32,
   pending_graphics_level_switch: Option<GraphicsSwitchingProcess>,
   pending_write_static_uniform: bool,
   fractal_uniform_data: FractalUniformData,
   fractal_uniform_buffer: UniformBuffer,
   fractal_uniform: BindGroup,
   fractal_buffer_offset: u64,
   demo_static_uniform_data: DemoStaticUniformData,
   demo_static_buffer_offset: u64,
   demo_dynamic_uniform_data: DemoDynamicUniformData,
   demo_dynamic_buffer_offset: u64,
   demo_uniform_buffer: UniformBuffer,
   demo_uniform: BindGroup,
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
struct DemoStaticUniformData {
   color_attachment_size: [u32; 2],
   aspect_ratio: f32,
   is_debug: f32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct DemoDynamicUniformData {
   mouse_position: [f32; 2],
   __padding: [u32; 2],
}

impl IDemo for Demo {
   fn tick(&mut self, input: &ExternalState) {
      self.fractal_uniform_data.fractal_zoom -= self.fractal_uniform_data.fractal_zoom * 0.25 * input.time_delta_sec() as f32;
      self.demo_static_uniform_data.aspect_ratio = input.aspect_ratio();
      self.demo_static_uniform_data.is_debug = input.debug_mode.map_or(0.0, f32::from);
      self.demo_static_uniform_data.color_attachment_size = input.screen_size.into();
      self.pending_write_static_uniform = true; // TODO: get from external state
      self.demo_dynamic_uniform_data.mouse_position = input.mouse_unit_position().into();
   }

   fn render(&mut self, webgpu: &Webgpu, backbuffer: &SurfaceTexture, _delta_sec: f64) -> Result<(), wgpu::SurfaceError> {
      let view = Utils::surface_view(backbuffer);
      let mut encoder = webgpu.device.create_command_encoder(
         &wgpu::CommandEncoderDescriptor { label: Some("Render Encoder"), });

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

         render_pass.set_pipeline(
            if self.use_antialiasing { &self.render_pipelines.antiialiasing }
            else { &self.render_pipelines.default }
         );
         if self.pending_write_static_uniform {
            self.demo_uniform_buffer.write(&webgpu.queue, self.demo_static_buffer_offset, &[self.demo_static_uniform_data]);
         }
         self.demo_uniform_buffer.write(&webgpu.queue, self.demo_dynamic_buffer_offset, &[self.demo_dynamic_uniform_data]);
         self.fractal_uniform_buffer.write(&webgpu.queue, self.fractal_buffer_offset, &[self.fractal_uniform_data]);
         const DEMO_UNIFORM_BIND_GROUP_INDEX: u32 = 0;
         const FRACTAL_UNIFORM_BIND_GROUP_INDEX: u32 = 1;
         render_pass.set_bind_group(DEMO_UNIFORM_BIND_GROUP_INDEX, &self.demo_uniform.bind_group, &[]);
         render_pass.set_bind_group(FRACTAL_UNIFORM_BIND_GROUP_INDEX, &self.fractal_uniform.bind_group, &[]);
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
      match self.pending_graphics_level_switch {
         Some(_) => Ok(GraphicsSwitchingProcess::poll(self, webgpu)),
         _ => Ok(std::task::Poll::Ready(())),
      }
   }

   fn progress_switching_graphics_level(&self) -> f32 {
      self.pending_graphics_level_switch.as_ref()
         .map_or(0.0, |s| s.progress())
   }

   fn drop_demo(&mut self, webgpu: &Webgpu) {
      web_sys::console::log_2(&"Rust demo drop custom".into(), &std::module_path!().into());
   }
}


impl Drop for Demo {
   fn drop(&mut self) {
      web_sys::console::log_2(&"Rust demo drop".into(), &std::module_path!().into());
   }
}

impl Demo {
   pub fn start_loading<'a>(webgpu: Rc<Webgpu>, color_target_format: wgpu::TextureFormat, graphics_level: GraphicsLevel) -> Box<dyn DemoLoadingFuture> {
      Box::new(DemoLoadingProcess {
         stage: Default::default(),
         stage_percent: 0.0,
         graphics_level,
         color_target_format,
         render_pipelines: Default::default(),
         vertex_shader: Default::default(),
         fragment_shader_default: Default::default(),
         fragment_shader_antialiasing: Default::default(),
         fractal_uniform: Default::default(),
         fractal_uniform_buffer: Default::default(),
         demo_uniform: Default::default(),
         demo_uniform_buffer: Default::default(),
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
      (demo.fractal_uniform_data.num_iterations, demo.use_antialiasing) = match self_.graphics_level {
        GraphicsLevel::Minimal => (25, false),
        GraphicsLevel::Low => (250, false),
        GraphicsLevel::Medium => (500, true),
        GraphicsLevel::High => (1000, true),
        GraphicsLevel::Ultra => (1500, true),
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
        Utils::make_vertex_shader(&device, VERTEX_SHADER_SRC);
        Utils::make_fragment_shader(&device, FRAGMENT_SHADER_SRC);
        Utils::make_fragment_shader(&device, FRAGMENT_SHADER_AA_SRC);
    }
}