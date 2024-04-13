use super::Webgpu;

pub struct WebgpuUtils {

}

impl WebgpuUtils {
   pub fn make_shader(webgpu: &Webgpu, shader_code: &str, label: &str) -> wgpu::ShaderModule {
      webgpu.device.create_shader_module(wgpu::ShaderModuleDescriptor {
         label: Some(label),
         source: wgpu::ShaderSource::Wgsl(shader_code.into()),
      })
   }

   pub fn make_vertex_shader(webgpu: &Webgpu, shader_code: &str) -> wgpu::ShaderModule {
      WebgpuUtils::make_shader(webgpu, shader_code, "Vertex Shader")
   }

   pub fn make_fragment_shader(webgpu: &Webgpu, shader_code: &str) -> wgpu::ShaderModule {
      WebgpuUtils::make_shader(webgpu,  shader_code, "Fragment Shader")
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