#![cfg(any(feature = "imgui", feature="imgui_web", feature="imgui_win"))]

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

      let hidpi_factor = window.scale_factor();
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
      (imgui, imgui_platform)
}