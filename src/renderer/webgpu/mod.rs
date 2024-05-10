pub mod buffer;
pub mod utils;
pub use utils::*;
pub mod draw;
pub mod uniform;
pub mod texture;

pub struct Webgpu {
   pub device: wgpu::Device,
   pub queue: wgpu::Queue,
}

pub struct WebgpuSurface<'window> {
   pub surface: wgpu::Surface<'window>,
   pub config: wgpu::SurfaceConfiguration,
}

impl Webgpu {
   #[cfg(feature = "web")]
   pub async fn new_with_canvas(power_preference: wgpu::PowerPreference) -> (web_sys::HtmlCanvasElement, Self, WebgpuSurface<'static>){
      use wasm_bindgen::JsCast;
      use crate::timer::ScopedTimer;
      let try_init_webgpu = |backend| async move {
         let _t = ScopedTimer::new("webgpu::try_init_webgpu");
         let document = web_sys::window().unwrap().document().unwrap();
         let canvas = document.create_element("canvas").unwrap();
         let canvas = canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();

         let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: backend, ..Default::default() });

         // # Safety
         let canvas_js: &wasm_bindgen::JsValue = &canvas;
         let raw_window_handle = wgpu::rwh::WebCanvasWindowHandle::new(
            std::ptr::NonNull::from(canvas_js).cast().into()).into();
         let raw_display_handle = wgpu::rwh::WebDisplayHandle::new().into();

         // NOTE: can panic, can't catch it with std::panic::catch_unwind !!!
         let surface = unsafe { instance.create_surface_unsafe(
            wgpu::SurfaceTargetUnsafe::RawHandle { raw_display_handle, raw_window_handle })
         }.map_err(|e| web_sys::console::log_2(&"Failed to create wgpu surface: ".into(), &e.to_string().into())).ok();
         if surface.is_none() {
            canvas.remove();
            return None;
         }
         
         let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
               power_preference,
               compatible_surface: Some(&surface.as_ref().unwrap()),
               force_fallback_adapter: false,
            },
         ).await;
         if adapter.is_none() {
            canvas.remove();
            return None;
         }
         Some((canvas, instance, surface.unwrap(), adapter.unwrap()))
      };
      
      let mut webgpu_artifacts = None;
      let backends_to_try = &[
         // (wgpu::Backends::DX12, "DX12".to_owned()),
         // (wgpu::Backends::BROWSER_WEBGPU, "WebGPU".to_owned()),
         // (wgpu::Backends::PRIMARY, "Vulkan/Metal/DX12/WebGPU".to_owned()),
         (wgpu::Backends::GL, "WebGL".to_owned()),
      ];
      for (backend, backend_name) in backends_to_try {
         log::warn!("Loading wgpu backend {backend_name}");
         webgpu_artifacts = try_init_webgpu(backend.clone()).await;
         if webgpu_artifacts.is_some() {
            log::warn!("Successful loading of backend {backend_name}");
            break;
         }
      }
      let _t = ScopedTimer::new("webgpu::new_with_canvas");
      let (canvas, _instance, surface, adapter) = webgpu_artifacts
         .expect("No wgpu backends are available. Can't start the application");
      let (width, height) = (canvas.width(), canvas.height()); // TODO: newly created, maybe pass size, or resize in init_fn

      let mut device_result = adapter.request_device(
         &Utils::default_device_descriptor(),
         None, // Trace path
      ).await;

      if let Err(e) = device_result {
         web_sys::console::log_2(
            &"Failed to get a webgpu device with default features".into(), &e.to_string().into());
         // fallback to more compatible features
         device_result = adapter.request_device(
            &Utils::downlevel_device_descriptor(),
            None,
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
      
      ( canvas, Self { device, queue }, WebgpuSurface{surface, config} )
   }

   #[cfg(feature = "win")]
   pub async fn new_with_winit(window: &winit::window::Window) -> ( Self, WebgpuSurface<'static> ) {
      use wgpu::rwh::{HasDisplayHandle,HasWindowHandle};
      let size = window.inner_size();

      let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
         backends: wgpu::Backends::all(),
         ..Default::default()
      });
      
      // # Safety
      // The surface needs to live as long as the window that created it.
      let surface = unsafe { instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle { 
         raw_display_handle: window.display_handle().unwrap().into(), 
         raw_window_handle: window.window_handle().unwrap().into(),
      }) }.unwrap();

      let adapter = instance.request_adapter(
         &wgpu::RequestAdapterOptions {
               power_preference: wgpu::PowerPreference::default(),
               compatible_surface: Some(&surface),
               force_fallback_adapter: false,
         },
      ).await.unwrap();

      let (device, queue) = adapter
         .request_device(&Utils::default_device_descriptor(), None)
         .await.unwrap();

      let surface_caps = surface.get_capabilities(&adapter);
      // Shader code in this tutorial assumes an sRGB surface texture. Using a different
      // one will result in all the colors coming out darker. If you want to support non
      // sRGB surfaces, you'll need to account for that when drawing to the frame.
      let surface_format = surface_caps.formats.iter()
         .copied()
         .filter(|f| f.is_srgb())
         .next()
         .unwrap_or(surface_caps.formats[0]);
      let config = wgpu::SurfaceConfiguration {
         usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
         format: surface_format,
         width: size.width,
         height: size.height,
         present_mode: surface_caps.present_modes[0],
         alpha_mode: surface_caps.alpha_modes[0],
         view_formats: vec![],
         desired_maximum_frame_latency: 2,
      };
      surface.configure(&device, &config);

      ( Self { device, queue }, WebgpuSurface{ surface, config } )
   }

   #[allow(unused)]
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

      Self {device, queue }
   }
}