use std::rc::Rc;
use futures::Future;
use wgpu::BufferUsages;

use crate::renderer::pipeline_loader::RenderPipelineFlatDescriptor;
use crate::renderer::webgpu::Utils;

use super::shader_loader::{FragmentShaderVariant, VertexShaderVariant};
use super::webgpu::buffer::{Buffer, IndexBuffer, VertexBuffer, VertexPosUv};
use super::webgpu::uniform::BindGroupInfo;
use super::webgpu::PipelineLayoutBuilder;
use super::{DemoLoadingFuture, DemoLoadingSimpleFuture, Dispose, ExternalState, GraphicsLevel, IDemo, LoadingArgs, Progress, RenderArgs, SimpleFuture, Webgpu};

const VERTEX_SHADER_VARIANT:   VertexShaderVariant   = VertexShaderVariant::Passthrough;
const FRAGMENT_SHADER_VARIANT: FragmentShaderVariant = FragmentShaderVariant::Uv;

#[derive(Default)]
enum DemoLoadingStage {
   Ready = 0,
   #[default] CompileShaders,
   BuildUniforms,
   BuildVertexData,
   BuildPipelines,
   StartSwitchingGraphicsLevel,
   SwitchGraphicsLevel,
}

struct DemoLoadingProcess {
   stage: DemoLoadingStage,
   stage_percent: f32,
   graphics_level: GraphicsLevel,
   loading_args: LoadingArgs,
   vertex_shader: Option<Rc<wgpu::ShaderModule>>,
   fragment_shader: Option<Rc<wgpu::ShaderModule>>,
   index_buffer: Option<IndexBuffer>,
   vertex_buffer: Option<VertexBuffer>,
   render_pipeline: Option<Rc<wgpu::RenderPipeline>>,
   loaded_demo: Option<Demo>,
   uniform_groups: Vec<BindGroupInfo>
}

impl Dispose for DemoLoadingProcess {
   fn dispose(&mut self) {
      match self.stage {
         DemoLoadingStage::Ready => {
            // demo is fully loaded, its lifetime is now separate, 
            // shouldn't free its resources
         },
         _ => {
            self.render_pipeline.take();
            self.vertex_shader.take();
            self.fragment_shader.take();
            self.stage = DemoLoadingStage::Ready;
            self.loaded_demo.take();
            log::info!("Rust loading drop: {}", std::module_path!());
         },
      }
   }
}

impl Drop for DemoLoadingProcess {
   fn drop(&mut self) {
      self.dispose();
   }
}

impl Progress for DemoLoadingProcess {
    fn progress(&self) -> f32 {
        self.stage_percent
    }
}

impl Future for DemoLoadingProcess {
   type Output = Box<dyn IDemo>;
   
   fn poll(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
      match self.as_mut().simple_poll(cx) {
         std::task::Poll::Pending => {
            cx.waker().wake_by_ref();
            std::task::Poll::Pending
         }
         poll => poll,
      }
   }
}

impl DemoLoadingSimpleFuture for DemoLoadingProcess{}

impl DemoLoadingFuture for DemoLoadingProcess {}

impl SimpleFuture for DemoLoadingProcess {
   type Output = Box<dyn IDemo>;
   //type Context = std::task::Context<'a>;

   fn simple_poll(mut self: std::pin::Pin<&mut Self>, _cx: &mut std::task::Context) -> std::task::Poll<Self::Output> {
      use DemoLoadingStage::*;
      match self.stage {
         CompileShaders => {
            self.compile_shaders();
            self.stage_percent = 0.3;
            self.stage = BuildUniforms;
         },
         BuildUniforms => {
            self.load_assets();
            self.stage_percent = 0.4;
            self.stage = BuildVertexData;
         },
         BuildVertexData => {
            self.build_vertex_data();
            self.stage_percent = 0.5;
            self.stage = BuildPipelines;
         },
         BuildPipelines => {
            self.build_pipelines();
            self.stage_percent = 0.6;
            self.stage = StartSwitchingGraphicsLevel;
         },
         StartSwitchingGraphicsLevel => {
            self.start_switching_graphics_level();
            self.stage_percent = 0.75;
            self.stage = SwitchGraphicsLevel;
         },
         SwitchGraphicsLevel => {
            let webgpu = self.loading_args.webgpu.clone();
            match self.loaded_demo.as_mut().unwrap().poll_switching_graphics_level(webgpu.as_ref()) {
               Ok(std::task::Poll::Pending)  => {
                  self.stage_percent = 0.75 + 0.25 * self.loaded_demo.as_ref().unwrap().progress_switching_graphics_level();
               },
               Ok(std::task::Poll::Ready(())) => {
                  self.stage_percent = 1.0;
                  self.stage = Ready;
                  return std::task::Poll::Ready(Box::new(
                     self.loaded_demo.take().unwrap()
                  ))
               },
               Err(e) => {
                  eprintln!("Error when switching graphics level: {} {}", std::module_path!(), e);
               },
            }
         }
         Ready => unreachable!("Should not poll the task again after std::task::Poll::Ready was polled"),
      }
      std::task::Poll::Pending
   }
}

impl DemoLoadingProcess {
   fn new(loading_args: LoadingArgs, graphics_level: GraphicsLevel) -> Self {
      Self {
         stage: Default::default(),
         stage_percent: 0.0,
         graphics_level,
         loading_args,
         render_pipeline: Default::default(),
         vertex_shader: Default::default(),
         fragment_shader: Default::default(),
         index_buffer: Default::default(),
         vertex_buffer: Default::default(),
         loaded_demo: Default::default(),
         uniform_groups: vec![],
      }
   }

   fn rebuild_pipelines(&mut self) {
      self.compile_shaders();
      self.make_bind_groups();
      self.build_pipelines();
   }

   fn compile_shaders(&mut self) {
      self.vertex_shader = Some(self.loading_args
         .get_vertex_shader(VERTEX_SHADER_VARIANT, None));
      self.fragment_shader = Some(self.loading_args
         .get_fragment_shader(FRAGMENT_SHADER_VARIANT, None));
   }

   fn make_bind_groups(&mut self) {

   }
   
   fn load_assets(&mut self) {
      let mut asset_loader = self.loading_args.asset_loader.borrow_mut();
      let mut load_texture = |path: &str, format: wgpu::TextureFormat| {
         let guid = asset_loader.load_texture(path.to_owned());
         let image = asset_loader.get_texture(guid).unwrap();
         let image_size = image.dimensions();
         let texture_size = wgpu::Extent3d {
            width: image_size.0,
            height: image_size.1,
            depth_or_array_layers: image_size.2,
        };
        self.loading_args.webgpu.device.create_texture(
            &wgpu::TextureDescriptor {
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                // Most images are stored using sRGB, so we need to reflect that here.
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                label: Some(path),
                view_formats: &[],
            }
        )
      };
      let albedo_texture = load_texture("assets/materials/leather/Leather_Padded_001_basecolor.jpg", wgpu::TextureFormat::Rgba8UnormSrgb);
      let occlusion_texture = load_texture("assets/materials/leather/Leather_Padded_001_ambientOcclusion.jpg", wgpu::TextureFormat::Rgba8UnormSrgb);
      let normal_texture = load_texture("assets/materials/leather/Leather_Padded_001_normal.jpg", wgpu::TextureFormat::Rgba8UnormSrgb);
      let height_texture = load_texture("assets/materials/leather/Leather_Padded_001_height.png", wgpu::TextureFormat::Rgba8UnormSrgb);
      let roughness_texture = load_texture("assets/materials/leather/Leather_Padded_001_roughness.jpg",
         wgpu::TextureFormat::Rgba8UnormSrgb);
      let texture_bind_group = BindGroupInfo::builder()
         .with_texture_2d(0, wgpu::ShaderStages::FRAGMENT,
            wgpu::TextureSampleType::Float { filterable:true },
            &Utils::texture_view(&albedo_texture, Some("albedo_lod")),)
         .with_sampler(1, wgpu::ShaderStages::FRAGMENT,
            &self.loading_args.premade.borrow().samplers.bilinear_sampler)
         .build(&self.loading_args.webgpu.device, Some("Fractal Bind Group"), None);

      self.uniform_groups = vec![texture_bind_group];
   }

   fn build_vertex_data(&mut self) {
      let wgpu = self.loading_args.webgpu.as_ref();
      const VERTICES: &[VertexPosUv] = &[
         VertexPosUv { position: [-0.0868241, 0.49240386, 0.0], uv: [0.4131759, 0.99240386], },
         VertexPosUv { position: [-0.49513406, 0.06958647, 0.0], uv: [0.0048659444, 0.56958647], },
         VertexPosUv { position: [-0.21918549, -0.44939706, 0.0], uv: [0.28081453, 0.05060294], },
         VertexPosUv { position: [0.35966998, -0.3473291, 0.0], uv: [0.85967, 0.1526709], },
         VertexPosUv { position: [0.44147372, 0.2347359, 0.0], uv: [0.9414737, 0.7347359], },
      ];
      self.vertex_buffer = Some(Buffer::new_vertex_init(
         &wgpu.device, bytemuck::cast_slice(VERTICES),
         BufferUsages::empty(), Some("Mesh attributes")));

      const INDICES: &[u16] = &[
         0, 1, 4,
         1, 2, 4,
         2, 3, 4,
      ];
      self.index_buffer = Some(Buffer::new_index_init(
         &wgpu.device, bytemuck::cast_slice(INDICES),
         wgpu::IndexFormat::Uint16, BufferUsages::empty(), Some("Mesh indices")));
   }

   fn build_pipelines(&mut self) {
      let mut builder = PipelineLayoutBuilder::new();
      let premade = self.loading_args.premade.borrow();
      builder = builder.with(&premade.global_uniform.bind_group_info);
      for group in self.uniform_groups.iter() {
         builder = builder.with(group);
      }
      let layout_descriptor = wgpu::PipelineLayoutDescriptor {
         label: Some("Render Pipeline Layout"),
         bind_group_layouts: &[&premade.global_uniform.bind_group_info.layout],
         push_constant_ranges: &[],
      };
      let render_pipeline_layout = self.loading_args.webgpu.device
         .create_pipeline_layout(&layout_descriptor);
      let vs = self.vertex_shader.take().unwrap();
      let fs = self.fragment_shader.take().unwrap();
      self.render_pipeline = Some(self.loading_args.get_pipeline(
         &RenderPipelineFlatDescriptor::new(
         &layout_descriptor,
         &wgpu::RenderPipelineDescriptor {
         label: Some("Render Pipeline"),
         layout: Some(&render_pipeline_layout),
         vertex: wgpu::VertexState {
               module: &vs,
               entry_point: "vs_main",
               buffers: &[VertexPosUv::layout()],
         },
         fragment: Some(wgpu::FragmentState {
               module: &fs,
               entry_point: "fs_main",
               targets: &[Some(wgpu::ColorTargetState {
                  format: self.loading_args.color_texture_format,
                  blend: Some(wgpu::BlendState::REPLACE),
                  write_mask: wgpu::ColorWrites::ALL,
               })],
         }),
         primitive: Utils::default_primitive_state(),
         depth_stencil: None, // 1.
         multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
         },
         multiview: None,
      })));
   }

   fn start_switching_graphics_level(&mut self) {
      self.loaded_demo = Some(Demo {
         render_pipeline: self.render_pipeline.take().unwrap(),
         index_buffer: self.index_buffer.take().unwrap(),
         vertex_buffer: self.vertex_buffer.take().unwrap(),
         pending_graphics_level_switch: None,
         graphcis_level: self.graphics_level,
      });
      self.loaded_demo.as_mut().unwrap()
         .start_switching_graphics_level(self.loading_args.clone(), self.graphics_level)
         .expect("WebGPU surface error");
   }
}

pub struct Demo {
   render_pipeline: Rc<wgpu::RenderPipeline>,
   index_buffer: IndexBuffer,
   vertex_buffer: VertexBuffer,
   pending_graphics_level_switch: Option<GraphicsSwitchingProcess>,
   graphcis_level: GraphicsLevel,
}

impl IDemo for Demo {
   fn tick(&mut self, _input: &ExternalState) {

   }

   fn render(&mut self, args: RenderArgs) -> Result<(), wgpu::SurfaceError> {
      let color = Some(Utils::surface_view(args.backbuffer));
      let mut encoder = args.webgpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
         label: Some("Render Encoder"),
      });

      {
         let mut render_pass = Utils::default_renderpass(&mut encoder, &color,&None);
         const DEMO_UNIFORM_BIND_GROUP_INDEX: u32 = 0;
         render_pass.set_bind_group(DEMO_UNIFORM_BIND_GROUP_INDEX, &args.global_uniform.bind_group_info.bind_group, &[]);
         render_pass.set_pipeline(&self.render_pipeline);
         const VERTEX_POS_UV_LOCATION: u32 = 0;
         self.vertex_buffer.bind(&mut render_pass, VERTEX_POS_UV_LOCATION);
         self.index_buffer.bind(&mut render_pass);
         render_pass.draw_indexed(0..self.index_buffer.num_indices, 0, 0..1);
      }
   
      // submit will accept anything that implements IntoIter
      args.webgpu.queue.submit(std::iter::once(encoder.finish()));
      Ok(())
   }

   fn rebuild_pipelines(&mut self, loading_args: LoadingArgs) {
      let mut loader = DemoLoadingProcess::new(loading_args, self.graphcis_level);
      loader.rebuild_pipelines();
      self.render_pipeline = loader.render_pipeline.take().unwrap();
   }

   #[cfg(any(feature = "imgui_win", feature = "imgui_web"))]
   fn render_imgui(&mut self, ui: &imgui::Ui, args: super::imgui_web::ImguiRenderArgs) {
      use imgui::*;
      let window = ui.window("Uv Sandbox Demo");
      window
         .size(args.size, Condition::FirstUseEver)
         .position(args.position, Condition::FirstUseEver)
         .always_auto_resize(true)
         .build(|| {
         });
   }

   fn start_switching_graphics_level(&mut self, _args: LoadingArgs, graphics_level: GraphicsLevel) -> Result<(), wgpu::SurfaceError> {
      log::info!("Rust start_switching_graphics_level {}", std::module_path!());
      self.pending_graphics_level_switch = Some(GraphicsSwitchingProcess{
         progress: 0.0,
         graphics_level,
      });
      Ok(())
   }

   fn poll_switching_graphics_level(&mut self, webgpu: &Webgpu) -> Result<std::task::Poll<()>, wgpu::SurfaceError> {
      if self.pending_graphics_level_switch.is_some() {
         Ok(GraphicsSwitchingProcess::poll(self, webgpu))
      } else {
         self.pending_graphics_level_switch.take();
         Ok(std::task::Poll::Ready(()))
      }
   }

   fn progress_switching_graphics_level(&self) -> f32 {
      self.pending_graphics_level_switch
         .as_ref()
         .map(|s| s.progress())
         .unwrap_or_default()
   }

   fn drop_demo(&mut self, _webgpu: &Webgpu) {
      log::info!("Rust demo drop {}", std::module_path!());
   }
}

impl Demo {
   pub fn start_loading<'a>(args: LoadingArgs, graphics_level: GraphicsLevel) -> Box<dyn DemoLoadingFuture> {
      Box::new(DemoLoadingProcess::new(args, graphics_level))
   }
}

pub struct GraphicsSwitchingProcess {
   progress: f32,
   #[allow(unused)] graphics_level: GraphicsLevel,
}

impl Progress for GraphicsSwitchingProcess {
    fn progress(&self) -> f32 {
      self.progress
   }
}

impl GraphicsSwitchingProcess {
   pub fn poll(demo: &mut Demo, _webgpu: &Webgpu) -> std::task::Poll<()> {
      let self_ = demo.pending_graphics_level_switch.as_mut().unwrap();
      self_.progress = 1.0;
      std::task::Poll::Ready(())
   }
}


#[cfg(test)]
mod tests {
   use std::cell::RefCell;
   use crate::renderer::{asset_loader::AssetLoader, Premade};

   use super::*;

   #[test]
   fn shaders_compile() {
      let webgpu = futures::executor::block_on(Webgpu::new_offscreen());
      let premade = Premade::new(&webgpu.device);
      let asset_loader = AssetLoader::new();
      let loading_args = LoadingArgs {
         webgpu: Rc::new(webgpu),
         color_texture_format: wgpu::TextureFormat::Rgba8Unorm,
         premade: Rc::new(RefCell::new(premade)),
         asset_loader: Rc::new(RefCell::new(asset_loader)),
      };
      let mut demo_loader = DemoLoadingProcess::new(loading_args, GraphicsLevel::Medium);
      demo_loader.compile_shaders();
   }
}