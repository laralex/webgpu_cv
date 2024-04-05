
mod js_interop;
mod webgl;
mod gl_utils;
mod renderer;
mod simple_async;

use renderer::SimpleFuture;
use renderer::{GraphicsLevel, MouseState, ExternalState};
use renderer::{IDemo};

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::WebGl2RenderingContext;
use std::cell::Cell;
use std::{cell::RefCell, ops::Deref, rc::Rc};
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
    demo: Rc<RefCell<Box<dyn IDemo>>>,
    pending_graphics_level: Rc<RefCell<Option<GraphicsLevel>>>,
    pending_demo_id: Rc<RefCell<Option<DemoId>>>,
    canvas: web_sys::HtmlCanvasElement,
    gl: Rc<web_sys::WebGl2RenderingContext>,
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
        let mut demo: Rc<RefCell<Box<dyn IDemo>>> = Rc::new(RefCell::new(Box::new(renderer::StubDemo{})));
        let (executor, spawner) = simple_async::new_executor_and_spawner();
        Ok(Self {
            demo,
            demo_state,
            pending_graphics_level: Rc::new(RefCell::new(None)),
            pending_demo_id: Rc::new(RefCell::new(None)),
            canvas,
            gl: Rc::new(gl),
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
    pub fn wasm_start_loading_demo(&mut self, demo_id: DemoId) {
        //*self.pending_demo_id.borrow_mut() = Some(demo_id);
        
        let loader_callback = Rc::new(RefCell::new(None));
        let loader_callback2 = loader_callback.clone();
        let demo = self.demo.clone();
        let gl = self.gl.clone();
        let mut demo_loading_future =
            renderer::start_loading_demo(demo_id, gl, self.demo_state.borrow().graphics_level);
        *loader_callback2.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            match (demo_loading_future.as_mut()).poll(/*cx*/&mut ()) {
                std::task::Poll::Pending => {
                    // poll again on the next frame
                    js_interop::request_animation_frame(&js_interop::window(), loader_callback.borrow().as_ref().unwrap());
                }
                std::task::Poll::Ready(new_demo) => {
                    *demo.borrow_mut() = new_demo;
                    web_sys::console::log_1(&"Rust ended wasm_start_loading_demo".into());
                }
            }
        }) as Box<dyn FnMut()>));
        js_interop::request_animation_frame(&js_interop::window(), loader_callback2.borrow().as_ref().unwrap());
    }

    #[wasm_bindgen]
    pub fn wasm_loop(&mut self) -> Result<(), JsValue> {
        // callbacks wired with JS canvas
        // engine callback will schedule timeout callback (to limit fps)
        // timeout callback will schedule engine callback (to render the next frame)
        let engine_tick = Rc::new(RefCell::new(None));
        let engine_first_tick = engine_tick.clone(); // to have a separate object, which is not owned by tick closure
        let timeout_tick = Rc::new(RefCell::new(None));
        let timout_into_engine_tick = timeout_tick.clone();

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
        
        let pending_graphics_level = self.pending_graphics_level.clone();
        let pending_demo_id = self.pending_demo_id.clone();
        // let frametime_limit_ms = demo_state.borrow().deref().d;
        // self.demo.set_graphics_level(let Ok(graphics_level) = elf.graphics_level)try_;.borrow( {}
            let demo_clone = self.demo.clone();
            // let set_demo_running_future: BoxFuture<IDemo> = None;
            // let set_demo_pending_future = None;
        let window_engine = js_interop::window();
        *engine_first_tick.borrow_mut() = Some(Closure::new(move || {
            // poll_pending_event(&pending_graphics_level, |&graphics_level| {
            //     demo.lock().unwrap().set_graphics_level(graphics_level.to_owned());
            //     demo_state.borrow_mut().graphics_level = graphics_level;
            // });
            
            // let demo_clone2 = demo_clone.clone();
            // poll_pending_event(&pending_demo_id, |&demo_id| {
            //     spawner.borrow().spawn(async {
                    
            //         *demo_clone2.lock().unwrap() = make_demo_future.into_future().await;
            //     });

            let time_now_sec = js_interop::now_sec();
            let elapsed_sec = time_now_sec - time_then_sec;
            time_then_sec = time_now_sec;

            // engine step
            let mut demo_state = demo_state.borrow_mut();
            if let Ok(mut demo) = demo_clone.try_borrow_mut() {
                demo_state.begin_frame(elapsed_sec as f32);
                demo.tick(&demo_state);
                {
                    gl.viewport(0, 0, demo_state.screen_size.0 as i32, demo_state.screen_size.1 as i32);
                    demo.render(&gl, demo_state.time_delta_sec);
                }
                demo_state.end_frame();
            }

            js_interop::set_frame_timeout(&window_engine, timeout_tick.borrow().as_ref().unwrap(), demo_state.time_delta_limit_ms);
        }));

        let window_timeout = js_interop::window();
        *timout_into_engine_tick.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            js_interop::request_animation_frame(&window_timeout, engine_tick.borrow().as_ref().unwrap());
        }) as Box<dyn FnMut()>));

        let window_first_tick = js_interop::window();
        js_interop::request_animation_frame(&window_first_tick, engine_first_tick.borrow().as_ref().unwrap());
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
