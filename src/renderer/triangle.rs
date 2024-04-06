use std::{future::Future, rc::Rc, sync::Mutex};

use super::{DemoLoadingFuture, Dispose, ExternalState, GraphicsLevel, IDemo, MouseState, Progress, SimpleFuture};
use crate::gl_utils;
use wasm_bindgen::convert::OptionIntoWasmAbi;
use web_sys::{WebGl2RenderingContext as GL, WebGlProgram, WebGlShader, WebGlVertexArrayObject};
use futures::{future::BoxFuture, FutureExt};

pub struct TriangleDemo {
   main_program: WebGlProgram,
   clear_color: [f32; 4],
   num_rendered_vertices: i32,
}

struct InstantiateDemoFuture;
impl Future for InstantiateDemoFuture {
   type Output = ();

   fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
      let mut x = 0_u32;
      for i in 0..200000 {
         x = x.saturating_add(i);
      }
      std::task::Poll::Ready(())
   }
}

enum DemoLoadingStage {
   CompileShaders = 0,
   LinkPrograms,
   DummyWait,
   SetGraphicsLevel,
   Ready,
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

   fn poll(mut self: std::pin::Pin<&mut Self>, cx: &mut Self::Context) -> std::task::Poll<Self::Output> {
      use DemoLoadingStage::*;
      match self.stage {
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
            self.stage_percent = 0.2;
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
            self.stage_percent = 0.4;
            self.stage = DummyWait;
            std::task::Poll::Pending
         }
         DummyWait => {
            let mut x = 0_i32;
            for i in 0..100000 {
               x = x.saturating_add(i);
            }
            self.stage_percent += 0.005;
            self.dummy_counter += 1;
            if (self.stage_percent >= 1.0) {
               self.stage_percent = 1.0;
               self.stage = SetGraphicsLevel;
            }
            if self.dummy_counter % 500 == 0 {
               web_sys::console::log_2(&"Rust continue loading: TriangleDemo".into(), &self.stage_percent.into());
            }
            std::task::Poll::Pending
         }
         SetGraphicsLevel => {
            let mut demo = TriangleDemo {
               main_program: self.main_program.as_ref().unwrap().clone(),
               clear_color: [0.0; 4],
               num_rendered_vertices: 3,
            };
            demo.set_graphics_level(self.graphics_level);
            self.stage_percent = 1.0;
            self.stage = Ready;
            web_sys::console::log_1(&"Rust loading ready: TriangleDemo".into());
            std::task::Poll::Ready(Box::new(demo))
         }
         Ready => unreachable!("Should not poll the task again after std::task::Poll::Ready was polled"),
      }
   }
}

impl DemoLoadingFuture for DemoLoadingProcess {}

impl TriangleDemo {
   pub fn start_loading<'a>(gl: Rc<GL>, graphics_level: GraphicsLevel) -> impl DemoLoadingFuture {
      DemoLoadingProcess {
         stage: DemoLoadingStage::CompileShaders,
         stage_percent: 0.0,
         graphics_level,
         main_program: Default::default(),
         vert_shader: Default::default(),
         frag_shader: Default::default(),
         gl,
         dummy_counter: 0,
      }
   }
}

impl IDemo for TriangleDemo {
   fn tick(&mut self, input: &ExternalState) {
      let mouse_pos = input.mouse_unit_position();
      self.clear_color[0] = mouse_pos.0;
      self.clear_color[1] = mouse_pos.1;
      self.clear_color[2] = input.mouse.get().left;
      self.clear_color[3] = 1.0;
      // web_sys::console::log_3(&"Rust tick".into(), &mouse_pos.0.into(), &mouse_pos.1.into());
   }

   fn render(&mut self, gl: &GL, delta_sec: f32) {
      gl.bind_framebuffer(GL::FRAMEBUFFER, None);
      gl_utils::clear_with_color_f32(
         gl, GL::COLOR_ATTACHMENT0, &self.clear_color, 0);
      gl.use_program(Some(&self.main_program));
      gl.draw_arrays(GL::TRIANGLES, 0, self.num_rendered_vertices);
   }

   fn set_graphics_level(&mut self, level: GraphicsLevel){
      self.num_rendered_vertices = match level {
         GraphicsLevel::Minimal => 0,
         GraphicsLevel::Low => 3,
         GraphicsLevel::Medium => 6,
         GraphicsLevel::High => 9,
         GraphicsLevel::Ultra => 12,
      };
   }

   fn drop_demo(&mut self, gl: &GL) {
      gl.delete_program(Some(&self.main_program));
      web_sys::console::log_1(&"Rust demo drop: TriangleDemo".into());
   }
}