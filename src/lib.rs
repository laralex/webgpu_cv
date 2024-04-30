pub mod js_interop;
pub mod renderer;
pub mod timer;
pub mod env;

#[cfg(feature = "web")]
use wasm_bindgen::prelude::*;

#[cfg_attr(feature = "web", wasm_bindgen)]
#[derive(Default, Clone, Copy)]
pub enum GraphicsLevel {
   Minimal = 0x00,
   Low = 0x10,
   #[default] Medium = 0x20,
   High = 0x30,
   Ultra = 0xFF,
}

#[cfg_attr(feature = "web", wasm_bindgen)]
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

impl AsRef<str> for DemoId {
    #[inline]
    fn as_ref(&self) -> &str {
        match self {
            DemoId::Stub => "Stub",
            DemoId::Triangle => "Triangle",
            DemoId::Fractal => "Mandelbrot Fractal",
            DemoId::FrameGeneration => "Frame Generation",
            DemoId::HeadAvatar => "Head Avatar",
            DemoId::FullBodyAvatar => "Full Body Avatar",
            DemoId::ProceduralGeneration => "Procedural Generation",
        }
    }
}

#[cfg(feature = "web")]
mod wasm {

use crate::env::log_init;
use crate::renderer;
use crate::renderer::{handle_keyboard, FrameStateRef};
use crate::timer::ScopedTimer;

use self::renderer::{DemoLoadingFuture, KeyboardState};

use super::*;
use renderer::{DemoLoadingSimpleFuture, ExternalState, IDemo, MouseState, Webgpu};
use web_sys::Element;
use std::pin::Pin;
use std::{cell::{RefCell, Cell}, rc::Rc};

#[wasm_bindgen(raw_module = "../modules/exports_to_wasm.js")]
extern "C" {
    fn demo_loading_apply_progress(progress: f32);
    fn demo_loading_finish();
    fn graphics_switching_apply_progress(progress: f32);
    fn graphics_switching_finish();
}

#[cfg(feature = "imgui_web")]
#[wasm_bindgen]
struct ImguiInstance {
    context: imgui::Context,
    platform: renderer::imgui::web_platform::WebsysPlatform,
    renderer: imgui_wgpu::Renderer,
    last_cursor: Option<Option<imgui::MouseCursor>>,
}

#[wasm_bindgen]
pub struct WasmInterface {
    #[allow(unused)]
    canvas: web_sys::HtmlCanvasElement,
    webgpu: Rc<Webgpu>,
    webgpu_surface: Rc<wgpu::Surface<'static>>,
    webgpu_config: Rc<RefCell<wgpu::SurfaceConfiguration>>,
    demo: Rc<RefCell<Box<dyn IDemo>>>,
    demo_state: Rc<RefCell<ExternalState>>,
    demo_id: Rc<RefCell<DemoId>>,
    previous_demo: Rc<RefCell<Box<dyn IDemo>>>,
    previous_demo_id: Rc<RefCell<DemoId>>,
    pending_loading_demo: Rc<RefCell<Option<Pin<Box<dyn DemoLoadingFuture>>>>>,
    // canvas: Option<web_sys::HtmlCanvasElement>,
    // gl: Rc<web_sys::WebGl2RenderingContext>,
    // demo_state_history: Rc<RefCell<renderer::DemoStateHistory>>, //::new();
    // demo_history_playback: Rc<RefCell<renderer::DemoHistoryPlayback>>, //::new();
    #[cfg(feature = "imgui_web")]
    imgui: Rc<RefCell<Option<ImguiInstance>>>,
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
    pub async fn new(canvas_dom_id: &str, canvas_parent_element: JsValue, level: GraphicsLevel) -> Result<WasmInterface, JsValue> {
        log_init();
        js_interop::js_log!("WASM Startup");

        let _t = ScopedTimer::new("WasmInterface::new");

        let demo_state = Rc::new(RefCell::new(renderer::ExternalState::default()));
        demo_loading_apply_progress(0.1);

        let pending_loading_demo = Rc::new(RefCell::new(None));
        let (canvas, webgpu, webgpu_surface) = Webgpu::new_with_canvas(
            wgpu::PowerPreference::None,
        ).await;
        demo_loading_apply_progress(0.5);

        {
            // callbacks wired with JS canvas
            canvas.set_id(canvas_dom_id);
            canvas_parent_element.dyn_into::<Element>()
                .expect("Invalid argument canvas_parent_element, must be DOM element")
                .append_child(&canvas)
                .expect("Failed to add canvas to given parent DOM element");

            let mut demo_state_mut = demo_state.borrow_mut();
            demo_state_mut.set_graphics_level(level);
            configure_mousedown(&canvas, demo_state_mut.mouse().clone())?;
            configure_mouseup(demo_state_mut.mouse().clone())?;
            configure_mousemove(&canvas, demo_state_mut.mouse().clone())?;
            configure_keydown(demo_state_mut.keyboard().clone())?;
            configure_keyup(demo_state_mut.keyboard().clone())?;
        }

        demo_loading_apply_progress(0.6);
        demo_loading_finish();

        Ok(Self {
            canvas,
            webgpu: Rc::new(webgpu),
            webgpu_surface: Rc::new(webgpu_surface.surface),
            webgpu_config: Rc::new(RefCell::new(webgpu_surface.config)),
            demo_state,
            demo: Rc::new(RefCell::new(Box::new(renderer::stub_demo::Demo{}))),
            demo_id: Rc::new(RefCell::new(DemoId::Stub)),
            previous_demo: Rc::new(RefCell::new(Box::new(renderer::stub_demo::Demo{}))),
            previous_demo_id: Rc::new(RefCell::new(DemoId::Stub)),
            pending_loading_demo,
            // demo_state_history: Rc::new(RefCell::new(renderer::DemoStateHistory::new())),
            // demo_history_playback: Rc::new(RefCell::new(renderer::DemoHistoryPlayback::new())),
            #[cfg(feature = "imgui_web")]
            imgui: Rc::new(RefCell::new(None)),
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
        self.webgpu_surface.configure(&self.webgpu.device, &self.webgpu_config.borrow());

        #[cfg(feature = "imgui_web")]
        if let Some(imgui) = &*self.imgui.borrow_mut() {
            let hidpi_factor = 1.0;
            let size = web_platform::PhysicalSize::PhysicalSize::new(width as usize, height as usize);
            // TODO: resize
            // imgui.platform.handle_event(
            //     imgui.context.io_mut(),
            //     hidpi_factor, &renderer::imgui::web_platform::WebEvent::Resized(size),
            // )
        }
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
        self.demo.borrow_mut().as_mut()
            .start_switching_graphics_level(self.webgpu.as_ref(), level)
            .expect("WebGPU surface error");
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
        
        let demo_state_ref = self.demo_state.clone();
        let demo_ref = self.demo.clone();
        let demo_id_ref = self.demo_id.clone();
        let previous_demo_ref = self.previous_demo.clone();
        let previous_demo_id_ref = self.previous_demo_id.clone();

        // cancel current loading process (drop resources it allocated already)
        demo_loading_finish();
        self.pending_loading_demo.borrow_mut().take();

        if demo_id == *self.demo_id.borrow() {
            // this demo is already fully loaded, don't need to load again
            return;
        }

        if demo_id == *self.previous_demo_id.borrow() {
            // this demo was the previous demo, and we cached it, so don't need to load it again
            demo_ref.swap(&previous_demo_ref);
            demo_id_ref.swap(&previous_demo_id_ref);
            demo_state_ref.as_ref().borrow_mut().reset();
            return;
        }

        // assign new current loading process
        let pending_loading_demo_ref = self.pending_loading_demo.clone();
        *pending_loading_demo_ref.borrow_mut() = Some(
            Box::into_pin(renderer::wasm::start_loading_demo(demo_id,
                self.webgpu.clone(),
                self.webgpu_config.borrow().format,
                self.demo_state.as_ref().borrow().graphics_level())));

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
                            previous_demo_ref.borrow_mut().drop_demo(webgpu_ref.as_ref());
                            previous_demo_ref.swap(&demo_ref);
                            previous_demo_id_ref.swap(&demo_id_ref);
                            *demo_ref.borrow_mut() = new_demo;
                            *demo_id_ref.borrow_mut() = demo_id;
                            demo_state_ref.borrow_mut().reset();
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
        let mut previous_timestamp_ms = 0.0;
        let window = js_interop::window();

        #[cfg(feature = "imgui_web")] {
            let surface_config = webgpu_config.borrow();
            let (width, height) = (surface_config.width as usize, surface_config.height as usize);
            let hidpi_scale = 1.0;
            let (mut imgui, imgui_platform) = renderer::imgui::init_from_raw(
                renderer::imgui::web_platform::PhysicalSize { width, height },
                hidpi_scale,
            );
            let mut imgui_renderer = imgui_wgpu::Renderer::new(&mut imgui, &webgpu.device, &webgpu.queue, imgui_wgpu::RendererConfig {
                texture_format: self.webgpu_config.borrow().format,
                ..Default::default()
            });
            *self.imgui.borrow_mut() = Some(ImguiInstance{
                context: imgui,
                platform: imgui_platform,
                renderer: imgui_renderer,
                last_cursor: None,
            })
        }
        #[cfg(feature = "imgui_web")]
        let imgui = self.imgui.clone();

        *engine_tick2.borrow_mut() = Some(Closure::new(move |now_timestamp_ms: f64| {
            // engine step
            let webgpu = webgpu.as_ref();
            if let (
                Ok(mut demo), Ok(mut demo_state), Ok(surface_texture),
            ) = (
                demo_clone.try_borrow_mut(), demo_state.try_borrow_mut(), webgpu_surface.get_current_texture(),
            ) {
                {
                    let keyboard = demo_state.keyboard().borrow().clone();
                    let frame_state = FrameStateRef {
                        demo_state_history: &mut demo_state_history,
                        demo_history_playback: &mut demo_history_playback,
                        demo_state: &mut demo_state,
                        previous_timestamp_ms,
                        now_timestamp_ms,
                    };
                    handle_keyboard(keyboard, frame_state);
                }

                // engine tick
                let tick_timestamp_ms = demo_history_playback.playback_timestamp_ms().unwrap_or(now_timestamp_ms);
                demo_state.tick(tick_timestamp_ms);
                demo.tick(&demo_state);
                
                // engine render
                match demo.render(webgpu, &surface_texture, demo_state.time_delta_sec()) {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => {
                        let _ = webgpu_config.try_borrow()
                            .inspect(|config| webgpu_surface.configure(&webgpu.device, &config));
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => return, // just quit rendering
                    Err(e) => eprintln!("{:?}", e), // resolved by the next frame
                }

                #[cfg(feature = "imgui_web")]
                if let Some(imgui) = &mut *imgui.borrow_mut() {
                    WasmInterface::render_imgui(&webgpu, imgui, &surface_texture);
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

    #[cfg(feature = "imgui_web")]
    fn render_imgui(webgpu: &Webgpu, imgui: &mut ImguiInstance, surface_texture: &wgpu::SurfaceTexture) {
        use imgui::*;
        imgui.platform
           .prepare_frame(imgui.context.io_mut());

        let ui = imgui.context.frame();
        let window = ui.window("Hello world");
        window
           .size([300.0, 100.0], Condition::FirstUseEver)
           .build(|| {
              ui.text("Hello world!");
              ui.text("This...is...imgui-rs on WGPU!");
              ui.separator();
              let mouse_pos = ui.io().mouse_pos;
              ui.text(format!(
                    "Mouse Position: ({:.1},{:.1})",
                    mouse_pos[0], mouse_pos[1]
              ));
           });
  
        let window = ui.window("Hello too");
        window
           .size([400.0, 200.0], Condition::FirstUseEver)
           .position([400.0, 200.0], Condition::FirstUseEver)
           .build(|| {
              ui.text(format!("Hello"));
           });
  
        ui.show_demo_window(&mut true);
  
        // submit imgui to webgpu
        let desired_cursor = ui.mouse_cursor();
        if imgui.last_cursor != Some(desired_cursor) {
           imgui.last_cursor = Some(desired_cursor);
           imgui.platform.prepare_render(ui);
        }

        let mut encoder: wgpu::CommandEncoder =
        webgpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let view = surface_texture
           .texture
           .create_view(&wgpu::TextureViewDescriptor::default());
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
           label: None,
           color_attachments: &[Some(wgpu::RenderPassColorAttachment {
              view: &view,
              resolve_target: None,
              ops: wgpu::Operations {
                 load: wgpu::LoadOp::Load,
                 store: wgpu::StoreOp::Store,
              },
           })],
           depth_stencil_attachment: None,
           occlusion_query_set: None,
           timestamp_writes: None,
        });
        imgui.renderer
           .render(imgui.context.render(), &webgpu.queue, &webgpu.device, &mut rpass)
           .inspect_err(|_| log::warn!("Imgui webgpu renderer failed"));
        std::mem::drop(rpass);
        webgpu.queue.submit(Some(encoder.finish()));
     }
}

fn configure_keydown(keyboard_state: Rc<RefCell<renderer::KeyboardState>>) -> Result<(), JsValue> {
    let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::KeyboardEvent| {
        // web_sys::console::log_2(&"Keycode".into(), &event.key_code().into());
        if event.default_prevented() {
            return; // Do nothing if the event was already processed
        }
        let mut current_state = keyboard_state.borrow_mut();
        match event.key_code() {
            77 => current_state.down_m(),
            188 => current_state.down_comma(),
            190 => current_state.down_dot(),
            _ => {},
        }
        current_state.shift = event.shift_key();
        current_state.ctrl = event.ctrl_key();
        current_state.alt = event.alt_key();
        event.prevent_default();
    });
    js_interop::document().add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())?;
    closure.forget();
    Ok(())
}

fn configure_keyup(keyboard_state: Rc<RefCell<renderer::KeyboardState>>) -> Result<(), JsValue> {
    let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::KeyboardEvent| {
        if event.default_prevented() {
            return; // Do nothing if the event was already processed
        }
        let mut current_state = keyboard_state.as_ref().borrow_mut();
        match event.key_code() {
            77 => current_state.up_m(),
            188 => current_state.up_comma(),
            190 => current_state.up_dot(),
            _ => {},
        }
        current_state.shift = event.shift_key();
        current_state.ctrl = event.ctrl_key();
        current_state.alt = event.alt_key();
        event.prevent_default();
    });
    js_interop::document().add_event_listener_with_callback("keyup", closure.as_ref().unchecked_ref())?;
    closure.forget();
    Ok(())
}

fn configure_mousemove(canvas: &web_sys::HtmlCanvasElement, mouse_state: Rc<RefCell<renderer::MouseState>>) -> Result<(), JsValue> {
    let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::MouseEvent| {
        let mut current_state = mouse_state.as_ref().borrow_mut();
        current_state.canvas_position_px = (event.offset_x(), event.offset_y()); // NOTE: origin at top-left
    });
    canvas.add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())?;
    closure.forget();
    Ok(())
}

fn configure_mousedown(canvas: &web_sys::HtmlCanvasElement, mouse_state: Rc<RefCell<renderer::MouseState>>) -> Result<(), JsValue> {
    let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::MouseEvent| {
        let mut current_state = mouse_state.as_ref().borrow_mut();
        match event.button() {
            0 => current_state.down_left(),
            1 => current_state.down_middle(),
            2 => current_state.down_right(),
            _ => {},
        }
    });
    canvas.add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())?;
    closure.forget();
    Ok(())
}

fn configure_mouseup(mouse_state: Rc<RefCell<renderer::MouseState>>) -> Result<(), JsValue> {
    let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::MouseEvent| {
        let mut current_state = mouse_state.as_ref().borrow_mut();
        match event.button() {
            0 => current_state.up_left(),
            1 => current_state.up_middle(),
            2 => current_state.up_right(),
            _ => {},
        }
    });
    js_interop::window().add_event_listener_with_callback("mouseup", closure.as_ref().unchecked_ref())?;
    closure.forget();
    Ok(())
}

} // mod wasm