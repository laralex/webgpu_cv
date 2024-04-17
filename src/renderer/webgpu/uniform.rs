pub struct BindGroup {
   pub bind_group: wgpu::BindGroup,
   pub bind_group_layout: wgpu::BindGroupLayout,
}

pub struct BindGroupBuilfer<'a> {
   layout_entries: Vec<wgpu::BindGroupLayoutEntry>,
   group_entries: Vec<wgpu::BindGroupEntry<'a>>,
}

impl BindGroup {
   pub fn builder<'a>() -> BindGroupBuilfer<'a> {
      BindGroupBuilfer {
         layout_entries: vec![],
         group_entries: vec![],
      }
   }
}

impl<'a> BindGroupBuilfer<'a> {
   pub fn with_uniform_buffer(mut self, binding: u32, visibility: wgpu::ShaderStages, buffer: &'a wgpu::Buffer) -> Self {
      let layout_entry = wgpu::BindGroupLayoutEntry {
         binding,
         visibility,
         ty: wgpu::BindingType::Buffer {
             ty: wgpu::BufferBindingType::Uniform,
             has_dynamic_offset: false,
             min_binding_size: None,
         },
         count: None,
      };
      let group_entry = wgpu::BindGroupEntry {
         binding,
         resource: buffer.as_entire_binding(),
      };
      self.layout_entries.push(layout_entry);
      self.group_entries.push(group_entry);
      self
   }

   pub fn with_uniform_buffer_range(mut self, binding: u32, visibility: wgpu::ShaderStages, buffer: &'a wgpu::Buffer, offset_size: (u64, u64)) -> Self {
      let layout_entry = wgpu::BindGroupLayoutEntry {
         binding,
         visibility,
         ty: wgpu::BindingType::Buffer {
             ty: wgpu::BufferBindingType::Uniform,
             has_dynamic_offset: false,
             min_binding_size: None,
         },
         count: None,
      };
      let group_entry = wgpu::BindGroupEntry {
         binding,
         resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding{
            buffer,
            offset: offset_size.0,
            size: std::num::NonZeroU64::new(offset_size.1),
         }),
      };
      self.layout_entries.push(layout_entry);
      self.group_entries.push(group_entry);
      self
   }

   pub fn build(self, device: &wgpu::Device, group_label: Option<&str>, layout_label: Option<&str>) -> BindGroup {
      let bind_group_layout = device.create_bind_group_layout(
         &wgpu::BindGroupLayoutDescriptor {
            label: layout_label,
            entries: &self.layout_entries,
         });
      let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
         label:group_label,
         layout: &bind_group_layout,
         entries: &self.group_entries,
      });
      BindGroup {
         bind_group_layout,
         bind_group,
      }
   }
}