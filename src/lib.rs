
mod js_interop;
mod webgl;
mod gl_utils;
mod renderer;

use renderer::{GraphicsLevel, MouseState};
use renderer::{triangle::TriangleDemo, IDemo};

use wasm_bindgen::prelude::*;
use lazy_static::lazy_static;
use web_sys::WebGl2RenderingContext;
use std::cell::Cell;
use std::sync::RwLock;
use std::{cell::RefCell, ops::Deref, rc::Rc, sync::Mutex};
use std::time::{Duration, Instant};

// static CELL: Lazy<Box<&dyn renderer::IDemo>> = Lazy::new(|| Box::default());
// static DEMO: &mut dyn renderer::IDemo = Default::default();
lazy_static!{
    static ref FRAME_IDX: RwLock<usize> = RwLock::new(0);
    static ref PENDING_VIEWPORT_RESIZE: RwLock<Option<(u32, u32)>> = RwLock::new(None);
    static ref PENDING_FRAMETIME_LIMIT_UPDATE: RwLock<Option<Duration>> = RwLock::new(None);
    static ref PENDING_GRAPHICS_LEVEL_UPDATE: RwLock<Option<GraphicsLevel>> = RwLock::new(None);
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
    *PENDING_VIEWPORT_RESIZE.write().unwrap() = Some((width, height));
}


#[wasm_bindgen]
pub fn wasm_set_fps_limit(fps_limit: u64) {
    *PENDING_FRAMETIME_LIMIT_UPDATE.write().unwrap() = Some(Duration::from_micros(1_000_000 / fps_limit));
}

#[wasm_bindgen]
pub fn wasm_set_graphics_level(level_code: u32) {
    *PENDING_GRAPHICS_LEVEL_UPDATE.write().unwrap() = Some(GraphicsLevel::from(level_code));
}

#[wasm_bindgen]
pub fn wasm_switch_demo() {

}

#[wasm_bindgen]
pub fn wasm_get_frame_idx() -> usize {
    FRAME_IDX.try_read().map_or(0, |r| *r)
}

fn configure_mousemove(canvas: &web_sys::HtmlCanvasElement, mouse_state: Rc<Cell<renderer::MouseState>>) -> Result<(), JsValue> {
    let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::MouseEvent| {
        let current_state = mouse_state.get();
        mouse_state.set(MouseState {
            viewport_position: (event.offset_x(), event.offset_y()),
            ..current_state
        });
    });
    canvas.add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())?;
    closure.forget();
    Ok(())
}

fn configure_mousedown(canvas: &web_sys::HtmlCanvasElement, mouse_state: Rc<Cell<renderer::MouseState>>) -> Result<(), JsValue> {
    let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::MouseEvent| {
        let current_state = mouse_state.get();
        match event.button() {
            0 => mouse_state.set(MouseState { left: 1.0, ..current_state }),
            1 => mouse_state.set(MouseState { middle: 1.0, ..current_state }),
            2 => mouse_state.set(MouseState { right: 1.0, ..current_state }),
            _ => {},
        }
    });
    canvas.add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())?;
    closure.forget();
    Ok(())
}

fn configure_mouseup(mouse_state: Rc<Cell<renderer::MouseState>>) -> Result<(), JsValue> {
    let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::MouseEvent| {
        let current_state = mouse_state.get();
        match event.button() {
            0 => mouse_state.set(MouseState { left: -1.0, ..current_state }),
            1 => mouse_state.set(MouseState { middle: -1.0, ..current_state }),
            2 => mouse_state.set(MouseState { right: -1.0, ..current_state }),
            _ => {},
        }
    });
    js_interop::window().add_event_listener_with_callback("mouseup", closure.as_ref().unchecked_ref())?;
    closure.forget();
    Ok(())
}

fn poll_pending_event<T, F: FnMut(&T)>(event: &RwLock<Option<T>>, mut handler: F) {
    if let Ok(reader) = event.try_read() {
        if let Some(v) = reader.as_ref() {
            handler(&v);
            std::mem::drop(reader);
            match event.try_write() {
                Ok(mut w) => *w = None,
                _ => web_sys::console::log_1(&"Failed to reset RwLock in wasm".into()),
            }
        }
    }
}

#[wasm_bindgen]
pub fn wasm_loop(canvas_dom_id: &str, target_fps: u32, graphics_level_code: u32) -> Result<(), JsValue> {
    // callbacks wired with JS canvas
    // engine callback will schedule timeout callback (to limit fps)
    // timeout callback will schedule engine callback (to render the next frame)
    let engine_cb = Rc::new(RefCell::new(None));
    let engine_cb_clone = engine_cb.clone(); // to have a separate object, which is not owned by tick closure
    let timeout_cb = Rc::new(RefCell::new(None));
    let timeout_cb_clone = timeout_cb.clone();

    let canvas = webgl::canvas(canvas_dom_id)?;
    let mut gl = webgl::init_webgl_context(&canvas).expect("Failed to get WebGL2 context");
    let mut demo_state = renderer::ExternalState::default();
    demo_state.date = chrono::Utc::now().date_naive(); /* NOTE: set once */
    demo_state.sound_sample_rate = 44100.0; /* NOTE: set once */
    configure_mousedown(&canvas, demo_state.mouse.clone())?;
    configure_mouseup(demo_state.mouse.clone())?;
    configure_mousemove(&canvas, demo_state.mouse.clone())?;

    let mut demo = TriangleDemo::new(&gl, GraphicsLevel::from(graphics_level_code));
    let mut time_then_sec = js_interop::now_sec();
    let mut target_frame_time_ms = 1000 / target_fps as i32;
    *engine_cb_clone.borrow_mut() = Some(Closure::new(move || {
        // handle events
        poll_pending_event(&PENDING_VIEWPORT_RESIZE, |&(new_width, new_height)| {
            web_sys::console::log_3(&"Rust resize".into(), &new_width.into(), &new_height.into());
            gl.viewport(0, 0, new_width as i32, new_height as i32);
            demo_state.screen_size = (new_width, new_height);
        });
        poll_pending_event(&PENDING_GRAPHICS_LEVEL_UPDATE, |&new_graphics_level| {
            demo.set_graphics_level(new_graphics_level);
        });
        poll_pending_event(&PENDING_FRAMETIME_LIMIT_UPDATE, |&new_target_frametime| {
            target_frame_time_ms = new_target_frametime.as_millis() as i32;
        });
        
        let time_now_sec = js_interop::now_sec();
        let elapsed_sec = time_now_sec - time_then_sec;
        time_then_sec = time_now_sec;

        // engine step
        demo_state.begin_frame(elapsed_sec as f32);
        demo.tick(&demo_state);
        demo.render(&mut gl, demo_state.time_delta_sec);
        demo_state.end_frame();
        *FRAME_IDX.write().unwrap() = demo_state.frame_idx;

        // Schedule ourself for another requestAnimationFrame callback.
        js_interop::set_frame_timeout(timeout_cb.borrow().as_ref().unwrap(), target_frame_time_ms);
    }));

    *timeout_cb_clone.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        js_interop::request_animation_frame(engine_cb.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    js_interop::request_animation_frame(engine_cb_clone.borrow().as_ref().unwrap());

    Ok(())
}

