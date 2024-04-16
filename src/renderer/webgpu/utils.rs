use wgpu::{BindGroupLayout, Device, Surface, SurfaceError, SurfaceTexture, TextureView};

use super::uniform::{PushConstantsCompatibility, BindGroup};

pub struct Utils;

impl Utils {
   pub fn surface_view(surface_texture: &SurfaceTexture) -> wgpu::TextureView {
      surface_texture.texture.create_view(
         &wgpu::TextureViewDescriptor::default())
   }

   pub fn make_shader(device: &Device, shader_code: &str, label: &str) -> wgpu::ShaderModule {
      device.create_shader_module(wgpu::ShaderModuleDescriptor {
         label: Some(label),
         source: wgpu::ShaderSource::Wgsl(shader_code.into()),
      })
   }

   pub fn make_vertex_shader(device: &Device, shader_code: &str) -> wgpu::ShaderModule {
      Utils::make_shader(device, shader_code, "Vertex Shader")
   }

   pub fn make_fragment_shader(device: &Device, shader_code: &str) -> wgpu::ShaderModule {
      Utils::make_shader(device,  shader_code, "Fragment Shader")
   }

   pub fn default_primitive_state() -> wgpu::PrimitiveState {
      wgpu::PrimitiveState {
         topology: wgpu::PrimitiveTopology::TriangleList,
         strip_index_format: None,
         front_face: wgpu::FrontFace::Ccw,
         cull_mode: Some(wgpu::Face::Back),
         // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
         polygon_mode: wgpu::PolygonMode::Fill,
         // Requires Features::DEPTH_CLIP_CONTROL
         unclipped_depth: false,
         // Requires Features::CONSERVATIVE_RASTERIZATION
         conservative: false,
     }
   }

   pub fn default_device_descriptor() -> wgpu::DeviceDescriptor<'static> {
      Utils::make_device_descriptor(wgpu::Features::PUSH_CONSTANTS)
   }

   pub fn downlevel_device_descriptor() -> wgpu::DeviceDescriptor<'static> {
      Utils::make_device_descriptor(wgpu::Features::empty())
   }

   pub fn make_device_descriptor(features: wgpu::Features) -> wgpu::DeviceDescriptor<'static> {
      wgpu::DeviceDescriptor {
         required_features: features,
         required_limits: if cfg!(target_arch = "wasm32") {
               wgpu::Limits::downlevel_webgl2_defaults()
         } else {
               wgpu::Limits::default()
         },
         label: None,
      }
   }

   pub fn supports_push_constants(device: &wgpu::Device, required_range: std::ops::Range<u32>) -> bool {
      device.features().contains(wgpu::Features::PUSH_CONSTANTS)
         && device.limits().max_push_constant_size > required_range.end
         && false // NOTE: currently WGSL doesn't support push_constants in shaders, so forcing disable to not accidentally use push constants
   }

   // pub fn make_uniform<T: Sized>(device: &wgpu::Device, visibility: wgpu::ShaderStages) -> BindGroup {
   //    let num_bytes = std::mem::size_of::<T>() as u32;
   //    let mut required_limits = device.limits();
   //    required_limits.max_push_constant_size = num_bytes;
   //    BindGroup::new(&device, visibility, num_bytes as u64)
   // }

   // pub fn bind_group<'a, 'b: 'a>(render_pass: &'a mut wgpu::RenderPass<'b>, queue: &wgpu::Queue, uniform: &'b BindGroup, data: &[u8], bind_group_index: u32) {
   //    let BindGroup{buffers: buffer, bind_group, ..} = uniform;
   //    queue.write_buffer(&buffer, 0, bytemuck::cast_slice(data));
   //    render_pass.set_bind_group(bind_group_index, &bind_group, &[]);
   // }
}

pub struct PipelineLayoutBuilder<'a> {
   uniform_group_layouts: Vec<&'a wgpu::BindGroupLayout>,
   push_constant_ranges: Vec<wgpu::PushConstantRange>,
}

impl<'a> PipelineLayoutBuilder<'a> {
   pub fn new() -> Self {
      Self {
         uniform_group_layouts: vec![],
         push_constant_ranges: vec![],
      }
   }

   pub fn with(self, uniform_group: &'a BindGroup) -> Self {
      self.with_uniform_group(uniform_group)
   }

   pub fn with_uniform_group(mut self, uniform_group: &'a BindGroup) -> Self {
      self.uniform_group_layouts.push(&uniform_group.bind_group_layout);
      self
   }

   pub fn with_push_constant(mut self, range: wgpu::PushConstantRange) -> Self {
      self.push_constant_ranges.push(range);
      self
   }

   pub fn build(self, device: &wgpu::Device, label: Option<&str>) -> wgpu::PipelineLayout {
      // let bind_group_layouts: Vec<&wgpu::BindGroupLayout> = self.uniform_group_layouts.iter()
      //    .map(|u| u)
      //    .collect::<Vec<_>>();
      device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
         label,
         bind_group_layouts: &self.uniform_group_layouts,
         push_constant_ranges: &self.push_constant_ranges,
      })
   }
}