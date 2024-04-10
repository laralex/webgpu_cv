use std::{rc::Rc};

use super::{DemoLoadingFuture, Dispose, ExternalState, GraphicsLevel, IDemo, Progress, SimpleFuture};
use crate::gl_utils;
use web_sys::{WebGl2RenderingContext as GL, WebGlProgram, WebGlShader};

#[derive(Default)]
enum DemoLoadingStage {
   Ready = 0,
   CompileShaders,
   LinkPrograms,
   #[default] DummyWait,
   StartSwitchingGraphicsLevel,
   SwitchGraphicsLevel,
}

struct DemoLoadingProcess {
   stage: DemoLoadingStage,
   stage_percent: f32,
   graphics_level: GraphicsLevel,
   main_program: Option<WebGlProgram>,
   vert_shader: Option<WebGlShader>,
   frag_shader: Option<WebGlShader>,
   gl: Rc<GL>,
   dummy_counter: usize,
   loaded_demo: Option<TriangleDemo>,
}

impl Dispose for DemoLoadingProcess {
   fn dispose(&mut self) {
      match self.stage {
         DemoLoadingStage::Ready => {
            // demo is fully loaded, its lifetime is now separate, 
            // shouldn't free its resources
         },
         _ => {
            self.gl.delete_shader(self.vert_shader.as_ref());
            self.gl.delete_shader(self.frag_shader.as_ref());
            self.gl.delete_program(self.main_program.as_ref());
            self.stage = DemoLoadingStage::Ready;
            self.loaded_demo.take();
            web_sys::console::log_2(&"Rust loading drop: TriangleDemo".into(), &self.stage_percent.into());
         },
      }
   }
}

impl Drop for DemoLoadingProcess {
   fn drop(&mut self) {
      self.dispose();
   }
}

impl Progress for DemoLoadingProcess {
    fn progress(&self) -> f32 {
        self.stage_percent
    }
}

impl SimpleFuture for DemoLoadingProcess {
   type Output = Box<dyn IDemo>;
   type Context = ();

   fn simple_poll(mut self: std::pin::Pin<&mut Self>, _cx: &mut Self::Context) -> std::task::Poll<Self::Output> {
      use DemoLoadingStage::*;
      match self.stage {
         DummyWait => {
            let mut x = 0_i32;
            for i in 0..100000 {
               x = x.saturating_add(i);
            }
            self.stage_percent += 0.007;
            self.dummy_counter += 1;
            if self.stage_percent >= 0.5 {
               self.stage = CompileShaders;
            }
            if self.dummy_counter % 500 == 0 {
               web_sys::console::log_2(&"Rust continue loading: TriangleDemo".into(), &self.stage_percent.into());
            }
            std::task::Poll::Pending
         }
         CompileShaders => {
            let vertex_shader_source = std::include_str!("shaders/no_vao_triangle.vert");
            let fragment_shader_source = std::include_str!("shaders/vertex_color.frag");
            let vertex_shader = gl_utils::compile_shader(
                  &self.gl, GL::VERTEX_SHADER, vertex_shader_source)
               .inspect_err(|err| panic!("Vert shader failed to compile {}", err.as_string().unwrap()))
               .unwrap();
            let fragment_shader = gl_utils::compile_shader(
                  &self.gl,GL::FRAGMENT_SHADER, fragment_shader_source)
               .inspect_err(|err| panic!("Frag shader failed to compile {}", err.as_string().unwrap()))
               .unwrap();
            self.vert_shader = Some(vertex_shader);
            self.frag_shader = Some(fragment_shader);
            self.stage_percent = 0.6;
            self.stage = LinkPrograms;
            std::task::Poll::Pending
         },
         LinkPrograms => {
            let main_program = gl_utils::link_program_vert_frag(
               &self.gl, &self.vert_shader.as_ref().unwrap(), &self.frag_shader.as_ref().unwrap())
               .inspect_err(|err| panic!("Program failed to link {}", err.as_string().unwrap()))
               .unwrap();
            self.main_program = Some(main_program);
            gl_utils::delete_program_shaders(&self.gl, &self.main_program.as_ref().unwrap());
            self.vert_shader = None;
            self.frag_shader = None;
            self.stage_percent = 0.7;
            self.stage = StartSwitchingGraphicsLevel;
            std::task::Poll::Pending
         }
         StartSwitchingGraphicsLevel => {
            self.loaded_demo = Some(TriangleDemo {
               gl: self.gl.clone(),
               main_program: self.main_program.as_ref().unwrap().clone(),
               clear_color: [0.0; 4],
               num_rendered_vertices: 3,
               pending_graphics_level_switch: None,
            });
            let graphics_level = self.graphics_level;
            let gl = self.gl.clone();
            self.loaded_demo.as_mut().unwrap()
                  .start_switching_graphics_level(gl.as_ref(), graphics_level);
            self.stage_percent = 0.75;
            self.stage = SwitchGraphicsLevel;
            std::task::Poll::Pending
         }
         SwitchGraphicsLevel => {
            let gl = self.gl.clone();
            match self.loaded_demo.as_mut().unwrap().poll_switching_graphics_level(gl.as_ref()) {
               std::task::Poll::Pending  => {
                  self.stage_percent = 0.75 + 0.25 * self.loaded_demo.as_ref().unwrap().progress_switching_graphics_level();
                  self.stage = SwitchGraphicsLevel;
                  std::task::Poll::Pending
               }
               std::task::Poll::Ready(()) => {
                  web_sys::console::log_1(&"Rust loading ready: TriangleDemo".into());
                  self.stage_percent = 1.0;
                  self.stage = Ready;
                  std::task::Poll::Ready(Box::new(
                     self.loaded_demo.take().unwrap()
                  ))
               },
            }
         }
         Ready => unreachable!("Should not poll the task again after std::task::Poll::Ready was polled"),
      }
   }
}

impl DemoLoadingFuture for DemoLoadingProcess {}

pub struct TriangleDemo {
   gl: Rc<GL>,
   main_program: WebGlProgram,
   clear_color: [f32; 4],
   num_rendered_vertices: i32,
   pending_graphics_level_switch: Option<GraphicsSwitchingProcess>,
}

impl Drop for TriangleDemo {
   fn drop(&mut self) {
      self.gl.delete_program(Some(&self.main_program));
   }
}

impl TriangleDemo {
   pub fn start_loading<'a>(gl: Rc<GL>, graphics_level: GraphicsLevel) -> Box<dyn DemoLoadingFuture> {
      Box::new(DemoLoadingProcess {
         stage: Default::default(),
         stage_percent: 0.0,
         graphics_level,
         main_program: Default::default(),
         vert_shader: Default::default(),
         frag_shader: Default::default(),
         loaded_demo: Default::default(),
         gl,
         dummy_counter: 0,
      })
   }
}


impl IDemo for TriangleDemo {
   fn tick(&mut self, input: &ExternalState) {
      let mouse_pos = input.mouse_unit_position();
      self.clear_color[0] = input.time_sec.sin() * 0.5 + 0.5 * mouse_pos.0;
      self.clear_color[1] = (input.time_sec * 1.2).sin() * 0.5 + 0.5;
      self.clear_color[2] = input.mouse.get().left;
      self.clear_color[3] = 1.0;
   }

   fn render(&mut self, gl: &GL, _delta_sec: f32) {
      gl.bind_framebuffer(GL::FRAMEBUFFER, None);
      gl_utils::clear_with_color_f32(
         gl, GL::COLOR_ATTACHMENT0, &self.clear_color, 0);
      gl.use_program(Some(&self.main_program));
      gl.draw_arrays(GL::TRIANGLES, 0, self.num_rendered_vertices);
   }

   fn start_switching_graphics_level(&mut self, _gl: &GL, graphics_level: GraphicsLevel) {
      web_sys::console::log_1(&"Rust start_switching_graphics_level: TriangleDemo".into());
      self.pending_graphics_level_switch = Some(GraphicsSwitchingProcess{
         progress: 0.0,
         graphics_level,
      });
   }

   fn poll_switching_graphics_level(&mut self, gl: &GL) -> std::task::Poll<()> {
      if self.pending_graphics_level_switch.is_some() {
         GraphicsSwitchingProcess::poll(self, gl)
      } else {
         self.pending_graphics_level_switch.take();
         std::task::Poll::Ready(())
      }
   }

   fn progress_switching_graphics_level(&self) -> f32 {
      self.pending_graphics_level_switch
         .as_ref()
         .map(|s| s.progress())
         .unwrap_or_default()
   }

   fn drop_demo(&mut self, gl: &GL) {
      gl.delete_program(Some(&self.main_program));
      web_sys::console::log_1(&"Rust demo drop: TriangleDemo".into());
   }
}

pub struct GraphicsSwitchingProcess {
   progress: f32,
   graphics_level: GraphicsLevel,
}

impl Dispose for GraphicsSwitchingProcess {
   fn dispose(&mut self) {
      web_sys::console::log_1(&"Rust graphics switching drop: TriangleDemo".into());
   }
}

impl Drop for GraphicsSwitchingProcess {
   fn drop(&mut self) {
      self.dispose();
   }
}

impl Progress for GraphicsSwitchingProcess {
    fn progress(&self) -> f32 {
      self.progress
   }
}

impl GraphicsSwitchingProcess {
   pub fn poll(demo: &mut TriangleDemo, _gl: &GL) -> std::task::Poll<()> {
      if demo.pending_graphics_level_switch.is_none() {
         return std::task::Poll::Ready(());
      }
      let self_ = demo.pending_graphics_level_switch.as_mut().unwrap();
      let mut x = 0_i32;
      for i in 0..100000 {
         x = x.saturating_add(i);
      }
      self_.progress += 0.005;
      if self_.progress >= 1.0 {
         demo.num_rendered_vertices = match self_.graphics_level {
            GraphicsLevel::Minimal => 0,
            GraphicsLevel::Low => 3,
            GraphicsLevel::Medium => 6,
            GraphicsLevel::High => 9,
            GraphicsLevel::Ultra => 12,
         };
         std::task::Poll::Ready(())
      } else {
         std::task::Poll::Pending
      }
   }
}
