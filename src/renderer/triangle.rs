use super::{ExternalState, IDemo, MouseState};
use crate::gl_utils;
use web_sys::{WebGl2RenderingContext as GL, WebGlProgram, WebGlVertexArrayObject};

pub struct TriangleDemo {
   main_program: WebGlProgram,
   render_timer: f32,
   tick_timer: f32,
   clear_color: [f32; 4],
}

impl TriangleDemo {
   pub fn new(gl: &GL) -> Self {
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

      Self {
         main_program,
         render_timer: 0.0,
         tick_timer: 0.0,
         clear_color: [0.0; 4],
      }
   }
}

impl IDemo for TriangleDemo {
   fn tick(&mut self, input: &ExternalState) {
      self.tick_timer += input.delta_sec;

      let mouse_pos = input.mouse.get().unit_position;
      self.clear_color[0] = mouse_pos.0;
      self.clear_color[1] = mouse_pos.1;
      self.clear_color[2] = input.mouse.get().left;
      self.clear_color[3] = 1.0;
      // web_sys::console::log_3(&"Rust mousedown".into(), &mouse_pos.0.into(), &mouse_pos.1.into());
   }

   fn render(&mut self, gl: &mut GL, delta_sec: f32) {
      self.render_timer += delta_sec;

      gl.bind_framebuffer(GL::FRAMEBUFFER, None);
      gl_utils::clear_with_color_f32(
         gl, GL::COLOR_ATTACHMENT0, &self.clear_color, 0);
      gl.use_program(Some(&self.main_program));
      gl.draw_arrays(GL::TRIANGLES, 0, 3);
   }
}