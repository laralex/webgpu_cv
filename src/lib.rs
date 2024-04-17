mod js_interop;
mod renderer;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Default, Clone, Copy)]
pub enum GraphicsLevel {
   Minimal = 0x00,
   Low = 0x10,
   #[default] Medium = 0x20,
   High = 0x30,
   Ultra = 0xFF,
}

#[cfg(feature = "web")]
mod wasm {

use self::renderer::{ExternalStateData, KeyboardState};

use super::*;
use renderer::{wasm::*, DemoLoadingFuture, ExternalState, IDemo, MouseState, Webgpu};
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
    webgpu_surface: Rc<wgpu::Surface<'static>>,
    webgpu_config: Rc<RefCell<wgpu::SurfaceConfiguration>>,
    demo: Rc<RefCell<Box<dyn IDemo>>>,
    demo_state: Rc<RefCell<ExternalState>>,
    demo_id: Rc<RefCell<DemoId>>,
    pending_loading_demo: Rc<RefCell<Option<Pin<Box<dyn DemoLoadingFuture>>>>>,
    // canvas: Option<web_sys::HtmlCanvasElement>,
    // gl: Rc<web_sys::WebGl2RenderingContext>,
    // demo_state_history: Rc<RefCell<renderer::DemoStateHistory>>, //::new();
    // demo_history_playback: Rc<RefCell<renderer::DemoHistoryPlayback>>, //::new();
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

impl From<u32> for GraphicsLevel {
    fn from(level_code: u32) -> Self {
      match level_code {
         0x00 => GraphicsLevel::Minimal,
         0x10 => GraphicsLevel::Low,
         0x20 => GraphicsLevel::Medium,
         0x30 => GraphicsLevel::High,
         0xFF => GraphicsLevel::Ultra,
         _ => Default::default(),
      }
    }
}

#[wasm_bindgen]
impl WasmInterface {

    #[wasm_bindgen(constructor)]
    pub async fn new(canvas_dom_id: &str, level: GraphicsLevel) -> Result<WasmInterface, JsValue> {
        #[cfg(feature = "console_error_panic_hook")]
        console_error_panic_hook::set_once();
        js_interop::js_log!("WASM Startup");

        demo_loading_apply_progress(0.1);
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id(canvas_dom_id).unwrap();
        let canvas = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;
        let demo_state = Rc::new(RefCell::new(renderer::ExternalState::default()));
        let pending_loading_demo = Rc::new(RefCell::new(None));
        let demo_id = Rc::new(RefCell::new(DemoId::Stub));
        demo_loading_apply_progress(0.4);
        
        {
            // callbacks wired with JS canvas
            let mut demo_state_mut = demo_state.borrow_mut();
            demo_state_mut.set_graphics_level(level);
            configure_mousedown(&canvas, demo_state_mut.mouse().clone())?;
            configure_mouseup(demo_state_mut.mouse().clone())?;
            configure_mousemove(&canvas, demo_state_mut.mouse().clone())?;
            configure_keydown(demo_state_mut.keyboard().clone())?;
            configure_keyup(demo_state_mut.keyboard().clone())?;
        }

        demo_loading_apply_progress(0.6);
        let (webgpu, webgpu_surface, webgpu_config) = Webgpu::new(canvas).await;
        demo_loading_finish();

        Ok(Self {
            webgpu: Rc::new(webgpu),
            webgpu_surface: Rc::new(webgpu_surface),
            webgpu_config: Rc::new(RefCell::new(webgpu_config)),
            demo: Rc::new(RefCell::new(Box::new(renderer::StubDemo{}))),
            demo_state,
            demo_id,
            pending_loading_demo,
            // demo_state_history: Rc::new(RefCell::new(renderer::DemoStateHistory::new())),
            // demo_history_playback: Rc::new(RefCell::new(renderer::DemoHistoryPlayback::new())),
        })
    }

    #[wasm_bindgen(js_name = getFrameIdx)]
    pub fn wasm_get_frame_idx(&self) -> usize {
        match self.demo_state.try_borrow() {
            Ok(state) => state.frame_idx(),
            _ => Default::default(),
        }
    }

    #[wasm_bindgen(js_name = resize)]
    pub fn wasm_resize(&mut self, width: u32, height: u32) {
        let mut demo_state_mut = self.demo_state.borrow_mut();
        demo_state_mut.set_screen_size((width, height));
        {
            let mut webgpu_config = self.webgpu_config.borrow_mut();
            webgpu_config.width = width;
            webgpu_config.height = height;
        }
        self.webgpu_surface.configure(&self.webgpu.device, &self.webgpu_config.borrow())
    }

    #[wasm_bindgen(js_name = setFpsLimit)]
    pub fn wasm_set_fps_limit(&mut self, fps_limit: f64) {
        self.demo_state.borrow_mut().set_time_delta_limit_ms(1_000.0 / fps_limit);
    }

    #[wasm_bindgen(js_name = setDebugMode)]
    pub fn wasm_set_debug_mode(&mut self, debug_mode: Option<u16>) {
        self.demo_state.borrow_mut().set_debug_mode(debug_mode);
    }

    #[wasm_bindgen(js_name = setGraphicsLevel)]
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

        self.demo_state.borrow_mut().set_graphics_level(level);
        let switcher_callback = Rc::new(RefCell::new(None));
        let switcher_callback2 = switcher_callback.clone();
        self.demo.borrow_mut().as_mut().start_switching_graphics_level(self.webgpu.as_ref(), level);
        let demo_ref = self.demo.clone();
        let webgpu_ref = self.webgpu.clone();

        // request to advance the loading process once per frame
        *switcher_callback.borrow_mut() = Some(Closure::new(move |_: f64| {
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
                    web_sys::console::log_1(&"Rust wasm_set_graphics_level".into());
                    graphics_switching_finish();
                }
                Err(_) => panic!("Error wasm_set_graphics_level")
            }
        }));
        js_interop::request_animation_frame(&js_interop::window(), switcher_callback.borrow().as_ref().unwrap());
    }

    #[wasm_bindgen(js_name = startLoadingDemo)]
    pub fn wasm_start_loading_demo(&mut self, demo_id: DemoId) {
        let loader_callback = Rc::new(RefCell::new(None));
        let loader_callback2 = loader_callback.clone();
        
        let demo_ref = self.demo.clone();
        let demo_state_ref = self.demo_state.clone();
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
            Box::into_pin(renderer::wasm::start_loading_demo(demo_id,
                self.webgpu.clone(),
                self.webgpu_config.borrow().format,
                self.demo_state.borrow().graphics_level())));

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
        *loader_callback.borrow_mut() = Some(Closure::new(move |_| {
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

    #[wasm_bindgen(js_name = renderLoop)]
    pub fn wasm_loop(&mut self) -> Result<(), JsValue> {
        // engine callback will schedule timeout callback (to limit fps)
        // timeout callback will schedule engine callback (to render the next frame)
        let engine_tick = Rc::new(RefCell::new(None));
        let engine_tick2 = engine_tick.clone(); // to have a separate object, which is not owned by tick closure
        let window = js_interop::window();
        let timeout_tick = Closure::wrap(Box::new(move || {
            js_interop::request_animation_frame(&window, engine_tick.borrow().as_ref().unwrap());
        }) as Box<dyn FnMut()>);

        let webgpu = self.webgpu.clone();
        let webgpu_surface = self.webgpu_surface.clone();
        let webgpu_config = self.webgpu_config.clone();
        let demo_state = self.demo_state.clone();
        let demo_clone = self.demo.clone();
        // let demo_state_history = self.demo_state_history.clone();
        // let demo_history_playback = self.demo_history_playback.clone();
        let mut demo_state_history = renderer::DemoStateHistory::new();
        let mut demo_history_playback = renderer::DemoHistoryPlayback::new();
        let window = js_interop::window();
        let mut previous_timestamp_ms = 0.0;
        *engine_tick2.borrow_mut() = Some(Closure::new(move |now_timestamp_ms: f64| {
            // engine step
            let webgpu = webgpu.as_ref();
            if let (
                Ok(mut demo), Ok(mut demo_state), Ok(surface_texture),
                // Ok(mut demo_state_history), Ok(mut demo_history_playback),
            ) = (
                demo_clone.try_borrow_mut(), demo_state.try_borrow_mut(), webgpu_surface.get_current_texture(),
                // demo_state_history.try_borrow_mut(), demo_history_playback.try_borrow_mut(),
            ) {
                let keyboard = demo_state.keyboard().get();
                let frame_state = FrameStateRef {
                    demo_state_history: &mut demo_state_history,
                    demo_history_playback: &mut demo_history_playback,
                    demo_state: &mut demo_state,
                    previous_timestamp_ms,
                    now_timestamp_ms,
                };
                handle_keyboard(keyboard, frame_state);

                // engine tick
                let tick_timestamp_ms = demo_history_playback.playback_timestamp_ms().unwrap_or(now_timestamp_ms);
                demo_state.tick(tick_timestamp_ms);
                demo.tick(&demo_state);
                
                // engine render
                match demo.render(webgpu, &surface_texture, demo_state.time_delta_sec()) {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => {
                        webgpu_config.try_borrow()
                            .inspect(|config| webgpu_surface.configure(&webgpu.device, &config));
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => return, // just quit rendering
                    Err(e) => eprintln!("{:?}", e), // resolved by the next frame
                }

                // swap buffers
                surface_texture.present();
                demo_state.dismiss_events();
            }
            {
                // setTimeout may overshoot the requested timeout, so compensate it by requesting less 
                const TIMEOUT_CORRECTION_FACTOR: f64 = 0.85;
                let ds = demo_state.borrow();
                let mut request_timeout = ds.time_delta_limit_ms() - ds.time_delta_ms();
                request_timeout = TIMEOUT_CORRECTION_FACTOR * request_timeout;
                js_interop::set_frame_timeout(&window, &timeout_tick, request_timeout.round() as i32);
            }
            previous_timestamp_ms = now_timestamp_ms;
            if !demo_history_playback.is_playing_back() {
                demo_state_history.store_state((*demo_state.borrow()).data());
            }
        }));

        let window = js_interop::window();
        js_interop::request_animation_frame(&window, engine_tick2.borrow().as_ref().unwrap());
        Ok(())
    }
}

struct FrameStateRef<'a> {
    demo_state_history: &'a mut renderer::DemoStateHistory,
    demo_history_playback: &'a mut renderer::DemoHistoryPlayback,
    demo_state: &'a mut renderer::ExternalState,
    previous_timestamp_ms: f64,
    now_timestamp_ms: f64,
}

fn handle_keyboard<'a>(keyboard: KeyboardState, state: FrameStateRef<'a>) {
    if keyboard.m < 0.0 {
        if state.demo_history_playback.toggle_frame_lock(state.previous_timestamp_ms) == false {
            // canceling frame lock, resume time
            let frame_idx = 0;
            state.demo_state.override_time(state.previous_timestamp_ms, frame_idx);
        }
    }
    if keyboard.comma < 0.0 || keyboard.comma > 0.0 && keyboard.shift {
        state.demo_history_playback.advance_back(&state.demo_state_history);
    }
    if keyboard.dot < 0.0 || keyboard.dot > 0.0 && keyboard.shift {
        state.demo_history_playback.advance_forward(&state.demo_state_history);
    }
}

fn configure_keydown(keyboard_state: Rc<Cell<renderer::KeyboardState>>) -> Result<(), JsValue> {
    let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::KeyboardEvent| {
        // web_sys::console::log_2(&"Keycode".into(), &event.key_code().into());
        if event.default_prevented() {
            return; // Do nothing if the event was already processed
        }
        let mut current_state = keyboard_state.get();
        match event.key_code() {
            77 => current_state.m = 1.0,
            188 => current_state.comma = 1.0,
            190 => current_state.dot = 1.0,
            _ => {},
        }
        current_state.shift = event.shift_key();
        current_state.ctrl = event.ctrl_key();
        current_state.alt = event.alt_key();
        keyboard_state.set(current_state);
        event.prevent_default();
    });
    js_interop::document().add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())?;
    closure.forget();
    Ok(())
}

fn configure_keyup(keyboard_state: Rc<Cell<renderer::KeyboardState>>) -> Result<(), JsValue> {
    let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::KeyboardEvent| {
        if event.default_prevented() {
            return; // Do nothing if the event was already processed
        }
        let mut current_state = keyboard_state.get();
        match event.key_code() {
            77 => current_state.m = -1.0,
            188 => current_state.comma = -1.0,
            190 => current_state.dot = -1.0,
            _ => {},
        }
        current_state.shift = event.shift_key();
        current_state.ctrl = event.ctrl_key();
        current_state.alt = event.alt_key();
        keyboard_state.set(current_state);
        event.prevent_default();
    });
    js_interop::document().add_event_listener_with_callback("keyup", closure.as_ref().unchecked_ref())?;
    closure.forget();
    Ok(())
}

fn configure_mousemove(canvas: &web_sys::HtmlCanvasElement, mouse_state: Rc<Cell<renderer::MouseState>>) -> Result<(), JsValue> {
    let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::MouseEvent| {
        let current_state = mouse_state.get();
        mouse_state.set(MouseState {
            canvas_position_px: (event.offset_x(), event.offset_y()), // NOTE: origin at top-left
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

} // mod wasm