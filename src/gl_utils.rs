use wasm_bindgen::{JsValue, JsCast};
use web_sys::{WebGl2RenderingContext as GL, WebGlShader, WebGlProgram};
use glam;

pub fn clear_with_color_u32(gl: &GL, attachment: u32, colors_array: &[u32], array_offset: u32) {
   let drawbuffer = (attachment - GL::COLOR_ATTACHMENT0) as i32;
   gl.clear_bufferuiv_with_u32_array_and_src_offset(
      GL::COLOR, drawbuffer, colors_array, array_offset)
}

pub fn clear_with_color_i32(gl: &GL, attachment: u32, colors_array: &[i32], array_offset: u32) {
   let drawbuffer = (attachment - GL::COLOR_ATTACHMENT0) as i32;
   gl.clear_bufferiv_with_i32_array_and_src_offset(
      GL::COLOR, drawbuffer, colors_array, array_offset)
}

pub fn clear_with_color_f32(gl: &GL, attachment: u32, colors_array: &[f32], array_offset: u32) {
   let drawbuffer = (attachment - GL::COLOR_ATTACHMENT0) as i32;
   gl.clear_bufferfv_with_f32_array_and_src_offset(
      GL::COLOR, drawbuffer, colors_array, array_offset)
}

pub fn clear_with_depth_stencil(gl: &GL, new_depth: f32, new_stencil: i32) {
   gl.clear_bufferfi(GL::DEPTH_STENCIL, 0, new_depth, new_stencil);
}

pub fn compile_shader(gl: &GL, shader_type: u32, source: &str) -> Result<WebGlShader, JsValue> {
   let shader = gl
       .create_shader(shader_type)
       .ok_or_else(|| JsValue::from_str("Unable to create shader object"))?;

   gl.shader_source(&shader, source);
   gl.compile_shader(&shader);

   let is_compiled = gl
       .get_shader_parameter(&shader, GL::COMPILE_STATUS)
       .as_bool()
       .unwrap_or(false);
   if is_compiled {
       Ok(shader)
   } else {
       Err(JsValue::from_str(
           &gl.get_shader_info_log(&shader)
               .unwrap_or_else(|| "Unknown error creating shader".into()),
       ))
   }
}

pub fn link_program_vert_frag(gl: &GL, vertex_shader: &WebGlShader, fragment_shader: &WebGlShader) -> Result<WebGlProgram, JsValue> {
   let shader_program = gl.create_program().unwrap();
   gl.attach_shader(&shader_program, &vertex_shader);
   gl.attach_shader(&shader_program, &fragment_shader);
   gl.link_program(&shader_program);

   let is_linked = gl
      .get_program_parameter(&shader_program, GL::LINK_STATUS)
      .as_bool()
      .unwrap_or(false);
   if is_linked {
      Ok(shader_program)
   } else {
      return Err(JsValue::from_str(
         &gl.get_program_info_log(&shader_program)
               .unwrap_or_else(|| "Unknown error linking program".into()),
      ));
   }
}

pub fn delete_program_shaders(gl: &GL, program: &WebGlProgram) {
   if let Some(shaders) = gl.get_attached_shaders(program) {
      for shader in shaders {
         let shader = shader.dyn_into::<WebGlShader>().unwrap();
         gl.detach_shader(&program, &shader);
         gl.delete_shader(Some(&shader));
      }
   }
}