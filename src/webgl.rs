use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::Element;
use web_sys::{WebGl2RenderingContext};
use std::convert::TryInto;

pub fn canvas(canvas_dom_id: &str) -> Result<web_sys::HtmlCanvasElement, Element> {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id(canvas_dom_id).unwrap();
    canvas.dyn_into::<web_sys::HtmlCanvasElement>()
}

pub fn init_webgl_context(canvas: &web_sys::HtmlCanvasElement) -> Result<WebGl2RenderingContext, JsValue> {
    let gl: WebGl2RenderingContext = canvas
        .get_context("webgl2")?
        .unwrap()
        .dyn_into::<WebGl2RenderingContext>()?;
    update_webgl_viewport(&gl, canvas);
    Ok(gl)
}

pub fn update_webgl_viewport(gl: &WebGl2RenderingContext, canvas: &web_sys::HtmlCanvasElement) -> Result<(), JsValue> {
    gl.viewport(
        0,
        0,
        canvas.width().try_into().unwrap(),
        canvas.height().try_into().unwrap(),
    );
    Ok(())
}