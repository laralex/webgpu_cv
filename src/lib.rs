
mod js_interop;
mod webgl;
mod gl_utils;
mod renderer;

use renderer::{triangle::TriangleDemo, IDemo};

use wasm_bindgen::prelude::*;
use lazy_static::lazy_static;
use web_sys::WebGl2RenderingContext;
use std::{cell::RefCell, ops::Deref, rc::Rc, sync::Mutex};

// static CELL: Lazy<Box<&dyn renderer::IDemo>> = Lazy::new(|| Box::default());
// static DEMO: &mut dyn renderer::IDemo = Default::default();
lazy_static!{
    static ref FRAME_IDX: Mutex<usize> = Mutex::new(0);
    static ref PENDING_VIEWPORT_RESIZE: Mutex<Option<(u32, u32)>> = Mutex::new(None);
    // static ref GAME: Mutex<game_of_life::Grid> = Mutex::new(game_of_life::Grid::new(1, 1));
    //static ref CANVAS: RwLock<web_sys::HtmlCanvasElement> = RwLock::default();
    //static ref CANVAS_ID: RwLock<String> = RwLock::new("__stub__".to_owned());
}

#[wasm_bindgen]
pub fn wasm_startup() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    js_interop::js_log!("WASM Startup");
    // let mut gl = webgl::init_webgl_context(canvas_id)
    //     .expect("Must pass correct id of <canvas> element");
    // unsafe {
    //     DEMO = Box::new(renderer::triangle::TriangleDemo::new(&*gl));
    // }
    // demo.tick(0.5);
    //demo.render(&mut gl, 0.5);

    // *CANVAS_ID.write().unwrap() = canvas_id.to_owned();
}

#[wasm_bindgen]
pub fn wasm_resize(gl: *mut web_sys::WebGl2RenderingContext, width: u32, height: u32) {
    *PENDING_VIEWPORT_RESIZE.lock().unwrap() = Some((width, height));
    //webgl::update_webgl_viewport(&gl, (width, height));
    // gl.viewport(0, 0, width, height);
}


#[wasm_bindgen]
pub fn wasm_get_frame_idx() -> usize {
    FRAME_IDX.try_lock().map(|g| *g).unwrap_or_default()
}

#[wasm_bindgen]
pub fn wasm_loop(canvas_element_id: &str) -> Result<(), JsValue> {
    let engine_tick = Rc::new(RefCell::new(None)).clone();
    let engine_tick_clone = engine_tick.clone(); // to have a separate object, which is not owned by tick closure

    let mut gl = webgl::init_webgl_context(canvas_element_id).expect("Failed to get WebGL2 context");
    let mut demo = TriangleDemo::new(&gl);

    *engine_tick_clone.borrow_mut() = Some(Closure::new(move || {
        let mut guard = PENDING_VIEWPORT_RESIZE.try_lock().unwrap();
        if let Some((new_width, new_height)) = *guard {
            gl.viewport(0, 0, new_width as i32, new_height as i32);
            *guard = None;
        }
        demo.tick(0.001);
        demo.render(&mut gl, 0.001);
        *FRAME_IDX.lock().unwrap() += 1;
        // Schedule ourself for another requestAnimationFrame callback.
        js_interop::request_animation_frame(engine_tick.borrow().as_ref().unwrap());
    }));

    js_interop::request_animation_frame(engine_tick_clone.borrow().as_ref().unwrap());
    Ok(())
}

