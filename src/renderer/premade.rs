use crate::renderer::GlobalUniform;
use super::{pipeline_loader::PipelineLoader, shader_loader::ShaderLoader, webgpu::Utils};

#[cfg(feature = "web")]
const USE_SHADER_CACHE: bool = true;
#[cfg(not(feature = "web"))]
const USE_SHADER_CACHE: bool = false;
const USE_PIPELINE_CACHE: bool = true;

pub struct Samplers {
   pub bilinear_sampler: wgpu::Sampler,
   pub nearest_sampler: wgpu::Sampler,
}

pub struct Premade {
   pub samplers: Samplers,
   pub global_uniform: GlobalUniform,
   pub shader_loader: ShaderLoader,
   pub pipeline_loader: PipelineLoader,
}

impl Samplers {
   pub fn new(device: &wgpu::Device) -> Self {
      Self {
         bilinear_sampler: device.create_sampler(&Utils::bilinear_sampler()),
         nearest_sampler: device.create_sampler(&Utils::nearest_sampler()),
      }
   }
}

impl Premade {
   pub fn new(device: &wgpu::Device) -> Self {
      let shader_loader = ShaderLoader::new(USE_SHADER_CACHE);
      let pipeline_loader = PipelineLoader::new(USE_PIPELINE_CACHE);
      Self {
         samplers: Samplers::new(device),
         global_uniform: GlobalUniform::new(device),
         shader_loader,
         pipeline_loader,
      }
   }
}