pub mod buffer;
pub mod utils;
pub use utils::*;
pub mod draw;
pub mod uniform;

use web_sys::HtmlCanvasElement;
struct SurfaceView {
   texture: Option<wgpu::SurfaceTexture>,
}
pub struct Webgpu {
   // pub surface: wgpu::Surface<'static>,
   pub device: wgpu::Device,
   pub queue: wgpu::Queue,
}

impl Webgpu {
   #[cfg(feature = "web")]
   pub async fn new(canvas: HtmlCanvasElement) -> (Self, wgpu::Surface<'static>, wgpu::SurfaceConfiguration) {
      // The instance is a handle to our GPU
      // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU

      let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
         backends: wgpu::Backends::GL,
         ..Default::default()
      });

      // # Safety
      // The surface needs to live as long as the window that created it.
      // State owns the window, so this should be safe.
      let (width, height) = (canvas.width(), canvas.height());
      let surface = instance.create_surface(wgpu::SurfaceTarget::Canvas(canvas)).unwrap();

      let adapter = instance.request_adapter(
         &wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
         },
      ).await.unwrap();

      let mut device_result = adapter.request_device(
         &Utils::default_device_descriptor(),
         None, // Trace path
      ).await;
      if let Err(e) = device_result {
         web_sys::console::log_1(
            &"Failed to request a webgpu device with default features, fallbacking to more compatible features".into());
         device_result = adapter.request_device(
            &Utils::downlevel_device_descriptor(),
            None, // Trace path
         ).await
      }
      let (device, queue) = device_result.expect("Failed to request wgpu device");

      let surface_caps = surface.get_capabilities(&adapter);
      let surface_format = surface_caps.formats.iter()
         .copied()
         .filter(|f| f.is_srgb())
         .next()
         .unwrap_or(surface_caps.formats[0]);

      let config = wgpu::SurfaceConfiguration {
         usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
         width,
         height,
         view_formats: vec![],
         desired_maximum_frame_latency: 2,
         format: surface_format,
         present_mode: surface_caps.present_modes[0],
         alpha_mode: surface_caps.alpha_modes[0],
      };

      surface.configure(&device, &config);

      ( Self { device, queue }, surface, config )
   }

   pub async fn new_offscreen() -> Self {
      let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
         backends: wgpu::Backends::GL,
         ..Default::default()
      });
      let adapter = instance
         .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: None,
            force_fallback_adapter: false,
         })
         .await
         .unwrap();
      let (device, queue) = adapter
         .request_device(&Utils::default_device_descriptor(), None)
         .await
         .unwrap();
      Self {device, queue}
   }

   // pub fn surface_configure(&self, config: &wgpu::SurfaceConfiguration) {
   //    self.surface.configure(&self.device, config);
   // }
}