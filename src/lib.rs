
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
pub fn wasm_set_graphics_level(level_code: u64) {
    *PENDING_GRAPHICS_LEVEL_UPDATE.write().unwrap() = Some(match level_code {
        0x00 => GraphicsLevel::Minimal,
        0x10 => GraphicsLevel::Low,
        0x20 => GraphicsLevel::Medium,
        0x30 => GraphicsLevel::High,
        0xFF => GraphicsLevel::Ultra,
        _ => Default::default(),
    });
}

#[wasm_bindgen]
pub fn wasm_switch_demo() {

}

#[wasm_bindgen]
pub fn wasm_get_frame_idx() -> usize {
    FRAME_IDX.try_read().map_or(0, |r| *r)
}

fn configure_mousemove(canvas: &web_sys::HtmlCanvasElement, gl: &WebGl2RenderingContext, mouse_state: Rc<Cell<renderer::MouseState>>) -> Result<(), JsValue> {
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

fn configure_mousedown(canvas: &web_sys::HtmlCanvasElement, gl: &WebGl2RenderingContext, mouse_state: Rc<Cell<renderer::MouseState>>) -> Result<(), JsValue> {
    let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::MouseEvent| {
        let current_state = mouse_state.get();
        match event.button() {
            0 => mouse_state.set(MouseState {
                left: 1.0,
                ..current_state
            }),
            1 => mouse_state.set(MouseState {
                middle: 1.0,
                ..current_state
            }),
            2 => mouse_state.set(MouseState {
                right: 1.0,
                ..current_state
            }),
            _ => {},
        }
    });
    canvas.add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())?;
    closure.forget();
    Ok(())
}

fn configure_mouseup(canvas: &web_sys::HtmlCanvasElement, gl: &WebGl2RenderingContext, mouse_state: Rc<Cell<renderer::MouseState>>) -> Result<(), JsValue> {
    let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::MouseEvent| {
        let current_state = mouse_state.get();
        match event.button() {
            0 => mouse_state.set(MouseState {
                left: -1.0,
                ..current_state
            }),
            1 => mouse_state.set(MouseState {
                middle: -1.0,
                ..current_state
            }),
            2 => mouse_state.set(MouseState {
                right: -1.0,
                ..current_state
            }),
            _ => {},
        }
    });
    js_interop::window().add_event_listener_with_callback("mouseup", closure.as_ref().unchecked_ref())?;
    closure.forget();
    Ok(())
}

#[wasm_bindgen]
pub fn wasm_loop(canvas_dom_id: &str, target_fps: u32) -> Result<(), JsValue> {
    // callbacks wired with JS canvas
    // engine callback will schedule timeout callback (to limit fps)
    // timeout callback will schedule engine callback (to render the next frame)
    let engine_cb = Rc::new(RefCell::new(None));
    let engine_cb_clone = engine_cb.clone(); // to have a separate object, which is not owned by tick closure
    let timeout_cb = Rc::new(RefCell::new(None));
    let timeout_cb_clone = timeout_cb.clone();

    *timeout_cb_clone.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        js_interop::request_animation_frame(engine_cb.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));


    let canvas = webgl::canvas(canvas_dom_id)?;
    let mut gl = webgl::init_webgl_context(&canvas).expect("Failed to get WebGL2 context");
    let mouse_state = Rc::new(Cell::new(renderer::MouseState {
        left: 0.0,
        middle: 0.0,
        right: 0.0,
        wheel: 0.0, /* TODO: not populated */
        viewport_position: (0, 0),
    }));
    let mut demo_state = renderer::ExternalState {
        mouse: mouse_state,
        screen_size: (1, 1),
        time_delta_sec: 0.0,
        time_sec: 0.0,
        frame_idx: *FRAME_IDX.read().unwrap(),
        frame_rate: 1.0,
        date: chrono::Utc::now().date_naive(), /* NOTE: set once */
        sound_sample_rate: 44100.0, /* NOTE: set once */
    };
    configure_mousedown(&canvas, &gl, demo_state.mouse.clone())?;
    configure_mouseup(&canvas, &gl, demo_state.mouse.clone())?;
    configure_mousemove(&canvas, &gl, demo_state.mouse.clone())?;

    let mut demo = TriangleDemo::new(&gl, PENDING_GRAPHICS_LEVEL_UPDATE.read().unwrap().clone().unwrap_or_default());
    let mut time_then_sec = js_interop::now() * 0.001;
    let mut target_frame_time_ms = 1000 / target_fps as i32;
    *engine_cb_clone.borrow_mut() = Some(Closure::new(move || {
        // handle events
        if let Ok(reader) = PENDING_VIEWPORT_RESIZE.try_read() {
            if let Some((new_width, new_height)) = *reader {
                web_sys::console::log_3(&"Rust resize".into(), &new_width.into(), &new_height.into());
                gl.viewport(0, 0, new_width as i32, new_height as i32);
                demo_state.screen_size = (new_width, new_height);
                std::mem::drop(reader);
                match PENDING_VIEWPORT_RESIZE.try_write() {
                    Ok(mut w) => *w = None,
                    _ => web_sys::console::log_1(&"Failed to reset RwLock in wasm: PENDING_VIEWPORT_RESIZE".into()),
                }
            }
        }
        if let Ok(reader) = PENDING_FRAMETIME_LIMIT_UPDATE.try_read() {
            if let Some(new_target_frametime) = *reader {
                target_frame_time_ms = new_target_frametime.as_millis() as i32;
                std::mem::drop(reader);
                match PENDING_FRAMETIME_LIMIT_UPDATE.try_write() {
                    Ok(mut w) => *w = None,
                    _ => web_sys::console::log_1(&"Failed to reset RwLock in wasm: PENDING_FRAMETIME_LIMIT_UPDATE".into()),
                }
            }
        }
        if let Ok(reader) = PENDING_GRAPHICS_LEVEL_UPDATE.try_read() {
            if let Some(new_graphics_level) = *reader {
                demo.set_graphics_level(new_graphics_level);
                std::mem::drop(reader);
                match PENDING_GRAPHICS_LEVEL_UPDATE.try_write() {
                    Ok(mut w) => *w = None,
                    _ => web_sys::console::log_1(&"Failed to reset RwLock in wasm: PENDING_GRAPHICS_LEVEL_UPDATE".into()),
                }
            }
        }
        
        let time_now_sec = js_interop::now() * 0.001;
        let elapsed_sec = time_now_sec - time_then_sec;
        time_then_sec = time_now_sec;

        // populate engine step input
        demo_state.time_delta_sec = (elapsed_sec as f32).max(1e-6);
        demo_state.time_sec += demo_state.time_delta_sec;
        demo_state.frame_rate = 1.0 / demo_state.time_delta_sec;
        demo_state.frame_idx = *FRAME_IDX.read().unwrap();

        // engine step
        demo.tick(&demo_state);
        demo.render(&mut gl, demo_state.time_delta_sec);

        // prepare next frame
        *FRAME_IDX.write().unwrap() += 1;

        // clear mouseup events for next frame
        let mut current_mouse_state = demo_state.mouse.get();
        if current_mouse_state.left < 0.0 {
            current_mouse_state.left = 0.0;
        }
        if current_mouse_state.middle < 0.0 {
            current_mouse_state.middle = 0.0;
        }
        if current_mouse_state.right < 0.0 {
            current_mouse_state.right = 0.0;
        }
        if current_mouse_state.left > 0.0 {
            web_sys::console::log_3(&"Rust delta/time".into(), &demo_state.frame_rate.into(), &demo_state.time_sec.into());
        }
        demo_state.mouse.set(current_mouse_state);

        // Schedule ourself for another requestAnimationFrame callback.
        js_interop::set_frame_timeout(timeout_cb.borrow().as_ref().unwrap(), target_frame_time_ms);
    }));

    js_interop::request_animation_frame(engine_cb_clone.borrow().as_ref().unwrap());
    Ok(())
}

