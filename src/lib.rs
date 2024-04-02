
mod js_interop;
mod webgl;
mod gl_utils;
mod renderer;

use renderer::{GraphicsLevel, MouseState, ExternalState};
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
// lazy_static!{
//     static ref FRAME_IDX: RwLock<usize> = RwLock::new(0);
//     static ref PENDING_VIEWPORT_RESIZE: RwLock<>> = RwLock::new(None);
//     static ref PENDING_FRAMETIME_LIMIT_UPDATE: RwLock<Option<Duration>> = RwLock::new(None);
//     static ref PENDING_GRAPHICS_LEVEL_UPDATE: RwLock<Option<GraphicsLevel>> = RwLock::new(None);
//     static ref PENDING_DEMO_SWITCH: RwLock<Option<js_sys::Function>> = RwLock::new(None);
//     // static ref GAME: Mutex<game_of_life::Grid> = Mutex::new(game_of_life::Grid::new(1, 1));
//     //static ref CANVAS: RwLock<web_sys::HtmlCanvasElement> = RwLock::default();
//     //static ref CANVAS_ID: RwLock<String> = RwLock::new("__stub__".to_owned());
// }

#[wasm_bindgen]
pub struct WasmInterface {
    demo_state: Rc<RefCell<ExternalState>>,
    pending_graphics_level: Rc<RefCell<Option<GraphicsLevel>>>,
    pending_demo_id: Rc<RefCell<Option<DemoId>>>,
    canvas: web_sys::HtmlCanvasElement,
    gl: web_sys::WebGl2RenderingContext,
}

#[wasm_bindgen]
#[derive(Copy, Clone)]
pub enum DemoId {
    Triangle,
    CareerHuawei,
    CareerSamsung,
    PublicationWacv2024,
    ProjectTreesRuler,
    ProjectThisCv,
    EducationMasters,
    EducationBachelor,
}

#[wasm_bindgen]
impl WasmInterface {

    #[wasm_bindgen(constructor)]
    pub fn new(canvas_dom_id: &str) -> Result<WasmInterface, JsValue> {
        #[cfg(feature = "console_error_panic_hook")]
        console_error_panic_hook::set_once();
        js_interop::js_log!("WASM Startup");

        let canvas = webgl::canvas(canvas_dom_id)?;
        let gl = webgl::init_webgl_context(&canvas).expect("Failed to get WebGL2 context");
        let demo_state = Rc::new(RefCell::new(renderer::ExternalState::default()));
        Ok(Self {
            demo_state,
            pending_graphics_level: Rc::new(RefCell::new(None)),
            pending_demo_id: Rc::new(RefCell::new(None)),
            canvas,
            gl,
        })
    }

    #[wasm_bindgen]
    pub fn wasm_get_frame_idx(&self) -> usize {
        match self.demo_state.try_borrow() {
            Ok(state) => state.frame_idx,
            _ => Default::default(),
        }
    }

    #[wasm_bindgen]
    pub fn wasm_resize(&mut self, gl: *mut web_sys::WebGl2RenderingContext, width: u32, height: u32) {
        if let Ok(mut state) = self.demo_state.try_borrow_mut() {
            state.screen_size = (width, height);
            web_sys::console::log_3(&"Rust resize".into(), &width.into(), &height.into());
            self.gl.viewport(0, 0, width as i32, height as i32);
        }
    }

    #[wasm_bindgen]
    pub fn wasm_set_fps_limit(&mut self, fps_limit: i32) {
        if let Ok(mut state) = self.demo_state.try_borrow_mut() {
            state.time_delta_limit_ms = 1_000 / fps_limit;
        }
    }

    #[wasm_bindgen]
    pub fn wasm_set_graphics_level(&mut self, level_code: u32) {
        *self.pending_graphics_level.borrow_mut() = Some(GraphicsLevel::from(level_code));
        // if let Ok(state) = self.demo_state.try_borrow_mut() {
        //     state.graphics_level = ;
        // }
    }

    #[wasm_bindgen]
    // , progress_callback: &js_sys::Function
    pub async fn wasm_switch_demo(&mut self, demo_id: DemoId) {
        *self.pending_demo_id.borrow_mut() = Some(demo_id);
    }

    #[wasm_bindgen]
    pub fn wasm_loop(&mut self) -> Result<(), JsValue> {
        // callbacks wired with JS canvas
        // engine callback will schedule timeout callback (to limit fps)
        // timeout callback will schedule engine callback (to render the next frame)
        let engine_cb = Rc::new(RefCell::new(None));
        let engine_cb_clone = engine_cb.clone(); // to have a separate object, which is not owned by tick closure
        let timeout_cb = Rc::new(RefCell::new(None));
        let timeout_cb_clone = timeout_cb.clone();

        {
            let mut demo_state_mut = self.demo_state.borrow_mut();
            demo_state_mut.date = chrono::Utc::now().date_naive(); /* NOTE: set once */
            demo_state_mut.sound_sample_rate = 44100.0; /* NOTE: set once */
            configure_mousedown(&self.canvas, demo_state_mut.mouse.clone())?;
            configure_mouseup(demo_state_mut.mouse.clone())?;
            configure_mousemove(&self.canvas, demo_state_mut.mouse.clone())?;
        }
        let mut gl = self.gl.clone();
        let demo_state = self.demo_state.clone();
        let mut time_then_sec = js_interop::now_sec();
        let mut demo: Box<dyn IDemo> = Box::new(TriangleDemo::new(&gl, self.demo_state.borrow().graphics_level));
        let pending_graphics_level = self.pending_graphics_level.clone();
        let pending_demo_id = self.pending_demo_id.clone();
        // let frametime_limit_ms = demo_state.borrow().deref().d;
        // self.demo.set_graphics_level(let Ok(graphics_level) = elf.graphics_level)try_;.borrow( {}
        *engine_cb_clone.borrow_mut() = Some(Closure::new(move || {
            poll_pending_event(&pending_graphics_level, |&graphics_level| {
                demo.set_graphics_level(graphics_level.to_owned());
                demo_state.borrow_mut().graphics_level = graphics_level;
            });
            poll_pending_event(&pending_demo_id, |&demo_id| {
                demo = renderer::make_demo(demo_id, &gl, demo_state.borrow().graphics_level);
            });

            let time_now_sec = js_interop::now_sec();
            let elapsed_sec = time_now_sec - time_then_sec;
            time_then_sec = time_now_sec;

            // engine step
            let mut demo_state = demo_state.borrow_mut();
            demo_state.begin_frame(elapsed_sec as f32);
            demo.tick(&demo_state);
            demo.render(&mut gl, demo_state.time_delta_sec);
            demo_state.end_frame();

            // Schedule ourself for another requestAnimationFrame callback.
            js_interop::set_frame_timeout(timeout_cb.borrow().as_ref().unwrap(), demo_state.time_delta_limit_ms);
        }));

        *timeout_cb_clone.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            js_interop::request_animation_frame(engine_cb.borrow().as_ref().unwrap());
        }) as Box<dyn FnMut()>));

        js_interop::request_animation_frame(engine_cb_clone.borrow().as_ref().unwrap());

        Ok(())
    }
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

fn poll_pending_event<T, F: FnMut(&T)>(event: &Rc<RefCell<Option<T>>>, mut handler: F) {
    if let Ok(reader) = event.try_borrow() {
        if let Some(v) = reader.as_ref() {
            handler(&v);
            std::mem::drop(reader);
            *event.borrow_mut() = None;
        }
    }
}
