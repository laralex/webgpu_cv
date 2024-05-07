use wgpu::{Device, SurfaceTexture};

use super::uniform::BindGroupInfo;

pub struct Utils;

impl Utils {
   pub fn default_renderpass<'a>(encoder: &'a mut wgpu::CommandEncoder, color: &'a Option<wgpu::TextureView>, depth: &'a Option<wgpu::TextureView>) -> wgpu::RenderPass<'a> {
      encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
         label: Some("Render Pass"),
         color_attachments: &[color.as_ref().map(|c| wgpu::RenderPassColorAttachment {
            view: c,
            resolve_target: None,
            ops: wgpu::Operations {
               load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
               store: wgpu::StoreOp::Store,
            },
         })],
         depth_stencil_attachment: depth.as_ref().map(|d| wgpu::RenderPassDepthStencilAttachment {
            view: d,
            depth_ops: Some(wgpu::Operations {
               load: wgpu::LoadOp::Clear(1.0),
               store: wgpu::StoreOp::Discard,
            }),
            stencil_ops: Some(wgpu::Operations {
               load: wgpu::LoadOp::Clear(0),
               store: wgpu::StoreOp::Discard,
            }),
         }),
         occlusion_query_set: None,
         timestamp_writes: None,
      })
   }
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

   #[allow(unused)]
   pub fn make_vertex_shader(device: &Device, shader_code: &str) -> wgpu::ShaderModule {
      Utils::make_shader(device, shader_code, "Vertex Shader")
   }
   
   #[allow(unused)]
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

   pub fn bilinear_sampler() -> wgpu::SamplerDescriptor<'static> {
      wgpu::SamplerDescriptor {
         label: Some("Sampler Bilinear"),
         address_mode_u: wgpu::AddressMode::ClampToEdge,
         address_mode_v: wgpu::AddressMode::ClampToEdge,
         address_mode_w: wgpu::AddressMode::ClampToEdge,
         mag_filter: wgpu::FilterMode::Linear,
         min_filter: wgpu::FilterMode::Nearest,
         mipmap_filter: wgpu::FilterMode::Nearest,
         lod_min_clamp: 0.0,
         lod_max_clamp: 32.0,
         compare: None,
         anisotropy_clamp: 1,
         border_color: None,
      }
   }

   pub fn nearest_sampler() -> wgpu::SamplerDescriptor<'static> {
      wgpu::SamplerDescriptor {
         label: Some("Sampler Nearest"),
         address_mode_u: wgpu::AddressMode::ClampToEdge,
         address_mode_v: wgpu::AddressMode::ClampToEdge,
         address_mode_w: wgpu::AddressMode::ClampToEdge,
         mag_filter: wgpu::FilterMode::Nearest,
         min_filter: wgpu::FilterMode::Nearest,
         mipmap_filter: wgpu::FilterMode::Nearest,
         lod_min_clamp: 0.0,
         lod_max_clamp: 32.0,
         compare: None,
         anisotropy_clamp: 1,
         border_color: None,
      }
   }

   pub fn default_device_descriptor() -> wgpu::DeviceDescriptor<'static> {
      Utils::make_device_descriptor(wgpu::Features::PUSH_CONSTANTS)
   }

   #[allow(unused)]
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

   #[allow(unused)]
   pub fn supports_push_constants(device: &wgpu::Device, required_range: std::ops::Range<u32>) -> bool {
      device.features().contains(wgpu::Features::PUSH_CONSTANTS)
         && device.limits().max_push_constant_size > required_range.end
         && false // NOTE: currently WGSL doesn't support push_constants in shaders, so forcing disable to not accidentally use push constants
   }

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

   pub fn from_uniform_iter(iter: impl Iterator<Item=&'a BindGroupInfo>) -> Self {
      let mut s = Self::new();
      for uniform_group in iter {
         s = s.with(uniform_group);
      }
      s
   }

   pub fn with(self, uniform_group: &'a BindGroupInfo) -> Self {
      self.with_uniform_group(uniform_group)
   }

   pub fn with_uniform_group(mut self, uniform_group: &'a BindGroupInfo) -> Self {
      self.uniform_group_layouts.push(&uniform_group.bind_group_layout);
      self
   }

   #[allow(unused)]
   pub fn with_push_constant(mut self, range: wgpu::PushConstantRange) -> Self {
      self.push_constant_ranges.push(range);
      self
   }

   pub fn build_descriptor(&'a self, label: Option<&'a str>) -> wgpu::PipelineLayoutDescriptor {
      wgpu::PipelineLayoutDescriptor {
         label,
         bind_group_layouts: &self.uniform_group_layouts,
         push_constant_ranges: &self.push_constant_ranges,
      }
   }

   #[allow(unused)]
   pub fn build(&'a self, device: &wgpu::Device, label: Option<&'a str>) -> wgpu::PipelineLayout {
      // let bind_group_layouts: Vec<&wgpu::BindGroupLayout> = self.uniform_group_layouts.iter()
      //    .map(|u| u)
      //    .collect::<Vec<_>>();
      let layout_descriptor = wgpu::PipelineLayoutDescriptor {
         label,
         bind_group_layouts: &self.uniform_group_layouts,
         push_constant_ranges: &self.push_constant_ranges,
      };
      device.create_pipeline_layout(&layout_descriptor)
   }
}