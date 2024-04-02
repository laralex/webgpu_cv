use super::{ExternalState, IDemo, MouseState, GraphicsLevel};
use crate::gl_utils;
use web_sys::{WebGl2RenderingContext as GL, WebGlProgram, WebGlVertexArrayObject};

pub struct TriangleDemo {
   main_program: WebGlProgram,
   clear_color: [f32; 4],
   num_rendered_vertices: i32,
}

impl TriangleDemo {
   pub fn new(gl: &GL, graphics_level: GraphicsLevel) -> Self {
      let vertex_shader_source = std::include_str!("shaders/no_vao_triangle.vert");
      let fragment_shader_source = std::include_str!("shaders/vertex_color.frag");

      let vertex_shader = gl_utils::compile_shader(
            &gl, GL::VERTEX_SHADER, vertex_shader_source)
        .inspect_err(|err| panic!("Vert shader failed to compile {}", err.as_string().unwrap()))
        .unwrap();
      let fragment_shader = gl_utils::compile_shader(
            &gl,GL::FRAGMENT_SHADER, fragment_shader_source)
         .inspect_err(|err| panic!("Frag shader failed to compile {}", err.as_string().unwrap()))
         .unwrap();
      let main_program = gl_utils::link_program_vert_frag(
         gl, &vertex_shader, &fragment_shader)
         .inspect_err(|err| panic!("Program failed to link {}", err.as_string().unwrap()))
         .unwrap();

      gl_utils::delete_program_shaders(gl, &main_program);
      web_sys::console::log_1(&"Rust loaded TriangleDemo".into());
      let mut me = Self {
         main_program,
         clear_color: [0.0; 4],
         num_rendered_vertices: 0,
      };
      me.set_graphics_level(graphics_level);
      me
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

   fn render(&mut self, gl: &mut GL, delta_sec: f32) {
      gl.bind_framebuffer(GL::FRAMEBUFFER, None);
      gl_utils::clear_with_color_f32(
         gl, GL::COLOR_ATTACHMENT0, &self.clear_color, 0);
      gl.use_program(Some(&self.main_program));
      gl.draw_arrays(GL::TRIANGLES, 0, self.num_rendered_vertices);
   }

   fn set_graphics_level(&mut self, level: GraphicsLevel) {
      self.num_rendered_vertices = match level {
         GraphicsLevel::Minimal => 0,
         GraphicsLevel::Low => 3,
         GraphicsLevel::Medium => 6,
         GraphicsLevel::High => 9,
         GraphicsLevel::Ultra => 12,
      };
   }

}