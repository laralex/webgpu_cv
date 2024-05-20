#[cfg(feature="web")]
pub async fn load_image(image_path: &str) -> (Vec<u8>, u32, u32) {
   let t = crate::timer::ScopedTimer::new("load_image");
   use wasm_bindgen::JsCast;
   use std::ops::Deref;
   let image = web_sys::HtmlImageElement::new().unwrap();
   image.set_src(image_path);
   let image_load_promise = image.decode();
   let result = wasm_bindgen_futures::JsFuture::from(image_load_promise)
      .await
      .unwrap();
   let (width, height) = (image.width(), image.height());
   let canvas = web_sys::OffscreenCanvas::new(width, height).unwrap();
   let context = canvas
      .get_context("2d")
      .unwrap()
      .unwrap()
      .dyn_into::<web_sys::OffscreenCanvasRenderingContext2d>()
      .unwrap();
   context.draw_image_with_html_image_element(&image, 0.0, 0.0);
   let image_data = context
      .get_image_data(0.0, 0.0, width as f64, height as f64)
      .unwrap()
      .data()
      .deref()
      .clone();
   // canvas.remove();
   image.remove();
   (image_data, width, height)
}

#[cfg(feature="not_web")]
pub async fn load_image(image_path: &str) -> (Vec<u8>, u32, u32) {
   let _t = crate::timer::ScopedTimer::new("load_image");
   let cwd = std::env::current_dir().expect("Failed to get current working dir");
   let abs_filepath = cwd.join("www").join(image_path);
   let img = image::io::Reader::open(abs_filepath)
      .unwrap()
      .decode()
      .unwrap();
   let decoded_bytes = img.to_rgba8()
      .into_vec();
   use image::GenericImageView;
   let dimensions = img.dimensions();
   (decoded_bytes, dimensions.0, dimensions.1)
}