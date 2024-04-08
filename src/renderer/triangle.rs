use std::{cell::RefCell, pin::Pin, rc::{Rc, Weak}};

use super::{DemoLoadingFuture, Dispose, ExternalState, GraphicsLevel, GraphicsSwitchingFuture, IDemo, MouseState, Progress, SimpleFuture};
use crate::gl_utils;
use web_sys::{WebGl2RenderingContext as GL, WebGlProgram, WebGlShader};

pub struct TriangleDemo {
   gl: Rc<GL>,
   main_program: WebGlProgram,
   clear_color: [f32; 4],
   num_rendered_vertices: i32,
}

impl Drop for TriangleDemo {
   fn drop(&mut self) {
      // if Rc::strong_count(&self.main_program) == 1 {
         self.gl.delete_program(Some(&self.main_program));
      // }
   }
}

impl Clone for TriangleDemo {
    fn clone(&self) -> Self {
        Self {
         gl: self.gl.clone(),
         main_program: self.main_program.clone(),
         clear_color: self.clear_color.clone(),
         num_rendered_vertices: self.num_rendered_vertices.clone(),
        }
    }
}

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
   graphics_switching: Option<Pin<Box<dyn GraphicsSwitchingFuture>>>,
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
            self.graphics_switching.take();
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

   fn simple_poll(mut self: std::pin::Pin<&mut Self>, cx: &mut Self::Context) -> std::task::Poll<Self::Output> {
      use DemoLoadingStage::*;
      match self.stage {
         DummyWait => {
            let mut x = 0_i32;
            for i in 0..100000 {
               x = x.saturating_add(i);
            }
            self.stage_percent += 0.005;
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
            });
            let graphics_level = self.graphics_level;
            let gl = self.gl.clone();
            self.graphics_switching = Some(
               Box::into_pin(self.loaded_demo.as_mut().unwrap()
                  .start_switching_graphics_level(gl, graphics_level))
            );
            self.stage_percent = 0.75;
            self.stage = SwitchGraphicsLevel;
            std::task::Poll::Pending
         }
         SwitchGraphicsLevel => {
            let mut still_switching = false;
            let mut switching_progress = 0.0;
            if let Some(switching_process) = &mut self.graphics_switching {
               match switching_process.as_mut().simple_poll(/*cx*/&mut ()) {
                  std::task::Poll::Pending  => {
                     still_switching = true;
                     switching_progress = switching_process.progress();
                  }
                  std::task::Poll::Ready(new_demo) => {
                     self.loaded_demo = Some(new_demo);
                  }
               }
            }
            if still_switching {
               self.stage_percent = 0.75 + 0.25*switching_progress;
               self.stage = SwitchGraphicsLevel;
               std::task::Poll::Pending
            } else {
               web_sys::console::log_1(&"Rust loading ready: TriangleDemo".into());
               self.stage_percent = 1.0;
               self.stage = Ready;
               std::task::Poll::Ready(Box::new(
                  self.loaded_demo.take().unwrap()
               ))
            }
         }
         Ready => unreachable!("Should not poll the task again after std::task::Poll::Ready was polled"),
      }
   }
}

impl DemoLoadingFuture for DemoLoadingProcess {}

impl TriangleDemo {
   pub fn start_loading<'a>(gl: Rc<GL>, graphics_level: GraphicsLevel) -> impl DemoLoadingFuture {
      DemoLoadingProcess {
         stage: Default::default(),
         stage_percent: 0.0,
         graphics_level,
         main_program: Default::default(),
         vert_shader: Default::default(),
         frag_shader: Default::default(),
         graphics_switching: Default::default(),
         loaded_demo: Default::default(),
         gl,
         dummy_counter: 0,
      }
   }
}


impl IDemo for TriangleDemo {
   fn tick(&mut self, input: &ExternalState) {
      let mouse_pos = input.mouse_unit_position();
      self.clear_color[0] = input.time_sec.sin() * 0.5 + 0.5;
      self.clear_color[1] = (input.time_sec * 1.2).sin() * 0.5 + 0.5;
      self.clear_color[2] = input.mouse.get().left;
      self.clear_color[3] = 1.0;
   }

   fn render(&mut self, gl: &GL, delta_sec: f32) {
      gl.bind_framebuffer(GL::FRAMEBUFFER, None);
      gl_utils::clear_with_color_f32(
         gl, GL::COLOR_ATTACHMENT0, &self.clear_color, 0);
      gl.use_program(Some(&self.main_program));
      gl.draw_arrays(GL::TRIANGLES, 0, self.num_rendered_vertices);
   }

   fn start_switching_graphics_level(&mut self, gl: Rc<GL>, graphics_level: GraphicsLevel) -> Box<dyn GraphicsSwitchingFuture> {
      web_sys::console::log_1(&"Rust start_switching_graphics_level: TriangleDemo".into());
      Box::new(GraphicsSwitchingProcess{
         progress: 0.0,
         demo: self.clone(),
         graphics_level,
      })
   }

   // fn start_switching_graphics_level_static(mut this: std::pin::Pin<Box<&mut Self>>, gl: Rc<GL>, graphics_level: GraphicsLevel) -> Pin<Box<dyn GraphicsSwitchingFuture>> {
   //    web_sys::console::log_1(&"Rust start_switching_graphics_level_static: TriangleDemo".into());
   //    Box::pin(GraphicsSwitchingProcess{
   //       progress: 0.0,
   //       demo: Pin::new(this.as_mut().deref_mut()),
   //       graphics_level,
   //    })
   // }

   fn drop_demo(&mut self, gl: &GL) {
      gl.delete_program(Some(&self.main_program));
      web_sys::console::log_1(&"Rust demo drop: TriangleDemo".into());
   }
}

pub struct GraphicsSwitchingProcess {
   progress: f32,
   graphics_level: GraphicsLevel,
   demo: TriangleDemo,
}

impl<'a> Dispose for GraphicsSwitchingProcess {
   fn dispose(&mut self) { }
}

impl<'a> Drop for GraphicsSwitchingProcess {
   fn drop(&mut self) {
      self.dispose();
   }
}

impl<'a> Progress for GraphicsSwitchingProcess {
    fn progress(&self) -> f32 {
      self.progress
   }
}

impl<'a> SimpleFuture for GraphicsSwitchingProcess {
   type Output = Box<dyn IDemo>;
   type Context = ();

   fn simple_poll(mut self: std::pin::Pin<&mut Self>, cx: &mut Self::Context) -> std::task::Poll<Self::Output> {
      let mut x = 0_i32;
      for i in 0..200000 {
         x = x.saturating_add(i);
      }
      self.progress += 0.01;
      if self.progress >= 1.0 {
         self.demo.num_rendered_vertices = match self.graphics_level {
            GraphicsLevel::Minimal => 0,
            GraphicsLevel::Low => 3,
            GraphicsLevel::Medium => 6,
            GraphicsLevel::High => 9,
            GraphicsLevel::Ultra => 12,
         };
         std::task::Poll::Ready(Box::new(self.demo))
      } else {
         std::task::Poll::Pending
      }
   }
}

impl<'a> GraphicsSwitchingFuture for GraphicsSwitchingProcess {}
