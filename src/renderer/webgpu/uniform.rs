use crate::timer::ScopedTimer;

pub struct BindGroupInfo {
   pub bind_group: wgpu::BindGroup,
   pub bind_group_layout: wgpu::BindGroupLayout,
   pub bind_group_layout_entries: Vec<wgpu::BindGroupLayoutEntry>,
}

pub struct BindGroupBuilfer<'a> {
   layout_entries: Vec<wgpu::BindGroupLayoutEntry>,
   group_entries: Vec<wgpu::BindGroupEntry<'a>>,
}

impl BindGroupInfo {
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

   pub fn with_texture_2d(self, binding: u32, visibility: wgpu::ShaderStages, sample_type: wgpu::TextureSampleType, view: &'a wgpu::TextureView) -> Self {
      self.with_texture(binding, visibility,
         wgpu::TextureViewDimension::D2, sample_type,
         /*multisampled*/ false, view)
   }

   pub fn with_texture_cube(self, binding: u32, visibility: wgpu::ShaderStages, sample_type: wgpu::TextureSampleType, view: &'a wgpu::TextureView) -> Self {
      self.with_texture(binding, visibility,
         wgpu::TextureViewDimension::Cube, sample_type,
         /*multisampled*/ false, view)
   }

   pub fn with_texture(mut self, binding: u32, visibility: wgpu::ShaderStages, view_dimension: wgpu::TextureViewDimension, sample_type: wgpu::TextureSampleType, multisampled: bool, view: &'a wgpu::TextureView) -> Self {
      let layout_entry = wgpu::BindGroupLayoutEntry {
         binding,
         visibility,
         ty: wgpu::BindingType::Texture {
            view_dimension,
            multisampled, // TODO: pass
            sample_type,
         },
         count: None,
      };
      let group_entry = wgpu::BindGroupEntry {
         binding,
         resource: wgpu::BindingResource::TextureView(&view),
      };
      self.layout_entries.push(layout_entry);
      self.group_entries.push(group_entry);
      self
   }

   pub fn with_sampler(mut self, binding: u32, visibility: wgpu::ShaderStages, sampler: &'a wgpu::Sampler) -> Self {
      let layout_entry = wgpu::BindGroupLayoutEntry {
         binding,
         visibility,
         ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
         count: None,
      };
      let group_entry = wgpu::BindGroupEntry {
         binding,
         resource: wgpu::BindingResource::Sampler(&sampler),
      };
      self.layout_entries.push(layout_entry);
      self.group_entries.push(group_entry);
      self
   }

   pub fn with_entries(mut self, layout_entry: wgpu::BindGroupLayoutEntry, entry: wgpu::BindGroupEntry<'a>) -> Self {
      self.layout_entries.push(layout_entry);
      self.group_entries.push(entry);
      self
   }

   pub fn build(self, device: &wgpu::Device, group_label: Option<&str>, layout_label: Option<&str>) -> BindGroupInfo {
      let _t = ScopedTimer::new("uniform_group::build");
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
      BindGroupInfo {
         bind_group_layout_entries: self.layout_entries,
         bind_group_layout,
         bind_group,
      }
   }
}