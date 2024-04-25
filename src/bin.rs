use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use my_renderer::{DemoId, GraphicsLevel};
use my_renderer::env::log_init;
use my_renderer::renderer::{FrameStateRef, webgpu::Webgpu, stub_demo, fractal, DemoHistoryPlayback, DemoStateHistory, ExternalState, IDemo};

struct State<'window> {
   // pub window: winit::window::Window,
   pub webgpu: Rc<Webgpu>,
   pub webgpu_surface: wgpu::Surface<'window>,
   pub webgpu_config: wgpu::SurfaceConfiguration,
   demo: Box<dyn IDemo>,
   demo_state: ExternalState,
   demo_id: DemoId,
   demo_state_history: DemoStateHistory,
   demo_history_playback: DemoHistoryPlayback,
   previous_timestamp_ms: f64,
}

impl<'window> State<'window> {
   #[cfg(feature = "win")]
   async fn new(window: &winit::window::Window) -> Self {
      // let window = Box::new(window);
      let (webgpu, surface) = Webgpu::new_with_winit(window).await;
      let webgpu = Rc::new(webgpu);
      let mut demo_state = ExternalState::default();
      demo_state.set_graphics_level(GraphicsLevel::Medium);
      demo_state.set_time_delta_limit_ms(1.0);
      let color_target_format = surface.config.format;
      let demo = fractal::Demo::start_loading(webgpu.clone(), color_target_format, GraphicsLevel::Medium).await;
      Self {
         webgpu,
         webgpu_surface: surface.surface,
         webgpu_config: surface.config,
         demo_state,
         demo,
         demo_id: DemoId::Stub,
         demo_state_history: DemoStateHistory::new(),
         demo_history_playback: DemoHistoryPlayback::new(),
         previous_timestamp_ms: 0.0,
     }
   }

   // fn window(&self) -> winit::window::Window { self.window }

   fn update(&mut self, now_timestamp_ms: f64) {
      {
         let frame_state = FrameStateRef {
            demo_state_history: &mut self.demo_state_history,
            demo_history_playback: &mut self.demo_history_playback,
            demo_state: &mut self.demo_state,
            previous_timestamp_ms: self.previous_timestamp_ms,
            now_timestamp_ms,
         };
         let keyboard = self.demo_state.keyboard().borrow();
         //handle_keyboard(keyboard, frame_state);
      }

      let tick_timestamp_ms = self.demo_history_playback.playback_timestamp_ms().unwrap_or(now_timestamp_ms);
      self.demo_state.tick(tick_timestamp_ms);
      self.demo.tick(&self.demo_state);

      self.previous_timestamp_ms = now_timestamp_ms;
      if !self.demo_history_playback.is_playing_back() {
         self.demo_state_history.store_state(self.demo_state.data());
      }
   }
   fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
      let surface_texture = self.webgpu_surface.get_current_texture()?;

      self.demo.render(&self.webgpu, &surface_texture, self.demo_state.time_delta_sec())?;

      // swap buffers
      surface_texture.present();
      self.demo_state.dismiss_events();
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
async fn run() {
   use winit::{
      event::*, event_loop::{ControlFlow, EventLoop}, keyboard::*, window::WindowBuilder
   };
   log_init();
   let event_loop = EventLoop::new()
      .expect("Winit failed to initialize");
   let window = WindowBuilder::new().build(&event_loop).unwrap();
   let mut state = State::new(&window).await;
   let window_ref = &window;
   let time_begin = SystemTime::now();
   event_loop.set_control_flow(ControlFlow::Poll);
   event_loop.run(move |event, elwt| match event {
      Event::WindowEvent {
         ref event,
         window_id,
      } if window_id == window_ref.id() => match event {
         WindowEvent::CloseRequested
         | WindowEvent::KeyboardInput {
            event:
                  KeyEvent {
                     state: ElementState::Pressed,
                     logical_key: Key::Named(NamedKey::Escape),
                     ..
                  },
            ..
         } => elwt.exit(),
         WindowEvent::KeyboardInput { event: KeyEvent {
            state: ElementState::Pressed,
            logical_key: Key::Named(NamedKey::Control), .. },
            ..
         } => state.demo_state.keyboard().borrow_mut().ctrl = true,
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
         window_ref.request_redraw();
      },
       _ => {}
   })
   .expect("Winit failed to start event loop");
}

fn main() {
   cfg_if::cfg_if! {
      if #[cfg(feature = "win")] {
         futures::executor::block_on(run());
      }
   }
}