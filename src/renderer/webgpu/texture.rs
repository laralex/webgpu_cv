pub struct TextureBuilder<'a> {
   size: wgpu::Extent3d,
   mip_level_count: u32,
   sample_count: u32,
   dimension: wgpu::TextureDimension,
   format: wgpu::TextureFormat,
   usage: wgpu::TextureUsages,
   label: Option<&'a str>,
   view_formats: Vec<wgpu::TextureFormat>,
}

impl<'a> TextureBuilder<'a> {
   pub fn new_2d(size: wgpu::Extent3d, format: wgpu::TextureFormat)-> TextureBuilder<'a> {
      Self {
         size,
         mip_level_count: 1,
         sample_count: 1,
         dimension: wgpu::TextureDimension::D2,
         format,
         usage: wgpu::TextureUsages::empty(),
         label: None,
         view_formats: vec![],
      }
   }

   pub fn with_mip_level_count(mut self, mip_level_count: u32) -> Self {
      self.mip_level_count = mip_level_count;
      self
   }

   pub fn with_sample_count(mut self, sample_count: u32) -> Self {
      self.sample_count = sample_count;
      self
   }

   pub fn with_dimension(mut self, dimension: wgpu::TextureDimension) -> Self {
      self.dimension = dimension;
      self
   }

   pub fn add_usage(mut self, usage: wgpu::TextureUsages) -> Self {
      self.usage |= usage;
      self
   }

   pub fn remove_usage(mut self, usage: wgpu::TextureUsages) -> Self {
      self.usage &= !usage;
      self
   }

   pub fn with_label(mut self, label: Option<&'a str>) -> Self {
      self.label = label;
      self
   }

   pub fn add_view_format(mut self, view_format: wgpu::TextureFormat) -> Self {
      self.view_formats.push(view_format);
      self
   }

   pub fn with_size(mut self, size: wgpu::Extent3d) -> Self {
      self.size = size;
      self
   }

   pub fn with_format(mut self, format: wgpu::TextureFormat) -> Self {
      self.format = format;
      self
   }

   pub fn build(self, device: &wgpu::Device) -> wgpu::Texture {
      device.create_texture(&wgpu::TextureDescriptor {
         size: self.size,
         mip_level_count: self.mip_level_count,
         sample_count: self.sample_count,
         dimension: self.dimension,
         format: self.format,
         usage: self.usage,
         label: self.label,
         view_formats: &self.view_formats,
      })
   }
}
