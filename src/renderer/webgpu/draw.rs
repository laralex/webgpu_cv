use std::ops::Range;

use super::buffer::{IndexBuffer, VertexBuffer};

pub fn draw_indexed<'a>(render_pass: &mut wgpu::RenderPass<'a>, vbs: &'a[&'a VertexBuffer], ib: &'a IndexBuffer) {
   for (i, vb) in vbs.iter().enumerate() {
      vb.bind(render_pass, i as u32);
   }
   ib.bind(render_pass);
   render_pass.draw_indexed(0..ib.num_indices, 0, 0..1);
}

pub fn draw_instances<'a>(render_pass: &mut wgpu::RenderPass<'a>, vbs: &'a[&'a VertexBuffer], ib: &'a IndexBuffer, base_vertex: i32, instances: Range<u32>) {
   for (i, vb) in vbs.iter().enumerate() {
      vb.bind(render_pass, i as u32);
   }
   ib.bind(render_pass);
   render_pass.draw_indexed(0..ib.num_indices, base_vertex, instances);
}