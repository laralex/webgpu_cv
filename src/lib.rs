
mod js_interop;
mod webgl;
mod gl_utils;
mod renderer;

use renderer::MouseState;
use renderer::{triangle::TriangleDemo, IDemo};

use wasm_bindgen::prelude::*;
use lazy_static::lazy_static;
use web_sys::WebGl2RenderingContext;
use std::cell::Cell;
use std::{cell::RefCell, ops::Deref, rc::Rc, sync::Mutex};
use std::time::{Duration, Instant};

// static CELL: Lazy<Box<&dyn renderer::IDemo>> = Lazy::new(|| Box::default());
// static DEMO: &mut dyn renderer::IDemo = Default::default();
lazy_static!{
    static ref FRAME_IDX: Mutex<usize> = Mutex::new(0);
    static ref PENDING_VIEWPORT_RESIZE: Mutex<Option<(u32, u32)>> = Mutex::new(None);
    static ref PENDING_FRAMETIME_LIMIT_UPDATE: Mutex<Option<Duration>> = Mutex::new(None);
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

// #[wasm_bindgen]
// pub fn wasm_get_resize_callback() -> Closure<dyn FnMut(u32, u32)> {

// }

#[wasm_bindgen]
pub fn wasm_set_fps_limit(fps_limit: u64) {
    *PENDING_FRAMETIME_LIMIT_UPDATE.lock().unwrap() = Some(Duration::from_micros(1_000_000 / fps_limit));
}

#[wasm_bindgen]
pub fn wasm_switch_demo() {

}

#[wasm_bindgen]
pub fn wasm_get_frame_idx() -> usize {
    FRAME_IDX.try_lock().map(|g| *g).unwrap_or_default()
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
        web_sys::console::log_1(&"Rust mousedown".into());
    });
    canvas.add_event_listener_with_callback("mouseup", closure.as_ref().unchecked_ref())?;
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
        viewport_position: (0, 0),
        unit_position: (0.0, 0.0),
    }));
    let mut demo_state = renderer::ExternalState {
        mouse: mouse_state,
        screen_size: Rc::new(Cell::new((0, 0))),
        delta_sec: 0.0,
    };
    configure_mousedown(&canvas, &gl, demo_state.mouse.clone())?;
    configure_mouseup(&canvas, &gl, demo_state.mouse.clone())?;
    configure_mousemove(&canvas, &gl, demo_state.mouse.clone())?;

    let mut demo = TriangleDemo::new(&gl);
    //let mut time_then = js_interop::now();
    let mut target_frametime_ms = 1000 / target_fps as i32;
    *engine_cb_clone.borrow_mut() = Some(Closure::new(move || {
        let mut guard = PENDING_VIEWPORT_RESIZE.try_lock().unwrap();
        if let Some((new_width, new_height)) = *guard {
            gl.viewport(0, 0, new_width as i32, new_height as i32);
            *guard = None;
        }
        if let Some(new_target_frametime) = *PENDING_FRAMETIME_LIMIT_UPDATE.try_lock().unwrap() {
            target_frametime_ms = new_target_frametime.as_millis() as i32;
        }
        //let time_now = js_interop::now();
        //let elapsed = Duration::from_secs_f64(time_now - time_then);
        //time_then = time_now;
        //let elapsed_sec = elapsed.as_secs_f32();

        let tick = 0.1 / target_frametime_ms as f32;
        //web_sys::console::log_2(&"TICK: ".into(), &tick.into());
        demo_state.delta_sec = tick;

        demo.tick(&demo_state);
        demo.render(&mut gl, tick);

        *FRAME_IDX.lock().unwrap() += 1;

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
        demo_state.mouse.set(current_mouse_state);

        // Schedule ourself for another requestAnimationFrame callback.
        js_interop::set_frame_timeout(timeout_cb.borrow().as_ref().unwrap(), target_frametime_ms);
    }));

    js_interop::request_animation_frame(engine_cb_clone.borrow().as_ref().unwrap());
    Ok(())
}

