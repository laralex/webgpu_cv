use std::{any, collections::HashMap, hash::{BuildHasher, Hash, Hasher}, path::Path, rc::Rc};

use super::{preprocessor::Preprocessor, webgpu::utils::Utils};

#[allow(unused)]
#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub enum VertexShaderVariant {
   TriangleFullscreen = 0,
   TriangleColored = 1,
}

// shader enum -> source code during compilation
impl AsRef<str> for VertexShaderVariant {
   fn as_ref(&self) -> &str {
      use VertexShaderVariant::*;
      match self {
         TriangleFullscreen => include_str!("shaders/triangle_fullscreen.vs.wgsl"),
         TriangleColored => include_str!("shaders/triangle_colored.vs.wgsl"),
      }
   }
}

// shader enum -> filesystem path
impl AsRef<std::path::Path> for VertexShaderVariant {
    fn as_ref(&self) -> &std::path::Path {
      use VertexShaderVariant::*;
      match self {
         TriangleFullscreen => "shaders/triangle_fullscreen.vs.wgsl".as_ref(),
         TriangleColored => "shaders/triangle_colored.vs.wgsl".as_ref(),
      }
    }
}

#[allow(unused)]
#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub enum FragmentShaderVariant {
   VertexColor = 0,
   FractalMandelbrot = 1,
   Uv = 2,
}

// shader enum -> source code during compilation
impl AsRef<str> for FragmentShaderVariant {
   fn as_ref(&self) -> &str {
      use FragmentShaderVariant::*;
      match self {
         VertexColor => include_str!("shaders/vertex_color.fs.wgsl"),
         FractalMandelbrot => include_str!("shaders/mandelbrot.fs.wgsl"),
         Uv => include_str!("shaders/uv.fs.wgsl"),
      }
   }
}

// shader enum -> filesystem path
impl AsRef<std::path::Path> for FragmentShaderVariant {
    fn as_ref(&self) -> &std::path::Path {
      use FragmentShaderVariant::*;
      match self {
         VertexColor => "shaders/vertex_color.fs.wgsl".as_ref(),
         FractalMandelbrot => "shaders/mandelbrot.fs.wgsl".as_ref(),
         Uv => "shaders/uv.fs.wgsl".as_ref(),
      }
    }
}

pub struct ShaderLoader {
   // loaded_vertex_shaders: HashMap<u64, Rc<wgpu::ShaderModule>>,
   // loaded_fragment_shaders: HashMap<u64, Rc<wgpu::ShaderModule>>,
   loaded_shaders: HashMap<u64, Rc<wgpu::ShaderModule>>,
   use_cache: bool,
}

impl ShaderLoader {
   pub fn new(use_cache: bool) -> Self {
      Self {
         use_cache,
         loaded_shaders: Default::default(),
         // loaded_vertex_shaders: Default::default(),
         // loaded_fragment_shaders: Default::default(),
      }
   }

   pub fn get_shader<T>(&mut self, device: &wgpu::Device, variant: T, preprocessor: Option<&mut Preprocessor>) -> Rc<wgpu::ShaderModule> where T: AsRef<str> + AsRef<Path> + Hash + 'static {
      let mut hash = 0;
      if self.use_cache {
         let mut hasher = self.loaded_shaders.hasher().build_hasher();
         variant.hash(&mut hasher);
         std::any::TypeId::of::<T>().hash(&mut hasher); // hash of type, because vert/frag shader variants are stored in same cache

         preprocessor.as_ref().inspect(|p| p.hash(&mut hasher));
         hash = hasher.finish();
         if let Some(shader) = self.loaded_shaders.get(&hash) {
            // #[cfg(feature = "web")]
            // web_sys::console::log_1(&"Shader cache hit".into());
            log::warn!("Shader cache hit {hash}");
            return shader.clone()
         }
         // #[cfg(feature = "web")]
         // web_sys::console::log_1(&"Shader cache MISS".into());
         log::warn!("Shader cache MISS {hash}");
      }
      let shader = Rc::new(Self::build_shader(device, variant, preprocessor));
      self.loaded_shaders.insert(hash, shader.clone());
      shader
   }
   
   fn build_shader<T>(device: &wgpu::Device, variant: T, preprocessor: Option<&mut Preprocessor>) -> wgpu::ShaderModule where T: AsRef<str> + AsRef<Path> {
      let filepath: &std::path::Path = variant.as_ref();
      let source_code;
      cfg_if::cfg_if!(
         if #[cfg(feature = "web")] {
            // source code embedded during compilation
            source_code = variant.as_ref()
         } else {
            // source code loaded from filesystem
            let cwd = std::env::current_dir().expect("Failed to get current working dir");
            let abs_filepath = cwd.join("src").join("renderer").join(filepath);
            let source_code_owned = std::fs::read_to_string(&abs_filepath)
               .expect(format!("File not found: {}", abs_filepath.to_str().unwrap()).as_ref());
            source_code = source_code_owned.as_ref();
         }
      );
      ShaderLoader::build_shader_module(
         device, source_code, filepath.to_str().unwrap(), preprocessor)
   }

   fn build_shader_module(device: &wgpu::Device, source_code: &str, label: &str, preprocessor: Option<&mut Preprocessor>) -> wgpu::ShaderModule {
      let source_code = match preprocessor {
         Some(preprocessor) => preprocessor.process(source_code)
            .expect(format!("Failed to run preprocessor on shader: {}", label).as_ref()),
         _ => source_code.to_owned(),
      };
      Utils::make_shader(device, &source_code, label)
   }
}
