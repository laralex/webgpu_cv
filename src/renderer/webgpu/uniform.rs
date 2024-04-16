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