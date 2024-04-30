fn main() {
   cfg_if::cfg_if! {
      if #[cfg(feature = "win")] {
         futures::executor::block_on(win::run());
      }
   }
}

#[cfg(feature = "win")]
mod win {

use std::time::{Instant, SystemTime, UNIX_EPOCH};
use std::rc::Rc;

use imgui::*;
use imgui_wgpu::{Renderer, RendererConfig, Texture, TextureConfig};
use imgui_winit_support::winit::{dpi::LogicalSize, event_loop::EventLoop, window::WindowBuilder};

use my_renderer::renderer::handle_keyboard;
use my_renderer::{DemoId, GraphicsLevel};
use my_renderer::env::log_init;
use my_renderer::renderer::{imgui_web, FrameStateRef, webgpu::Webgpu, stub_demo, fractal, DemoHistoryPlayback, DemoStateHistory, ExternalState, IDemo};
use wgpu::{Color, SurfaceTexture};

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
   demo: Box<dyn IDemo>,
   demo_state: ExternalState,
   demo_id: DemoId,
   demo_state_history: DemoStateHistory,
   demo_history_playback: DemoHistoryPlayback,
   previous_timestamp_ms: f64,
   imgui: imgui::Context,
   imgui_renderer: imgui_wgpu::Renderer,
   imgui_platform: imgui_winit_support::WinitPlatform,
   imgui_last_cursor: Option<Option<imgui::MouseCursor>>,
}

impl<'window> State<'window> {
   #[cfg(feature = "win")]
   async fn new(window: &'window winit::window::Window) -> Self {
      let (webgpu, surface) = Webgpu::new_with_winit(window).await;
      let webgpu = Rc::new(webgpu);
      let mut demo_state = ExternalState::default();
      demo_state.set_graphics_level(GraphicsLevel::Medium);
      demo_state.set_time_delta_limit_ms(1.0);
      demo_state.set_debug_mode(Some(1));
      let color_target_format = surface.config.format;
      let demo = fractal::Demo::start_loading(webgpu.clone(), color_target_format, GraphicsLevel::Medium).await;
     
      // Set up dear imgui
      let (mut imgui, imgui_platform) = imgui_web::init_from_winit(&window);
      let mut imgui_renderer = Renderer::new(&mut imgui, &webgpu.device, &webgpu.queue, RendererConfig {
         texture_format: color_target_format,
         ..Default::default()
      });

      Self {
         window,
         webgpu,
         webgpu_surface: surface.surface,
         webgpu_config: surface.config,
         demo_state,
         demo,
         demo_id: DemoId::Stub,
         demo_state_history: DemoStateHistory::new(),
         demo_history_playback: DemoHistoryPlayback::new(),
         previous_timestamp_ms: 0.0,
         imgui,
         imgui_platform,
         imgui_renderer,
         imgui_last_cursor: None,
     }
   }

   // fn window(&self) -> winit::window::Window { self.window }

   fn update(&mut self, now_timestamp_ms: f64) {
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

      let tick_timestamp_ms = self.demo_history_playback.playback_timestamp_ms().unwrap_or(now_timestamp_ms);
      self.demo_state.tick(tick_timestamp_ms);
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

      self.imgui_renderer
         .render(self.imgui.render(), &self.webgpu.queue, &self.webgpu.device, &mut renderpass)
         .inspect_err(|_| log::warn!("Imgui webgpu renderer failed"));
      std::mem::drop(renderpass);
      self.webgpu.queue.submit(Some(encoder.finish()));
   }

   fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
      let surface_texture = self.webgpu_surface.get_current_texture()?;

      // render demo
      self.demo.render(&self.webgpu, &surface_texture, self.demo_state.time_delta_sec())?;

      // imgui
      self.imgui_platform
         .prepare_frame(self.imgui.io_mut(), &self.window)
         .inspect_err(|_| log::warn!("Imgui winit failed to prepare frame"));

      {
         let ui = self.imgui.new_frame();
         let desired_cursor = ui.mouse_cursor();
         if self.imgui_last_cursor != Some(desired_cursor) {
            self.imgui_last_cursor = Some(desired_cursor);
            self.imgui_platform.prepare_render(ui, &self.window);
         }

         self.demo.render_imgui(&ui);

         // common UI
         ui.show_demo_window(&mut true);
      }

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

use winit::{
      dpi::PhysicalPosition, event::*, event_loop::ControlFlow, keyboard::*
   };
   log_init();
   let event_loop = EventLoop::new()
      .expect("Winit failed to initialize");
   let window = WindowBuilder::new().build(&event_loop).unwrap();
   let mut state = State::new(&window).await;
   let window_ref = &window;
   let time_begin = SystemTime::now();
   event_loop.set_control_flow(ControlFlow::Poll);
   event_loop.run(move |event, elwt| {
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
            state.update(now_timestamp_ms);
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
      state.imgui_platform.handle_event(state.imgui.io_mut(), &window_ref, &event);
   }).expect("Winit failed to start event loop");
}

}