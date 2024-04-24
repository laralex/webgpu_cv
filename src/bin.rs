use std::rc::Rc;
use std::time::Instant;

use my_wasm::env::log_init;
use my_wasm::renderer::{stub_demo, fractal, DemoHistoryPlayback, DemoStateHistory, ExternalState, IDemo};
use my_wasm::DemoId;
// mod env;
// use env::log_init;
use my_wasm::{renderer::FrameStateRef, GraphicsLevel};
use my_wasm::renderer::webgpu::Webgpu;


use winit::{
   event::*, event_loop::{ControlFlow, EventLoop}, keyboard::*, window::WindowBuilder
};

struct State<'window> {
   // pub window: winit::window::Window,
   pub webgpu: Rc<Webgpu<'window>>,
   pub webgpu_config: wgpu::SurfaceConfiguration,
   demo: Box<dyn IDemo>,
   demo_state: ExternalState,
   demo_id: DemoId,
   demo_state_history: DemoStateHistory,
   demo_history_playback: DemoHistoryPlayback,
   previous_timestamp_ms: f64,
}

impl<'window> State<'window> {
   async fn new(window: &'window winit::window::Window) -> Self {
      // let window = Box::new(window);
      let (webgpu, webgpu_config) = Webgpu::new_with_winit(&window).await;
      let webgpu = Rc::new(webgpu);
      let mut demo_state = ExternalState::default();
      demo_state.set_graphics_level(GraphicsLevel::Medium);
      let color_target_format = webgpu_config.format;
      let demo = fractal::Demo::start_loading(webgpu.clone(), color_target_format, GraphicsLevel::Medium).await;
      Self {
         webgpu,
         webgpu_config,
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
      let keyboard = self.demo_state.keyboard().get();
      let frame_state = FrameStateRef {
         demo_state_history: &mut self.demo_state_history,
         demo_history_playback: &mut self.demo_history_playback,
         demo_state: &mut self.demo_state,
         previous_timestamp_ms: self.previous_timestamp_ms,
         now_timestamp_ms,
      };
      //handle_keyboard(keyboard, frame_state);

      let tick_timestamp_ms = self.demo_history_playback.playback_timestamp_ms().unwrap_or(now_timestamp_ms);
      self.demo_state.tick(tick_timestamp_ms);
      self.demo.tick(&self.demo_state);

      self.previous_timestamp_ms = now_timestamp_ms;
      if !self.demo_history_playback.is_playing_back() {
         self.demo_state_history.store_state(self.demo_state.data());
      }
   }
   fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
      let surface_texture = self.webgpu.surface.get_current_texture()?;

      self.demo.render(&self.webgpu, &surface_texture, self.demo_state.time_delta_sec())?;

      // swap buffers
      surface_texture.present();
      self.demo_state.dismiss_events();
      Ok(())
   }
   fn reconfigure(&mut self) {
      self.webgpu.surface.configure(&self.webgpu.device, &self.webgpu_config);
   }
   pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
      if new_size.width > 0 && new_size.height > 0 {
          self.webgpu_config.width = new_size.width;
          self.webgpu_config.height = new_size.height;
          self.webgpu.surface.configure(&self.webgpu.device, &self.webgpu_config);
      }
  }
}

async fn run() {
   log_init();
   let event_loop = EventLoop::new()
      .expect("Winit failed to initialize");
   let window = WindowBuilder::new().build(&event_loop).unwrap();
   let mut state = State::new(&window).await;
   let window_ref = &window;
   event_loop.set_control_flow(ControlFlow::Poll);
   let mut previous_instant = Instant::now();
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
         WindowEvent::Resized(physical_size) => {
            state.resize(*physical_size);
         }
         WindowEvent::ScaleFactorChanged { inner_size_writer, .. } => {
            // new_inner_size is &&mut so we have to dereference it twice
            // inner_size_writer.w
            // state.resize(**new_inner_size);
         }
         _ => {}
      },
      Event::AboutToWait => {
         let now_instant = Instant::now();
         let now_timestamp_ms = now_instant.elapsed().as_nanos() as f64 * 0.001;
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
   futures::executor::block_on(run());
}