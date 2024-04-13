use wgpu::{Device, Surface, SurfaceError, SurfaceTexture, TextureView};

use super::Webgpu;

pub struct WebgpuUtils {

}

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
         front_face: wgpu::FrontFace::Cw,
         cull_mode: Some(wgpu::Face::Back),
         // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
         polygon_mode: wgpu::PolygonMode::Fill,
         // Requires Features::DEPTH_CLIP_CONTROL
         unclipped_depth: false,
         // Requires Features::CONSERVATIVE_RASTERIZATION
         conservative: false,
     }
   }
}