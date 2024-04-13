
mod js_interop;
mod renderer;
use renderer::{DemoLoadingFuture, ExternalState, IDemo, MouseState, Webgpu};

use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;
use std::pin::Pin;
use std::{cell::{RefCell, Cell}, rc::Rc};

#[wasm_bindgen(raw_module = "../modules/exports_to_wasm.js")]
extern "C" {
    fn demo_loading_apply_progress(progress: f32);
    fn demo_loading_finish();
    fn graphics_switching_apply_progress(progress: f32);
    fn graphics_switching_finish();
}

#[wasm_bindgen]
pub struct WasmInterface {
    webgpu: Rc<Webgpu>,
    webgpu_config: Rc<RefCell<wgpu::SurfaceConfiguration>>,
    demo: Rc<RefCell<Box<dyn IDemo>>>,
    demo_state: Rc<RefCell<ExternalState>>,
    demo_id: Rc<RefCell<DemoId>>,
    pending_loading_demo: Rc<RefCell<Option<Pin<Box<dyn DemoLoadingFuture>>>>>,
    // canvas: Option<web_sys::HtmlCanvasElement>,
    // gl: Rc<web_sys::WebGl2RenderingContext>,
}

#[wasm_bindgen]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum DemoId {
    Stub,
    Triangle,
    Fractal,
    FrameGeneration,
    HeadAvatar,
    FullBodyAvatar,
    ProceduralGeneration,
}

#[wasm_bindgen]
#[derive(Default, Clone, Copy)]
pub enum GraphicsLevel {
   Minimal = 0x00,
   Low = 0x10,
   #[default] Medium = 0x20,
   High = 0x30,
   Ultra = 0xFF,
}

#[wasm_bindgen]
impl WasmInterface {

    #[wasm_bindgen(constructor)]
    pub async fn new(canvas_dom_id: &str) -> Result<WasmInterface, JsValue> {
        #[cfg(feature = "console_error_panic_hook")]
        console_error_panic_hook::set_once();
        js_interop::js_log!("WASM Startup");

        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id(canvas_dom_id).unwrap();
        let canvas = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;
        let demo_state = Rc::new(RefCell::new(renderer::ExternalState::default()));
        let pending_loading_demo = Rc::new(RefCell::new(None));
        let demo_id = Rc::new(RefCell::new(DemoId::Stub));

        {
            // callbacks wired with JS canvas
            let mut demo_state_mut = demo_state.borrow_mut();
            demo_state_mut.sound_sample_rate = 44100.0; /* NOTE: set once */
            configure_mousedown(&canvas, demo_state_mut.mouse.clone())?;
            configure_mouseup(demo_state_mut.mouse.clone())?;
            configure_mousemove(&canvas, demo_state_mut.mouse.clone())?;
        }

        let (webgpu, webgpu_config) = Webgpu::new(canvas).await;

        Ok(Self {
            webgpu: Rc::new(webgpu),
            webgpu_config: Rc::new(RefCell::new(webgpu_config)),
            demo: Rc::new(RefCell::new(Box::new(renderer::StubDemo{}))),
            demo_state,
            demo_id,
            pending_loading_demo,
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
    pub fn wasm_resize(&mut self, width: u32, height: u32) {
        if let Ok(mut state) = self.demo_state.try_borrow_mut() {
            state.screen_size = (width, height);
            {
                let mut webgpu_config = self.webgpu_config.borrow_mut();
                webgpu_config.width = width;
                webgpu_config.height = height;
            }
            self.webgpu.surface_configure(&self.webgpu_config.borrow());
        }
    }

    #[wasm_bindgen]
    pub fn wasm_set_fps_limit(&mut self, fps_limit: i32) {
        if let Ok(mut state) = self.demo_state.try_borrow_mut() {
            state.time_delta_limit_ms = 1_000 / fps_limit;
        }
    }

    #[wasm_bindgen]
    pub fn wasm_set_graphics_level(&mut self, level: GraphicsLevel) {
        // NOTE: If the request to switch graphics level is issued,
        // prior to completion of loading of a new demo,
        // then the result is UNDEFINED (probably the new demo will be initialized with previous graphics level)
        // NOTE: If another request to this function is given before
        // `poll_switching_graphics_level` returns Ready enum, 
        // then the current request is aborted and deallocated, and a new one is started
        // NOTE: If the current demo is switched prior to completion of graphics level switch,
        // then the graphics level is switched for the PREVIOUS demo ,
        // and the new demo will be initialized with new graphics level,

        self.demo_state.borrow_mut().graphics_level = level;
        let switcher_callback = Rc::new(RefCell::new(None));
        let switcher_callback2 = switcher_callback.clone();
        self.demo.borrow_mut().as_mut().start_switching_graphics_level(self.webgpu.as_ref(), level);
        let demo_ref = self.demo.clone();
        let webgpu_ref = self.webgpu.clone();

        // request to advance the loading process once per frame
        *switcher_callback.borrow_mut() = Some(Closure::new(move |_: usize| {
            let poll = demo_ref.borrow_mut().poll_switching_graphics_level(webgpu_ref.as_ref());
            match poll {
                Ok(std::task::Poll::Pending) => {
                    graphics_switching_apply_progress(demo_ref.borrow().progress_switching_graphics_level());
                    // run next loading step on the next frame
                    js_interop::request_animation_frame(&js_interop::window(), switcher_callback2.borrow().as_ref().unwrap());
                },
                Ok(std::task::Poll::Ready(())) => {
                    graphics_switching_apply_progress(1.0);
                    // finished switching
                    // don't request another `request_animation_frame`
                    web_sys::console::log_1(&"Rust graphics_switching_finish".into());
                    graphics_switching_finish();
                }
                Err(_) => panic!("Error when switching graphics level")
            }
        }));
        js_interop::request_animation_frame(&js_interop::window(), switcher_callback.borrow().as_ref().unwrap());
    }

    #[wasm_bindgen]
    pub fn wasm_start_loading_demo(&mut self, demo_id: DemoId) {
        let loader_callback = Rc::new(RefCell::new(None));
        let loader_callback2 = loader_callback.clone();
        
        let demo_ref = self.demo.clone();
        let demo_id_ref = self.demo_id.clone();

        // cancel current loading process (drop resources it allocated already)
        demo_loading_finish();
        self.pending_loading_demo.borrow_mut().take();

        if demo_id == *self.demo_id.borrow() {
            // this demo is already fully loaded, don't need to load again
            return;
        }

        // assign new current loading process
        let pending_loading_demo_ref = self.pending_loading_demo.clone();
        *pending_loading_demo_ref.borrow_mut() = Some(
            Box::into_pin(renderer::start_loading_demo(demo_id,
                self.webgpu.clone(),
                self.webgpu_config.borrow().format,
                self.demo_state.borrow().graphics_level)));

        let webgpu_ref = self.webgpu.clone();

        // NOTE: since render loop and loading are not in sync
        // after loading is done there maybe a blank frame
        // thus run finish callback only 2 frames later
        let finish = Closure::new(move |_| demo_loading_finish());
        let finish_after_1_frame = Closure::new(move |_| {
            // wait +1 frame
            js_interop::request_animation_frame(&js_interop::window(), &finish);
        });
        // request to advance the loading proecess once per frame
        *loader_callback.borrow_mut() = Some(Closure::new(move |_: usize| {
            if let Ok(mut loading_process_ref) = pending_loading_demo_ref.try_borrow_mut() {
                if let Some(loading_process) = loading_process_ref.as_mut() {
                    match loading_process.as_mut().simple_poll(/*cx*/&mut ()) {
                        std::task::Poll::Pending => {
                            demo_loading_apply_progress(loading_process.progress());
                            // run next loading step on the next frame
                            js_interop::request_animation_frame(&js_interop::window(), loader_callback2.borrow().as_ref().unwrap());
                        }
                        std::task::Poll::Ready(new_demo) => {
                            // finished loading, assign the global state to new demo
                            demo_loading_apply_progress(loading_process.progress());
                            demo_ref.borrow_mut().drop_demo(webgpu_ref.as_ref());
                            *demo_ref.borrow_mut() = new_demo;
                            *demo_id_ref.borrow_mut() = demo_id;
                            *loading_process_ref = None;

                            // wait +1 frame
                            js_interop::request_animation_frame(&js_interop::window(), &finish_after_1_frame);
                        }
                    }
                } else {
                    // no pending loading
                    // don't request another `request_animation_frame`
                }
            } else {
                // failed to check if loading exists, wait until next frame to try again
                js_interop::request_animation_frame(&js_interop::window(), loader_callback2.borrow().as_ref().unwrap());
            }
        }));
        js_interop::request_animation_frame(&js_interop::window(), loader_callback.borrow().as_ref().unwrap());
    }

    #[wasm_bindgen]
    pub fn wasm_loop(&mut self) -> Result<(), JsValue> {
        // engine callback will schedule timeout callback (to limit fps)
        // timeout callback will schedule engine callback (to render the next frame)
        let engine_tick = Rc::new(RefCell::new(None));
        let engine_first_tick = engine_tick.clone(); // to have a separate object, which is not owned by tick closure
        let window_timeout = js_interop::window();
        let timeout_tick = Closure::wrap(Box::new(move || {
            js_interop::request_animation_frame(&window_timeout, engine_tick.borrow().as_ref().unwrap());
        }) as Box<dyn FnMut()>);

        let webgpu = self.webgpu.clone();
        let webgpu_config = self.webgpu_config.clone();
        let demo_state = self.demo_state.clone();
        let demo_clone = self.demo.clone();
        let window_engine = js_interop::window();
        *engine_first_tick.borrow_mut() = Some(Closure::new(move |timestamp_ms: usize| {
            // engine step
            let webgpu = webgpu.as_ref();
            if let (
                Ok(mut demo), Ok(mut demo_state), Ok(surface_texture)
            ) = (
                demo_clone.try_borrow_mut(), demo_state.try_borrow_mut(), webgpu.surface.get_current_texture()
            ) {
                demo_state.begin_frame(timestamp_ms);
                demo.tick(&demo_state);
                match demo.render(webgpu, &surface_texture, demo_state.time_delta_sec) {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => {
                        webgpu_config.try_borrow()
                            .inspect(|config| webgpu.surface_configure(config));
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => return, // just quit rendering
                    Err(e) => eprintln!("{:?}", e), // resolved by the next frame
                }
                surface_texture.present();
                demo_state.end_frame();
            }
            {
                // setTimeout may overshoot the requested timeout, so compensate it by requesting less 
                const TIMEOUT_CORRECTION_FACTOR: f32 = 0.85;
                let ds = demo_state.borrow();
                let mut request_timeout = ds.time_delta_limit_ms - ds.time_delta_ms as i32;
                request_timeout = (TIMEOUT_CORRECTION_FACTOR * (request_timeout as f32)) as i32;
                js_interop::set_frame_timeout(&window_engine, &timeout_tick, request_timeout);
            }
        }));

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