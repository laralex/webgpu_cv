use std::rc::Rc;
use wgpu::{util::DeviceExt, SurfaceTexture};
use bytemuck;

use crate::renderer::webgpu_utils::WebgpuUtils;
use crate::GraphicsLevel;

use super::{DemoLoadingFuture, Dispose, ExternalState, IDemo, Progress, SimpleFuture, Webgpu};

const VERTEX_SHADER_SRC:   &str = include_str!("shaders/triangle_fullscreen.vs.wgsl");
const FRAGMENT_SHADER_SRC: &str = include_str!("shaders/mandelbrot.fs.wgsl");

#[derive(Default)]
enum DemoLoadingStage {
   Ready = 0,
   #[default] CompileShaders,
   LinkPrograms,
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
   uniform_buffer: Option<wgpu::Buffer>,
   uniform_bind_group: Option<wgpu::BindGroup>,
   uniform_bind_group_layout: Option<wgpu::BindGroupLayout>,
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
            self.uniform_bind_group.take();
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
            let device = &self.webgpu.as_ref().device;
            let vertex_shader = WebgpuUtils::make_vertex_shader(device, VERTEX_SHADER_SRC);
            let fragment_shader = WebgpuUtils::make_fragment_shader(device, FRAGMENT_SHADER_SRC);
            std::mem::drop(device);
            self.vertex_shader = Some(vertex_shader);
            self.fragment_shader = Some(fragment_shader);
            self.stage_percent += 0.1;
            self.stage = CreateUniforms;
            std::task::Poll::Pending
         },
         CreateUniforms => {
            let uniform_buffer = self.webgpu.device.create_buffer(
               &wgpu::BufferDescriptor {
                   label: Some("uniform_buffer"),
                   size: std::mem::size_of::<UniformData>() as u64,
                   mapped_at_creation: false,
                   usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
               }
            );
            self.uniform_bind_group_layout = Some(self.webgpu.device.create_bind_group_layout(
               &wgpu::BindGroupLayoutDescriptor {
               label: Some("bind_layout"),
               entries: &[
                   wgpu::BindGroupLayoutEntry {
                       binding: 0,
                       visibility: wgpu::ShaderStages::FRAGMENT,
                       ty: wgpu::BindingType::Buffer {
                           ty: wgpu::BufferBindingType::Uniform,
                           has_dynamic_offset: false,
                           min_binding_size: None,
                       },
                       count: None,
                   }
               ],
            }));
            self.uniform_bind_group = Some(self.webgpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
               label: Some("uniform_bind_group"),
               layout: &self.uniform_bind_group_layout.as_ref().unwrap(),
               entries: &[
                   wgpu::BindGroupEntry {
                       binding: 0,
                       resource: uniform_buffer.as_entire_binding(),
                   }
               ],
            }));
            self.uniform_buffer = Some(uniform_buffer);
            self.stage_percent += 0.1;
            self.stage = LinkPrograms;
            std::task::Poll::Pending
         }
         LinkPrograms => {
            let render_pipeline_layout =
               self.webgpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                  label: Some("Render Pipeline Layout"),
                  bind_group_layouts: &[self.uniform_bind_group_layout.as_ref().unwrap()],
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
            self.stage_percent += 0.1;
            self.stage = StartSwitchingGraphicsLevel;
            std::task::Poll::Pending
         }
         StartSwitchingGraphicsLevel => {
            self.loaded_demo = Some(Demo {
               render_pipeline: self.render_pipeline.take().unwrap(),
               num_rendered_vertices: 3,
               pending_graphics_level_switch: None,
               uniform_data: UniformData {
                  fractal_center_zoom: [-1.1900443, 0.3043895, 1e-2],
                  _padding: 0.0,
               },
               uniform_buffer: self.uniform_buffer.take().unwrap(),
               uniform_bind_group: self.uniform_bind_group.take().unwrap(),
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

pub struct Demo {
   render_pipeline: wgpu::RenderPipeline,
   num_rendered_vertices: i32,
   pending_graphics_level_switch: Option<GraphicsSwitchingProcess>,
   uniform_data: UniformData,
   uniform_buffer: wgpu::Buffer,
   uniform_bind_group: wgpu::BindGroup,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct UniformData {
   fractal_center_zoom: [f32; 3],
   _padding: f32,
}

impl IDemo for Demo {
   fn tick(&mut self, input: &ExternalState) {
      self.uniform_data.fractal_center_zoom[2] -= self.uniform_data.fractal_center_zoom[2] * 0.1 * input.time_delta_sec;
   }

   fn render(&mut self, webgpu: &Webgpu, backbuffer: &SurfaceTexture, _delta_sec: f32) -> Result<(), wgpu::SurfaceError> {
      webgpu.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[self.uniform_data]));
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
         render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
         render_pass.draw(0..3, 0..1); // self.num_rendered>vertices
      }
   
      // submit will accept anything that implements IntoIter
      webgpu.queue.submit(std::iter::once(encoder.finish()));
      Ok(())
   }

   fn start_switching_graphics_level(&mut self, webgpu: &Webgpu, graphics_level: GraphicsLevel) -> Result<(), wgpu::SurfaceError> {
      web_sys::console::log_2(&"Rust start_switching_graphics_level".into(), &std::module_path!().into());
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
         uniform_buffer: Default::default(),
         uniform_bind_group: Default::default(),
         uniform_bind_group_layout: Default::default(),
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