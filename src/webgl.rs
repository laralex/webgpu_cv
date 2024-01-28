use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGl2RenderingContext, WebGlShader, WebGlProgram};
use std::convert::TryInto;

pub fn init_webgl_context(canvas_id: &str) -> Result<WebGl2RenderingContext, JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id(canvas_id).unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;
    let gl: WebGl2RenderingContext = canvas
        .get_context("webgl2")?
        .unwrap()
        .dyn_into::<WebGl2RenderingContext>()?;
    update_webgl_viewport(&gl, canvas_id);
    Ok(gl)
}

pub fn update_webgl_viewport(gl: &WebGl2RenderingContext, canvas_id: &str) -> Result<(), JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id(canvas_id).unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;
    gl.viewport(
        0,
        0,
        canvas.width().try_into().unwrap(),
        canvas.height().try_into().unwrap(),
    );
    Ok(())
}