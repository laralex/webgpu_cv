use std::{collections::HashMap, hash::{Hash, Hasher, BuildHasher}, rc::Rc};

use super::{preprocessor::Preprocessor, webgpu::utils::Utils};

#[allow(unused)]
#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub enum VertexShaderVariant {
   TriangleFullscreen,
   TriangleColored,
}

#[allow(unused)]
#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub enum FragmentShaderVariant {
   VertexColor,
   FractalMandelbrot,
   Uv,
}

pub struct ShaderLoader {
   loaded_vertex_shaders: HashMap<u64, Rc<wgpu::ShaderModule>>,
   loaded_fragment_shaders: HashMap<u64, Rc<wgpu::ShaderModule>>,
   use_cache: bool,
}

impl ShaderLoader {
   pub fn new(use_cache: bool) -> Self {
      Self {
         use_cache,
         loaded_vertex_shaders: Default::default(),
         loaded_fragment_shaders: Default::default(),
      }
   }

   pub fn get_vertex_shader(&mut self, device: &wgpu::Device, variant: VertexShaderVariant, preprocessor: Option<&mut Preprocessor>) -> Rc<wgpu::ShaderModule> {
      let mut hash = 0;
      if self.use_cache {
         let mut hasher = self.loaded_vertex_shaders.hasher().build_hasher();
         variant.hash(&mut hasher);
         preprocessor.as_ref().hash(&mut hasher);
         hash = hasher.finish();
         if let Some(shader) = self.loaded_vertex_shaders.get(&hash) {
            #[cfg(feature = "web")]
            web_sys::console::log_2(&"Vertex shader cache hit".into(), &(variant as usize).into());
            return shader.clone()
         }
         #[cfg(feature = "web")]
         web_sys::console::log_2(&"Vertex shader cache MISS".into(), &(variant as usize).into());
      }
      let shader = Rc::new(Self::make_vertex_shader(device, variant, preprocessor));
      self.loaded_vertex_shaders.insert(hash, shader.clone());
      shader
   }
   
   pub fn get_fragment_shader(&mut self, device: &wgpu::Device, variant: FragmentShaderVariant, preprocessor: Option<&mut Preprocessor>) -> Rc<wgpu::ShaderModule> {
      let mut hash = 0;
      if self.use_cache {
         let mut hasher = self.loaded_vertex_shaders.hasher().build_hasher();
         variant.hash(&mut hasher);
         preprocessor.as_ref().hash(&mut hasher);
         hash = hasher.finish();
         if let Some(shader) = self.loaded_fragment_shaders.get(&hash) {
            #[cfg(feature = "web")]
            web_sys::console::log_2(&"Fragment shader cache hit".into(), &(variant as usize).into());
            return shader.clone()
         }
         #[cfg(feature = "web")]
         web_sys::console::log_2(&"Fragment shader cache MISS".into(), &(variant as usize).into());
      }
      let shader = Rc::new(Self::make_fragment_shader(device, variant, preprocessor));
      self.loaded_fragment_shaders.insert(hash, shader.clone());
      shader
   }

   fn make_vertex_shader(device: &wgpu::Device, variant: VertexShaderVariant, preprocessor: Option<&mut Preprocessor>) -> wgpu::ShaderModule {
      use VertexShaderVariant::*;
      let (source_code, label) = match variant {
         TriangleFullscreen =>
            (include_str!("shaders/triangle_fullscreen.vs.wgsl"), "triangle_fullscreen.vs.wgsl"),
         TriangleColored =>
            (include_str!("shaders/triangle_colored.vs.wgsl")   , "triangle_colored.vs.wgsl"),
      };
      let source_code = match preprocessor {
         Some(preprocessor) => preprocessor.process(source_code)
            .expect("Failed to run preprocessor on vertex shader"),
         _ => source_code.to_owned(),
      };
      Utils::make_shader(device, &source_code, label)
   }

   fn make_fragment_shader(device: &wgpu::Device, variant: FragmentShaderVariant, preprocessor: Option<&mut Preprocessor>) -> wgpu::ShaderModule {
      use FragmentShaderVariant::*;
      let (source_code, label) = match variant {
         VertexColor =>
            (include_str!("shaders/vertex_color.fs.wgsl"), "vertex_color.fs.wgsl"),
         FractalMandelbrot =>
            (include_str!("shaders/mandelbrot.fs.wgsl"), "mandelbrot.fs.wgsl"),
         Uv =>
            (include_str!("shaders/uv.fs.wgsl"), "uv.fs.wgsl"),
      };
      let source_code = match preprocessor {
         Some(preprocessor) => preprocessor.process(source_code)
            .expect("Failed to run preprocessor on fragment shader"),
         _ => source_code.to_owned(),
      };
      Utils::make_shader(device, &source_code, label)
   }
}
