use super::Utils;

pub struct Premade {
   pub bilinear_sampler: wgpu::Sampler,
   pub nearest_sampler: wgpu::Sampler,
}

impl Premade {
   pub fn new(device: &wgpu::Device) -> Self {
      Self {
         bilinear_sampler: device.create_sampler(&Utils::bilinear_sampler()),
         nearest_sampler: device.create_sampler(&Utils::nearest_sampler()),
      }
   }
}