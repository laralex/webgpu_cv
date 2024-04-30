#![cfg(any(feature = "imgui", feature="imgui_web", feature="imgui_win"))]

#[cfg(feature = "web")]
pub mod web_platform;

#[cfg(feature = "imgui_win")]
pub fn init_from_winit(window: &winit::window::Window) -> (imgui::Context, imgui_winit_support::WinitPlatform) {
   let mut imgui = imgui::Context::create();
   let mut imgui_platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
   imgui_platform.attach_window(
      imgui.io_mut(),
      &window,
      imgui_winit_support::HiDpiMode::Default,
   );
   imgui.set_ini_filename(None);
   configure_font(&mut imgui, window.scale_factor());

   (imgui, imgui_platform)
}

#[cfg(feature = "imgui_web")]
pub fn init_from_raw(display_size_px: web_platform::PhysicalSize, hidpi_factor: f64) -> (imgui::Context, web_platform::WebsysPlatform) {
   let mut imgui = imgui::Context::create();
   let imgui_platform = web_platform::WebsysPlatform::init(
      &mut imgui,
      display_size_px,
      hidpi_factor,
      web_platform::HiDpiMode::Default,
   );
   imgui.set_ini_filename(None);
   configure_font(&mut imgui, hidpi_factor);

   (imgui, imgui_platform)
}

pub fn configure_font(imgui: &mut imgui::Context, hidpi_factor: f64) {
   let font_size = (13.0 * hidpi_factor) as f32;
   imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

   imgui.fonts().add_font(&[imgui::FontSource::DefaultFontData {
      config: Some(imgui::FontConfig {
            oversample_h: 1,
            pixel_snap_h: true,
            size_pixels: font_size,
            ..Default::default()
      }),
   }]);
}