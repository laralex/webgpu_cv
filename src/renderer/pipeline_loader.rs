use std::{collections::HashMap, hash::{Hash, BuildHasher, Hasher}, rc::Rc};

#[derive(Hash)]
pub struct RenderPipelineFlatDescriptor<'a> {
   // pub bind_group_layout_descriptors: &'a Vec<BindGroupLayoutDescriptor<'a>>,
   pipeline_layout_descriptor: PipelineLayoutDescriptor<'a>,
   pipeline_descriptor: RenderPipelineDescriptor<'a>,
}

impl<'a> RenderPipelineFlatDescriptor<'a> {
   pub fn new(
      // bind_group_layout_descriptors: &'a Vec<BindGroupLayoutDescriptor<'a>>,
      pipeline_layout_descriptor: &'a wgpu::PipelineLayoutDescriptor<'a>,
      pipeline_descriptor: &'a wgpu::RenderPipelineDescriptor<'a>,
   ) -> Self {
      Self {
         // bind_group_layout_descriptors,
         pipeline_layout_descriptor: PipelineLayoutDescriptor(pipeline_layout_descriptor),
         pipeline_descriptor: RenderPipelineDescriptor(pipeline_descriptor),
      }
   }
}

pub struct PipelineLoader {
   cache: HashMap<u64, Rc<wgpu::RenderPipeline>>,
   use_cache: bool,
}

impl PipelineLoader {
   pub fn new(use_cache: bool) -> Self {
      Self {
         use_cache,
         cache: Default::default(),
      }
   }

   pub fn get_pipeline<'a>(&mut self, device: &wgpu::Device, flat_descriptor: &'a RenderPipelineFlatDescriptor) -> Rc<wgpu::RenderPipeline> {
      let mut hash = 0;
      if self.use_cache {
         let mut hasher = self.cache.hasher().build_hasher();
         flat_descriptor.hash(&mut hasher);
         hash = hasher.finish();
         if let Some(pipeline) = self.cache.get(&hash) {
            #[cfg(feature = "web")]
            web_sys::console::log_1(&"Pipeline cache hit".into());
            return pipeline.clone()
         }
         #[cfg(feature = "web")]
         web_sys::console::log_1(&"Pipeline cache MISS".into());
      }
      let pipeline = Rc::new(device.create_render_pipeline(&flat_descriptor.pipeline_descriptor.0));
      self.cache.insert(hash, pipeline.clone());
      pipeline
   }
}

struct RenderPipelineDescriptor<'a>(&'a wgpu::RenderPipelineDescriptor<'a>);

impl<'a> Hash for RenderPipelineDescriptor<'a> {
   fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
      // TODO: doesn't consider layout of bind groups and push constants
      // self.0.layout.map(|layout| layout.global_id().hash(state)); // ?
      self.0.vertex.module.global_id().hash(state); // ?
      self.0.vertex.entry_point.hash(state);
      for (i, buffer) in self.0.vertex.buffers.iter().enumerate() {
         i.hash(state);
         buffer.hash(state);
      }
      if let Some(fragment) = self.0.fragment.as_ref() {
         fragment.entry_point.hash(state);
         fragment.module.global_id().hash(state); // ?
         for (i, target_state) in fragment.targets.iter().enumerate() {
            i.hash(state);
            target_state.hash(state);
         }
      }
      self.0.primitive.hash(state);
      self.0.depth_stencil.hash(state);
      self.0.multisample.hash(state);
      self.0.multiview.hash(state);
   }
}

struct PipelineLayoutDescriptor<'a>(&'a wgpu::PipelineLayoutDescriptor<'a>);

impl<'a> Hash for PipelineLayoutDescriptor<'a> {
   fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
      for (i, &_group) in self.0.bind_group_layouts.iter().enumerate() {
         i.hash(state);
         // group.global_id().hash(state);
      }
      for (i, push_range) in self.0.push_constant_ranges.iter().enumerate() {
         i.hash(state);
         push_range.hash(state);
      }
   }
}

struct BindGroupLayoutDescriptor<'a>(&'a wgpu::BindGroupLayoutDescriptor<'a>);

impl<'a> Hash for BindGroupLayoutDescriptor<'a> {
   fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
      for (i, entry) in self.0.entries.iter().enumerate() {
         i.hash(state);
         entry.hash(state);
      }
   }
}