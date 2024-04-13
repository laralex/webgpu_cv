use std::cell::RefCell;

use web_sys::HtmlCanvasElement;
use wgpu::{SurfaceError, TextureView};

struct SurfaceView {
   texture: Option<wgpu::SurfaceTexture>,
}
pub struct Webgpu {
   pub surface: wgpu::Surface<'static>,
   pub device: wgpu::Device,
   pub queue: wgpu::Queue,
}

impl Webgpu {
   pub async fn new(canvas: HtmlCanvasElement) -> (Self, wgpu::SurfaceConfiguration) {
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

       let (device, queue) = adapter.request_device(
           &wgpu::DeviceDescriptor {
               required_features: wgpu::Features::empty(),
               // WebGL doesn't support all of wgpu's features, so if
               // we're building for the web, we'll have to disable some.
               required_limits: if cfg!(target_arch = "wasm32") {
                   wgpu::Limits::downlevel_webgl2_defaults()
               } else {
                   wgpu::Limits::default()
               },
               label: None,
           },
           None, // Trace path
       ).await.unwrap();

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

       ( Self { surface, device, queue }, config )
   }

   pub async fn new_offscreen() -> (wgpu::Device, wgpu::Queue) {
      let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
         backends: wgpu::Backends::GL,
         ..Default::default()
      });
      let adapter = instance
         .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: None,
            force_fallback_adapter: true,
         })
         .await
         .unwrap();
      let (device, queue) = adapter
         .request_device(&Default::default(), None)
         .await
         .unwrap();
      (device, queue)
   }

   pub fn surface_configure(&self, config: &wgpu::SurfaceConfiguration) {
      self.surface.configure(&self.device, config);
   }
}