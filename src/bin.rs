fn main() {
   cfg_if::cfg_if!(
      if #[cfg(feature = "win")] {
         tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async { win::run().await; })
     }
   );
}

#[cfg(feature = "win")]
mod win {

use std::cell::RefCell;
use std::task::Poll;
use std::time::SystemTime;
use std::rc::Rc;

use imgui::*;
use imgui_wgpu::{Renderer, RendererConfig};
use imgui_winit_support::winit::{event_loop::EventLoop, window::WindowBuilder};

use my_renderer::renderer::{GlobalUniform, LoadingArgs, RenderArgs};
use my_renderer::renderer::{handle_keyboard, demo_uv, imgui_web, FrameStateRef, webgpu::Webgpu, demo_stub, demo_fractal, DemoHistoryPlayback, DemoStateHistory, ExternalState, IDemo};
use my_renderer::{DemoId, GraphicsLevel};
use my_renderer::env::log_init;

// Set up texture
// let lenna_bytes = include_bytes!("../resources/checker.png");
// let image =
//    image::load_from_memory_with_format(lenna_bytes, ImageFormat::Png).expect("invalid image");
// let image = image.to_rgba8();
// let (width, height) = image.dimensions();
// let raw_data = image.into_raw();

// let texture_config = TextureConfig {
//    size: Extent3d {
//          width,
//          height,
//          ..Default::default()
//    },
//    label: Some("lenna texture"),
//    format: Some(wgpu::TextureFormat::Rgba8Unorm),
//    ..Default::default()
// };

// let texture = Texture::new(&device, &renderer, texture_config);

// texture.write(&queue, &raw_data, width, height);
// let lenna_texture_id = renderer.textures.insert(texture);
      

struct State<'window> {
   pub window: &'window winit::window::Window,
   pub webgpu: Rc<Webgpu>,
   pub webgpu_surface: wgpu::Surface<'window>,
   pub webgpu_config: wgpu::SurfaceConfiguration,
   global_uniform: Rc<RefCell<GlobalUniform>>,
   demo: Box<dyn IDemo>,
   demo_state: ExternalState,
   demo_state_history: DemoStateHistory,
   demo_history_playback: DemoHistoryPlayback,
   previous_timestamp_ms: f64,
   imgui: Option<imgui::Context>,
   imgui_renderer: imgui_wgpu::Renderer,
   imgui_platform: imgui_winit_support::WinitPlatform,
   imgui_last_cursor: Option<Option<imgui::MouseCursor>>,
   imgui_exports: ImguiExports,
   // demos_names: [&str; 3],
   demos_ids: [&'static DemoId; 3],
   demo_idx: i32,
}

#[derive(Default)]
struct ImguiExports {
   graphics_level_idx: usize,
   debug_mode: i32,
   debug_mode_enabled: bool,
   playback_enabled: bool,
   screen_override_enabled: bool,
   screen_override: [u32; 2],
   screen_backup: [u32; 2],
}

const GRAPHICS_LEVELS: [GraphicsLevel; 5] = [
   GraphicsLevel::Minimal, GraphicsLevel::Low, GraphicsLevel::Medium,
   GraphicsLevel::High, GraphicsLevel::Ultra];

impl<'window> State<'window> {
   async fn load_demo(&mut self, id: DemoId) -> Box<dyn IDemo> {
      let loading_args = LoadingArgs {
         webgpu: self.webgpu.clone(),
         global_uniform: self.global_uniform.clone(),
         color_texture_format: self.webgpu_config.format,
      };
      let loader = match id {
        DemoId::Stub => demo_stub::Demo::start_loading(),
        DemoId::Uv => demo_uv::Demo::start_loading(loading_args, self.demo_state.graphics_level()),
        DemoId::Fractal => demo_fractal::Demo::start_loading(loading_args, self.demo_state.graphics_level()),
        _ => demo_stub::Demo::start_loading(),
      //   DemoId::FrameGeneration => todo!(),
      //   DemoId::HeadAvatar => todo!(),
      //   DemoId::FullBodyAvatar => todo!(),
      //   DemoId::ProceduralGeneration => todo!(),
      };
      loader.await
   }
   #[cfg(feature = "win")]
   async fn new(window: &'window winit::window::Window) -> Self {
      let (webgpu, surface) = Webgpu::new_with_winit(window).await;
      let webgpu = Rc::new(webgpu);
      let mut demo_state = ExternalState::default();
      demo_state.set_graphics_level(GraphicsLevel::Medium);
      demo_state.set_time_delta_limit_ms(1.0);
      demo_state.set_debug_mode(Some(1));
      let global_uniform = Rc::new(RefCell::new(GlobalUniform::new(&webgpu.device)));
      let loading_args = LoadingArgs {
         webgpu: webgpu.clone(),
         global_uniform: global_uniform.clone(),
         color_texture_format: surface.config.format,
      };
      let demo = demo_fractal::Demo::start_loading(loading_args, demo_state.graphics_level()).await;
      let demo_idx = 0 as i32;
      let demos_ids = [
         &DemoId::Fractal,
         &DemoId::Uv,
         &DemoId::Stub,
      ];
      // Set up dear imgui
      let (mut imgui, imgui_platform) = imgui_web::init_from_winit(&window);
      let imgui_renderer = Renderer::new(&mut imgui, &webgpu.device, &webgpu.queue, RendererConfig {
         texture_format: surface.config.format,
         ..Default::default()
      });

      let graphics_level_idx = GRAPHICS_LEVELS.iter().enumerate()
         .filter_map(|(i, level)| (*level == demo_state.graphics_level()).then_some(i))
         .take(1)
         .next().unwrap_or(0);
      let imgui_exports = ImguiExports{
         graphics_level_idx: graphics_level_idx,
         debug_mode: demo_state.debug_mode().unwrap_or_default() as i32,
         debug_mode_enabled: demo_state.debug_mode().is_some(),
         ..Default::default()
      };
      Self {
         window,
         webgpu,
         webgpu_surface: surface.surface,
         webgpu_config: surface.config,
         global_uniform,
         demo_state,
         demo,
         demo_idx,
         demos_ids,
         demo_state_history: DemoStateHistory::new(),
         demo_history_playback: DemoHistoryPlayback::new(),
         previous_timestamp_ms: 0.0,
         imgui: Some(imgui),
         imgui_platform,
         imgui_renderer,
         imgui_last_cursor: None,
         imgui_exports,
     }
   }

   // fn window(&self) -> winit::window::Window { self.window }

   fn tick(&mut self, now_timestamp_ms: f64) {
      {
         let keyboard = self.demo_state.keyboard().borrow().clone();
         let frame_state = FrameStateRef {
            demo_state_history: &mut self.demo_state_history,
            demo_history_playback: &mut self.demo_history_playback,
            demo_state: &mut self.demo_state,
            previous_timestamp_ms: self.previous_timestamp_ms,
            now_timestamp_ms,
         };
         handle_keyboard(keyboard, frame_state);
      }

      self.tick_imgui(now_timestamp_ms);
      let tick_timestamp_ms = self.demo_history_playback.playback_timestamp_ms().unwrap_or(now_timestamp_ms);
      self.demo_state.tick(tick_timestamp_ms);
      self.global_uniform.borrow_mut().update_cpu(&self.demo_state);
      self.global_uniform.borrow_mut().update_gpu(&self.webgpu.queue);
      self.demo.tick(&self.demo_state);
      self.previous_timestamp_ms = now_timestamp_ms;
   }

   fn frame_cleanup(&mut self) {
      if !self.demo_history_playback.is_playing_back() {
         self.demo_state_history.store_state(self.demo_state.data());
      }
      self.demo_state.dismiss_events();
   }

   fn render_imgui(&mut self, surface_texture: &wgpu::SurfaceTexture) {
      // submit imgui to webgpu
      let mut encoder: wgpu::CommandEncoder =
      self.webgpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

      let view = surface_texture
         .texture
         .create_view(&wgpu::TextureViewDescriptor::default());
      let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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

      let _ = self.imgui_renderer
         .render(self.imgui.as_mut().unwrap().render(), &self.webgpu.queue, &self.webgpu.device, &mut renderpass)
         .inspect_err(|_| log::warn!("Imgui webgpu renderer failed"));
      std::mem::drop(renderpass);
      self.webgpu.queue.submit(Some(encoder.finish()));
   }
   
   fn swtich_graphics_level(&mut self, level: GraphicsLevel) {
      self.demo_state.set_graphics_level(level);
      let loading_args = LoadingArgs {
         webgpu: self.webgpu.clone(),
         global_uniform: self.global_uniform.clone(),
         color_texture_format: self.webgpu_config.format,
      };
      self.demo.start_switching_graphics_level(loading_args, level)
         .expect("Failed to start switching graphics level");
      loop {
         match self.demo.poll_switching_graphics_level(&self.webgpu) {
            Ok(Poll::Ready(())) => break,
            Ok(Poll::Pending) => log::info!("Switching graphics level to {}", level.as_ref()),
            _ => {
               log::error!("Error while switching graphics level to {}", level.as_ref());
               break;
            }
         }
      }
   }

   fn tick_imgui(&mut self, now_timestamp_ms: f64) {
      // imgui capture updates
      let _ = self.imgui_platform
         .prepare_frame(self.imgui.as_mut().unwrap().io_mut(), &self.window)
         .inspect_err(|_| log::warn!("Imgui winit failed to prepare frame"));

      let mut imgui_local = self.imgui.take().unwrap();
      let ui = imgui_local.new_frame();
      let desired_cursor = ui.mouse_cursor();
      if self.imgui_last_cursor != Some(desired_cursor) {
         self.imgui_last_cursor = Some(desired_cursor);
         self.imgui_platform.prepare_render(ui, &self.window);
      }

      let imgui_common_args = imgui_web::ImguiRenderArgs {
         position: [10.0, 10.0],
         size: [350.0, 350.0],
      };
      let imgui_demo_args = imgui_web::ImguiRenderArgs::new_right_from(
         &imgui_common_args, [10.0, 0.0]);

      self.demo.render_imgui(&ui, imgui_demo_args);

      // common UI
      let window = ui.window("Common");
      window
         .size(imgui_common_args.size, Condition::FirstUseEver)
         .position(imgui_common_args.position, Condition::FirstUseEver)
         .movable(false)
         .build(|| {
         if ui.list_box("Demo",&mut self.demo_idx,
         &self.demos_ids, self.demos_ids.len() as i32) {
            self.demo = futures::executor::block_on(
               self.load_demo(*self.demos_ids[self.demo_idx as usize]));
         }

         if ui.combo("Graphics level", &mut self.imgui_exports.graphics_level_idx, 
            &GRAPHICS_LEVELS, |level| level.as_ref().into()) {
            self.swtich_graphics_level(GRAPHICS_LEVELS[self.imgui_exports.graphics_level_idx])
         }

         ui.separator();
         let mut upd_debug_mode = false;
         upd_debug_mode |= ui.checkbox("Enable debug mode", &mut self.imgui_exports.debug_mode_enabled);
         upd_debug_mode |= ui.input_int("Debug mode", &mut self.imgui_exports.debug_mode).step_fast(0).build();
         if upd_debug_mode {
            self.demo_state.set_debug_mode(self.imgui_exports.debug_mode_enabled
               .then_some(self.imgui_exports.debug_mode as u16));
         }
         if ui.button("Recompile shaders") {
            let loading_args = LoadingArgs {
               webgpu: self.webgpu.clone(),
               global_uniform: self.global_uniform.clone(),
               color_texture_format: self.webgpu_config.format,
            };
            self.demo.rebuild_pipelines(loading_args);
         }

         ui.separator();
         if ui.checkbox("Enable frame playback", &mut self.imgui_exports.playback_enabled) {
            if self.imgui_exports.playback_enabled {
               self.demo_history_playback.start_playback(
                  now_timestamp_ms, self.demo_state.time_now_ms())
            } else {
               if let Some((_, resume_time_ms)) = self.demo_history_playback.cancel_playback() {
                  self.demo_state_history.reset_history();
                  let frame_idx = 0;
                  self.demo_state.override_time(now_timestamp_ms, resume_time_ms, frame_idx);
               }
            }
         }
         let mut play_back = false;
         ui.same_line();
         play_back |= ui.button("<<") || ui.is_item_active();
         ui.same_line();
         play_back |= ui.button("<");
         if play_back {
            self.demo_history_playback.play_back(&self.demo_state_history);
         }
         let mut play_forward = false;
         ui.same_line();
         play_forward |= ui.button(">");
         ui.same_line();
         play_forward |= ui.button(">>") || ui.is_item_active();
         if play_forward{
            self.demo_history_playback.play_forward(&self.demo_state_history);
         }
         ui.separator();
         if ui.checkbox("Screen size override", &mut self.imgui_exports.screen_override_enabled) {
            if self.imgui_exports.screen_override_enabled {
               let (w, h) = self.demo_state.screen_size();
               self.imgui_exports.screen_backup = [w, h];
               self.imgui_exports.screen_override = self.imgui_exports.screen_backup;
            } else {
               self.demo_state.set_screen_size(
                  (self.imgui_exports.screen_backup[0], self.imgui_exports.screen_backup[1]));
            }
         }
         if self.imgui_exports.screen_override_enabled &&
            imgui::Drag::new("W/H")
            .range(100, 3000)
            .speed(2.0)
            .build_array(ui,&mut self.imgui_exports.screen_override) {
            self.demo_state.set_screen_size(
               (self.imgui_exports.screen_override[0], self.imgui_exports.screen_override[1]));
         }
      });
      self.imgui = Some(imgui_local);
   }

   fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
      let surface_texture = self.webgpu_surface.get_current_texture()?;

      // render demo
      self.demo.render(RenderArgs{
         webgpu: &self.webgpu,
         backbuffer: &surface_texture,
         global_uniform: &self.global_uniform.borrow(),
         time_delta_sec: self.demo_state.time_delta_sec()})?;
      // render imgui
      self.render_imgui(&surface_texture);

      // swap buffers
      surface_texture.present();
      Ok(())
   }

   fn reconfigure(&mut self) {
      self.webgpu_surface.configure(&self.webgpu.device, &self.webgpu_config);
   }

   pub fn resize(&mut self, (width, height): (u32, u32)) {
      if width > 0 && height > 0 {
         self.demo_state.set_screen_size((width, height));
          self.webgpu_config.width = width;
          self.webgpu_config.height = height;
          self.webgpu_surface.configure(&self.webgpu.device, &self.webgpu_config);
      }
   }
   pub fn resize_factor(&mut self, scale_factor: f64) {
      self.resize((
         (self.webgpu_config.width as f64 * scale_factor) as u32,
         (self.webgpu_config.height as f64 * scale_factor) as u32,
      ));
   }
}

#[cfg(feature = "win")]
pub async fn run() {
   use std::cmp::Ordering;

   use winit::{event::*, event_loop::ControlFlow, keyboard::*};
   log_init();
   let event_loop = EventLoop::new()
      .expect("Winit failed to initialize");
   let window = WindowBuilder::new()
      .with_inner_size(winit::dpi::PhysicalSize::new(800, 600))
      // .with_maximized(true)
      .with_title("WebGPU demos for Alexey Larionov's web CV")
      .build(&event_loop).unwrap();
   let mut state = State::new(&window).await;
   let window_ref = &window;
   let time_begin = SystemTime::now();
   event_loop.set_control_flow(ControlFlow::Poll);
   event_loop.run(move |event, elwt| {
      state.imgui_platform.handle_event(state.imgui.as_mut().unwrap().io_mut(), &window_ref, &event);
      match event {
         Event::WindowEvent {
            ref event,
            window_id,
         } if window_id == window_ref.id() => match event {
            WindowEvent::CloseRequested => elwt.exit(),
            WindowEvent::KeyboardInput { event: KeyEvent {
               state: press_state,
               logical_key, physical_key, .. },
               ..
            } => {
               let is_pressed = ElementState::is_pressed(*press_state);
               let press_value = if is_pressed { 1.0 } else { -1.0 };
               let mut keyboard = state.demo_state.keyboard().borrow_mut();
               match (logical_key, physical_key) {
                  (Key::Named(NamedKey::Escape), _) => elwt.exit(),
                  (Key::Named(NamedKey::Control), _) => keyboard.ctrl = is_pressed,
                  (Key::Named(NamedKey::Shift), _) => keyboard.shift = is_pressed,
                  (Key::Named(NamedKey::Alt), _) => keyboard.alt = is_pressed,
                  (_, PhysicalKey::Code(KeyCode::KeyM)) => keyboard.set_m(press_value),
                  (_, PhysicalKey::Code(KeyCode::Comma)) => keyboard.set_comma(press_value),
                  (_, PhysicalKey::Code(KeyCode::Period)) => keyboard.set_dot(press_value),
                  (_, PhysicalKey::Code(KeyCode::Backquote)) => keyboard.set_backquote(press_value),
                  (_, PhysicalKey::Code(KeyCode::BracketLeft)) => keyboard.set_bracket_left(press_value),
                  (_, PhysicalKey::Code(KeyCode::BracketRight)) => keyboard.set_bracket_right(press_value),
                  (_, PhysicalKey::Code(digit)) if *digit >= KeyCode::Digit0 && *digit <= KeyCode::Digit9
                     => keyboard.set_digit(*digit as usize - KeyCode::Digit0 as usize, press_value),
                  _ => {},
               }
            },
            WindowEvent::MouseInput { state: press_state, button, .. } => {
               let is_pressed = ElementState::is_pressed(*press_state);
               let press_value = if is_pressed { 1.0 } else { -1.0 };
               let mut mouse = state.demo_state.mouse().borrow_mut();
               match button {
                  MouseButton::Left => { mouse.set_left(press_value); },
                  MouseButton::Middle => { mouse.set_middle(press_value); },
                  MouseButton::Right => { mouse.set_right(press_value); },
                  _ => {},
               }
            },
            WindowEvent::MouseWheel { delta, phase: TouchPhase::Moved, .. } => match delta {
               MouseScrollDelta::LineDelta(to_right, to_bottom) => {
                  let to_up = (-to_bottom).clamp(-1.0, 1.0); // TODO: maybe need to divide by some min/max
                  let to_right = (to_right).clamp(-1.0, 1.0); // TODO: maybe need to divide by some min/max
                  state.demo_state.mouse().borrow_mut().wheel = (to_right, to_up);
               },
               MouseScrollDelta::PixelDelta(pos) => {
                  let pos = pos.to_logical::<f64>(window_ref.scale_factor());
                  let h = match pos.x.partial_cmp(&0.0) {
                        Some(Ordering::Greater) => 1.0,
                        Some(Ordering::Less) => -1.0,
                        _ => 0.0,
                  };
                  let v = match pos.y.partial_cmp(&0.0) {
                        Some(Ordering::Greater) => -1.0,
                        Some(Ordering::Less) => 1.0,
                        _ => 0.0,
                  };
                  state.demo_state.mouse().borrow_mut().wheel = (h, v);
               },
            },
            WindowEvent::CursorMoved { position, .. } => {
               state.demo_state.mouse().borrow_mut().canvas_position_px = (position.x as i32, position.y as i32);
            },
            WindowEvent::Resized(physical_size) => {
               state.resize((physical_size.width, physical_size.height));
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
               state.resize_factor(*scale_factor);
            }
            _ => {}
         },
         Event::AboutToWait => {
            let now_timestamp_ms = SystemTime::now()
               .duration_since(time_begin)
               .unwrap()
               .as_micros() as f64 * 0.001;
            state.tick(now_timestamp_ms);
            match state.render() {
               Ok(_) => {}
               // Reconfigure the surface if lost
               Err(wgpu::SurfaceError::Lost) => {
                  state.reconfigure();
               },
               // The system is out of memory, we should probably quit
               Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
               // All other errors (Outdated, Timeout) should be resolved by the next frame
               Err(e) => eprintln!("{:?}", e),
            }

            state.frame_cleanup();
            window_ref.request_redraw();
         },
         _ => {}
      };
   }).expect("Winit failed to start event loop");
}

}