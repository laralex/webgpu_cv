use crate::renderer::GlobalUniform;

use super::webgpu::Utils;

pub struct Samplers {
   pub bilinear_sampler: wgpu::Sampler,
   pub nearest_sampler: wgpu::Sampler,
}

pub struct Premade {
   pub samplers: Samplers,
   pub global_uniform: GlobalUniform,
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
      Self {
         samplers: Samplers::new(device),
         global_uniform: GlobalUniform::new(device),
      }
   }
}