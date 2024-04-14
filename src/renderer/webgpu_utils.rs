use wgpu::{BindGroupLayout, Device, Surface, SurfaceError, SurfaceTexture, TextureView};

pub struct WebgpuUtils;

impl WebgpuUtils {
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
      WebgpuUtils::make_shader(device, shader_code, "Vertex Shader")
   }

   pub fn make_fragment_shader(device: &Device, shader_code: &str) -> wgpu::ShaderModule {
      WebgpuUtils::make_shader(device,  shader_code, "Fragment Shader")
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
      WebgpuUtils::make_device_descriptor(wgpu::Features::PUSH_CONSTANTS)
   }

   pub fn downlevel_device_descriptor() -> wgpu::DeviceDescriptor<'static> {
      WebgpuUtils::make_device_descriptor(wgpu::Features::empty())
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

   pub fn make_compatible_push_constant<T: Sized>(device: &wgpu::Device, visibility: wgpu::ShaderStages, bind_group_index: u32) -> PushConstantsCompatibility {
      let num_bytes = std::mem::size_of::<T>() as u32;
      let mut required_limits = device.limits();
      required_limits.max_push_constant_size = num_bytes;
      if WebgpuUtils::supports_push_constants(&device, 0..num_bytes) {
         PushConstantsCompatibility::PushConstant(
            wgpu::PushConstantRange{stages: visibility, range: 0..num_bytes})
      } else {
         // fallback to uniforms
         PushConstantsCompatibility::Uniform(
            UniformGroup::new(&device, visibility, num_bytes as u64,),
            bind_group_index)
      }
   }

   pub fn bind_compatible_push_constant<'a, 'b: 'a>(render_pass: &'a mut wgpu::RenderPass<'b>, queue: &wgpu::Queue, uniform: &'b PushConstantsCompatibility, data: &[u8]) {
      match &uniform {
         PushConstantsCompatibility::Uniform(UniformGroup{buffer, bind_group, ..}, bind_group_index) => {
            queue.write_buffer(&buffer, 0, bytemuck::cast_slice(data));
            render_pass.set_bind_group(*bind_group_index, &bind_group, &[]);
         }
         PushConstantsCompatibility::PushConstant(range) => {
            render_pass.set_push_constants(range.stages, range.range.start, bytemuck::cast_slice(data));
         }
      }
   }
}

pub struct UniformGroup {
   pub buffer: wgpu::Buffer,
   pub bind_group: wgpu::BindGroup,
   pub bind_group_layout: wgpu::BindGroupLayout,
}

impl UniformGroup {
   pub fn new(device: &wgpu::Device, visibility: wgpu::ShaderStages, num_bytes: u64) -> Self {
      let buffer = device.create_buffer(
         &wgpu::BufferDescriptor {
             label: Some("uniform_buffer"),
             size: num_bytes,
             mapped_at_creation: false,
             usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
         }
      );
      let bind_group_layout = device.create_bind_group_layout(
         &wgpu::BindGroupLayoutDescriptor {
         label: Some("bind_layout"),
         entries: &[
             wgpu::BindGroupLayoutEntry {
                 binding: 0,
                 visibility,
                 ty: wgpu::BindingType::Buffer {
                     ty: wgpu::BufferBindingType::Uniform,
                     has_dynamic_offset: false,
                     min_binding_size: None,
                 },
                 count: None,
             }
         ],
      });
      let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
         label: Some("uniform_bind_group"),
         layout: &bind_group_layout,
         entries: &[
             wgpu::BindGroupEntry {
                 binding: 0,
                 resource: buffer.as_entire_binding(),
             }
         ],
      });
      Self {
         buffer,
         bind_group,
         bind_group_layout,
      }
   }
}

pub enum PushConstantsCompatibility {
   Uniform(UniformGroup, u32),
   PushConstant(wgpu::PushConstantRange),
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

   pub fn with(self, uniform_or_pushconstant: &'a PushConstantsCompatibility) -> Self {
      match uniform_or_pushconstant {
         PushConstantsCompatibility::Uniform(group, _) => self.with_uniform_group(group),
         PushConstantsCompatibility::PushConstant(range) => self.with_push_constant(range.clone()),
      }
   }

   pub fn with_uniform_group(mut self, uniform_group: &'a UniformGroup) -> Self {
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