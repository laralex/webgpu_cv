use std::ops::Rem;
use std::rc::Rc;
use futures::Future;
use wgpu::{PipelineLayoutDescriptor, ShaderStages, SurfaceTexture};
use bytemuck;

use crate::renderer::webgpu::Utils;
use crate::timer::ScopedTimer;
use crate::GraphicsLevel;

use super::pipeline_loader::RenderPipelineFlatDescriptor;
use super::preprocessor::Preprocessor;
use super::shader_loader::{FragmentShaderVariant, VertexShaderVariant};
use super::webgpu::buffer::{Buffer, UniformBuffer};
use super::webgpu::utils::PipelineLayoutBuilder;
use super::webgpu::uniform::BindGroup;
use super::{DemoLoadingFuture, DemoLoadingSimpleFuture, Dispose, ExternalState, IDemo, Progress, SimpleFuture, Webgpu};

const VERTEX_SHADER_VARIANT: VertexShaderVariant = VertexShaderVariant::TriangleFullscreen;
// const FRAGMENT_SHADER_VARIANT: FragmentShaderVariant = FragmentShaderVariant::Uv;
const FRAGMENT_SHADER_VARIANT: FragmentShaderVariant = FragmentShaderVariant::FractalMandelbrot;

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
   vertex_shader: Option<Rc<wgpu::ShaderModule>>,
   fragment_shader_default: Option<Rc<wgpu::ShaderModule>>,
   fragment_shader_antialiasing: Option<Rc<wgpu::ShaderModule>>,
   uniform_groups: Option<Vec<BindGroup>>,
   fractal_uniform_buffer: Option<UniformBuffer>,
   demo_uniform_buffer: Option<UniformBuffer>,
   demo_stable_buffer_offset: u64,
   demo_dynamic_buffer_offset: u64,
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
            self.uniform_groups.take();
            self.render_pipelines.take();
            self.vertex_shader.take();
            self.fragment_shader_default.take();
            self.fragment_shader_antialiasing.take();
            self.stage = DemoLoadingStage::Ready;
            self.loaded_demo.take();
            #[cfg(feature = "web")]
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
            if self.vertex_shader.is_none() {
               self.compile_shader_vert();
            } else if self.fragment_shader_default.is_none() {
               self.compile_shader_frag_default();
            } else if self.fragment_shader_antialiasing.is_none() {
               self.compile_shader_frag_aa();
            } else {
               self.stage = CreateUniforms;
            }
            self.stage_percent += 0.1;
            std::task::Poll::Pending
         },
         CreateUniforms => {
            self.build_uniforms();
            self.stage_percent += 0.1;
            self.stage = CreatePipelines;
            std::task::Poll::Pending
         }
         CreatePipelines => {
            self.build_pipelines();
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

impl DemoLoadingSimpleFuture for DemoLoadingProcess {}

impl DemoLoadingFuture for DemoLoadingProcess {}


impl DemoLoadingProcess {
   fn new(webgpu: Rc<Webgpu>, color_target_format: wgpu::TextureFormat, graphics_level: GraphicsLevel) -> Self {
      Self {
         stage: Default::default(),
         stage_percent: 0.0,
         graphics_level: graphics_level,
         color_target_format: color_target_format,
         render_pipelines: Default::default(),
         vertex_shader: Default::default(),
         fragment_shader_default: Default::default(),
         fragment_shader_antialiasing: Default::default(),
         uniform_groups: Default::default(),
         fractal_uniform_buffer: Default::default(),
         demo_uniform_buffer: Default::default(),
         demo_stable_buffer_offset: Default::default(),
         demo_dynamic_buffer_offset: Default::default(),
         loaded_demo: Default::default(),
         webgpu,
      }
   }

   #[allow(unused)]
   fn compile_shaders(&mut self) {
      self.compile_shader_vert();
      self.compile_shader_frag_default();
      self.compile_shader_frag_aa();
   }

   fn rebuild_pipelines(&mut self) {
      self.compile_shaders();
      self.build_uniforms();
      self.build_pipelines();
   }

   fn compile_shader_vert(&mut self) {
      let _t = ScopedTimer::new("compile_shader_vert");
      let vertex_shader = self.webgpu.get_vertex_shader(VERTEX_SHADER_VARIANT, None);
      self.vertex_shader = Some(vertex_shader);
   }

   fn compile_shader_frag_default(&mut self) {
      let _t = ScopedTimer::new("compile_shader_frag_default");
      let mut preprocessor = Preprocessor::new();
      let fragment_shader_default = self.webgpu.get_fragment_shader(FRAGMENT_SHADER_VARIANT, Some(&mut preprocessor));
      self.fragment_shader_default = Some(fragment_shader_default);
   }

   fn compile_shader_frag_aa(&mut self) {
      let _t = ScopedTimer::new("compile_shader_frag_aa");
      let mut preprocessor = Preprocessor::new();
      preprocessor.define("USE_ANTIALIASING", "1");
      let fragment_shader_antialiasing = self.webgpu.get_fragment_shader(FRAGMENT_SHADER_VARIANT, Some(&mut preprocessor));
      self.fragment_shader_antialiasing = Some(fragment_shader_antialiasing);
   }

   fn build_uniforms(&mut self) {
      let _t = ScopedTimer::new("create_uniforms");
      let uniform_visibility = ShaderStages::FRAGMENT;
      let uniform_usage = wgpu::BufferUsages::COPY_DST;
      self.demo_stable_buffer_offset = 0;
      self.demo_dynamic_buffer_offset = self.webgpu.device.limits().min_uniform_buffer_offset_alignment as u64;

      let demo_stable_uniform_size = std::mem::size_of::<DemoStableUniformData>() as u64;
      let demo_dynamic_uniform_size = std::mem::size_of::<DemoDynamicUniformData>() as u64;
      let demo_buffer = Buffer::new_uniform_size(
         &self.webgpu.device, self.demo_dynamic_buffer_offset + demo_dynamic_uniform_size,
          uniform_usage, Some("Demo Bind Buffer"));
      let fractal_buffer = Buffer::new_uniform::<FractalUniformData>(
         &self.webgpu.device, uniform_usage, Some("Fractal Bind Buffer"));
      let demo_uniform_group = BindGroup::builder()
         .with_uniform_buffer_range(0, uniform_visibility,
            &demo_buffer.buffer, (self.demo_stable_buffer_offset, demo_stable_uniform_size))
         .with_uniform_buffer_range(1, uniform_visibility,
            &demo_buffer.buffer, (self.demo_dynamic_buffer_offset, demo_dynamic_uniform_size))
         .build(&self.webgpu.device, Some("Demo Bind Group"), None);
      self.demo_uniform_buffer = Some(demo_buffer);

      let fractal_uniform_group = BindGroup::builder()
         .with_uniform_buffer(0, uniform_visibility, &fractal_buffer.buffer)
         .build(&self.webgpu.device, Some("Fractal Bind Group"), None);
      self.fractal_uniform_buffer = Some(fractal_buffer);

      self.uniform_groups = Some(vec![demo_uniform_group, fractal_uniform_group]);
   }

   fn build_pipelines(&mut self) {
      let _t = ScopedTimer::new("create_pipelines");
      let builder = PipelineLayoutBuilder::from_uniform_iter(self.uniform_groups.as_ref().unwrap().iter());
      let pipeline_layout_descr = builder.build_descriptor(Some("Render Pipeline Layout"));
      
      let vs = self.vertex_shader.take().unwrap();
      let fs = self.fragment_shader_default.take().unwrap();
      let fs_aa = self.fragment_shader_antialiasing.take().unwrap();
      self.render_pipelines = Some(FractalRenderPipelines{
         default: self.build_render_pipeline("Render Pipeline - Default",
            &pipeline_layout_descr, &vs, &fs),
         antialiasing: self.build_render_pipeline("Render Pipeline - AA",
            &pipeline_layout_descr, &vs, &fs_aa),
      });
   }

   fn build_render_pipeline(&self, label: &str, layout_descriptor: &wgpu::PipelineLayoutDescriptor, vs: &wgpu::ShaderModule, fs: &wgpu::ShaderModule) -> Rc<wgpu::RenderPipeline> {
      let _t = ScopedTimer::new("create_render_pipeline");
      let render_pipeline_layout = self.webgpu.device.create_pipeline_layout(&layout_descriptor);
      self.webgpu.get_pipeline(&RenderPipelineFlatDescriptor::new(
         // &self.uniform_groups,
         layout_descriptor,
         &wgpu::RenderPipelineDescriptor {
            label: Some(label),
            layout: Some(&render_pipeline_layout),
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
         },
      ))
   }

   fn switch_graphics_level(&mut self) {
      let default_fractal_zoom = 2.0;
      self.loaded_demo = Some(Demo {
         current_graphics_level: self.graphics_level,
         render_pipelines: self.render_pipelines.take().unwrap(),
         use_antialiasing: false,
         num_rendered_vertices: 3,
         pending_graphics_level_switch: None,
         pending_write_stable_uniform: true,
         default_fractal_zoom,
         fractal_uniform_data: FractalUniformData {
            fractal_center: [-1.1900443, 0.3043895],
            fractal_zoom: default_fractal_zoom,
            num_iterations: 1000,
            color_bias: [3.0, 3.5, 4.0],
            color_power: 0.22,
         },
         fractal_buffer_offset: 0,
         fractal_uniform_buffer: self.fractal_uniform_buffer.take().unwrap(),
         uniform_groups: self.uniform_groups.take().unwrap(),
         demo_stable_uniform_data: DemoStableUniformData {
            color_attachment_size: [1, 1],
            aspect_ratio: 1.0,
            is_debug: 0.0,
         },
         demo_stable_buffer_offset: self.demo_stable_buffer_offset,
         demo_dynamic_uniform_data: DemoDynamicUniformData {
            mouse_position: [0.0, 0.0],
            __padding: Default::default(),
         },
         demo_dynamic_buffer_offset: self.demo_dynamic_buffer_offset,
         demo_uniform_buffer: self.demo_uniform_buffer.take().unwrap(),
      });
      let webgpu = self.webgpu.clone();
      self.loaded_demo.as_mut().unwrap()
            .start_switching_graphics_level(webgpu.as_ref(), self.graphics_level)
            .expect("WebGPU surface error");
   }
}

struct FractalRenderPipelines {
   default: Rc<wgpu::RenderPipeline>,
   antialiasing: Rc<wgpu::RenderPipeline>,
}
pub struct Demo {
   current_graphics_level: GraphicsLevel,
   render_pipelines: FractalRenderPipelines,
   use_antialiasing: bool,
   num_rendered_vertices: u32,
   pending_graphics_level_switch: Option<GraphicsSwitchingProcess>,
   pending_write_stable_uniform: bool,
   default_fractal_zoom: f32,
   uniform_groups: Vec<BindGroup>,
   fractal_uniform_data: FractalUniformData,
   fractal_uniform_buffer: UniformBuffer,
   fractal_buffer_offset: u64,
   demo_stable_uniform_data: DemoStableUniformData,
   demo_stable_buffer_offset: u64,
   demo_dynamic_uniform_data: DemoDynamicUniformData,
   demo_dynamic_buffer_offset: u64,
   demo_uniform_buffer: UniformBuffer,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct FractalUniformData {
   fractal_center: [f32; 2],
   fractal_zoom: f32,
   num_iterations: i32,
   color_bias: [f32; 3],
   color_power: f32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct DemoStableUniformData {
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
      const DEMO_LENGTH_SECONDS: f64 = 45.0;
      let zoom_scale = (-0.3*input.time_now_sec().rem(DEMO_LENGTH_SECONDS)).exp() as f32;
      if input.debug_mode() < Some(2) {
         self.fractal_uniform_data.fractal_zoom = self.default_fractal_zoom * zoom_scale;
      }
      self.demo_stable_uniform_data.color_attachment_size = input.screen_size().into();
      self.demo_stable_uniform_data.aspect_ratio = input.aspect_ratio();
      self.demo_stable_uniform_data.is_debug = input.debug_mode().map_or(0.0, f32::from);
      self.pending_write_stable_uniform = self.pending_write_stable_uniform || input.is_stable_updated();
      self.demo_dynamic_uniform_data.mouse_position = input.mouse_unit_position().into();
   }

   fn render(&mut self, webgpu: &Webgpu, backbuffer: &SurfaceTexture, _delta_sec: f64) -> Result<(), wgpu::SurfaceError> {
      {
         if self.pending_write_stable_uniform {
            self.demo_uniform_buffer.write(&webgpu.queue, self.demo_stable_buffer_offset, &[self.demo_stable_uniform_data]);
            self.pending_write_stable_uniform = false;
         }
         self.demo_uniform_buffer.write(&webgpu.queue, self.demo_dynamic_buffer_offset, &[self.demo_dynamic_uniform_data]);
         self.fractal_uniform_buffer.write(&webgpu.queue, self.fractal_buffer_offset, &[self.fractal_uniform_data]);
      }

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
            if self.use_antialiasing { &self.render_pipelines.antialiasing }
            else { &self.render_pipelines.default }
         );
         const DEMO_UNIFORM_BIND_GROUP_INDEX: u32 = 0;
         const FRACTAL_UNIFORM_BIND_GROUP_INDEX: u32 = 1;
         render_pass.set_bind_group(DEMO_UNIFORM_BIND_GROUP_INDEX, &self.uniform_groups[0].bind_group, &[]);
         render_pass.set_bind_group(FRACTAL_UNIFORM_BIND_GROUP_INDEX, &self.uniform_groups[1].bind_group, &[]);
         render_pass.draw(0..self.num_rendered_vertices, 0..1); // self.num_rendered>vertices
      }
   
      // submit will accept anything that implements IntoIter
      webgpu.queue.submit(std::iter::once(encoder.finish()));
      Ok(())
   }

   #[cfg(any(feature = "imgui_win", feature = "imgui_web"))]
   fn render_imgui(&mut self, ui: &imgui::Ui, args: super::imgui_web::ImguiRenderArgs) {
      use imgui::*;
      let window = ui.window("Fractal Demo");
      window
         .size(args.size, Condition::FirstUseEver)
         .position(args.position, Condition::FirstUseEver)
         .build(|| {
            imgui::Drag::new("Num iterations")
               .range(1, 2000)
               .speed(2.0)
               .build(ui,&mut self.fractal_uniform_data.num_iterations);
            let drag_speed = self.fractal_uniform_data.fractal_zoom * 0.05;
            imgui::Drag::new("Zoom")
               .range(1e-16, 2.0)
               .speed(drag_speed)
               .flags(SliderFlags::LOGARITHMIC)
               .build(ui,&mut self.fractal_uniform_data.fractal_zoom);
            imgui::Drag::new("Center")
               .range(-2.0, 2.0)
               .speed(drag_speed)
               .build_array(ui, &mut self.fractal_uniform_data.fractal_center);
            imgui::Drag::new("Color bias")
               .range(-5.0, 5.0)
               .speed(0.01)
               .build_array(ui, &mut self.fractal_uniform_data.color_bias);
            imgui::Drag::new("Color power")
               .range(-0.15, 1.0)
               .speed(0.005)
               .build(ui,&mut self.fractal_uniform_data.color_power);
            ui.checkbox("Antialiasing", &mut self.use_antialiasing);
            ui.separator();
         });
   }

   fn rebuild_pipelines(&mut self, webgpu: Rc<Webgpu>, color_target_format: wgpu::TextureFormat) {
      let mut loader = DemoLoadingProcess::new(
         webgpu.clone(), color_target_format, self.current_graphics_level);
      loader.rebuild_pipelines();
      self.render_pipelines = loader.render_pipelines.take().unwrap();
   }

   fn start_switching_graphics_level(&mut self, _webgpu: &Webgpu, graphics_level: GraphicsLevel) -> Result<(), wgpu::SurfaceError> {
      // TODO: fix this, the graphics level is not yet finished switching
      self.current_graphics_level = graphics_level;
      #[cfg(feature = "web")]
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

   fn drop_demo(&mut self, _webgpu: &Webgpu) {
      #[cfg(feature = "web")]
      web_sys::console::log_2(&"Rust demo drop custom".into(), &std::module_path!().into());
   }
}

impl Demo {
   pub fn start_loading(webgpu: Rc<Webgpu>, color_target_format: wgpu::TextureFormat, graphics_level: GraphicsLevel) -> Box<dyn DemoLoadingFuture> {
      Box::new(DemoLoadingProcess::new(webgpu, color_target_format, graphics_level))
   }

   pub fn make_command_buffers(&mut self) {
      // precache encoder.begin_render_pass into internal textures here
      // can't precache render into backbuffer, because it requires a ref to TextureView
   }
}

pub struct GraphicsSwitchingProcess {
   progress: f32,
   graphics_level: GraphicsLevel,
}

impl Dispose for GraphicsSwitchingProcess {
   fn dispose(&mut self) {
      #[cfg(feature = "web")]
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
        GraphicsLevel::Minimal => (50, false),
        GraphicsLevel::Low => (250, false),
        GraphicsLevel::Medium => (500, true),
        GraphicsLevel::High => (1000, true),
        GraphicsLevel::Ultra => (1500, true),
      };
      self_.progress = 1.0;
      demo.make_command_buffers();
      std::task::Poll::Ready(())
   }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
   fn shaders_compile() {
      let webgpu = futures::executor::block_on(Webgpu::new_offscreen());
      let mut demo_loader = DemoLoadingProcess::new(Rc::new(webgpu),
      wgpu::TextureFormat::Rgba8Unorm, GraphicsLevel::Medium);
      demo_loader.compile_shaders();
   }
}