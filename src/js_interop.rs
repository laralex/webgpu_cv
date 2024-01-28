use wasm_bindgen::{closure::Closure, JsCast};
use web_sys;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! js_log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}
pub(crate) use js_log;

pub fn now() -> f64 {
   web_sys::window()
       .expect("should have a Window")
       .performance()
       .expect("should have a Performance")
       .now()
}

pub fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

pub fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

pub fn document() -> web_sys::Document {
    window()
        .document()
        .expect("should have a document on window")
}

pub fn body() -> web_sys::HtmlElement {
    document().body().expect("document should have a body")
}