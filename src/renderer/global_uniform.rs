use wgpu::ShaderStages;

use crate::{renderer::webgpu::buffer::Buffer, timer::ScopedTimer};

use super::{webgpu::{buffer::UniformBuffer, uniform::BindGroupInfo}, ExternalState};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct StableGlobalUniformData {
   pub color_attachment_size: [u32; 2],
   pub aspect_ratio: f32,
   pub is_debug: f32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DynamicGlobalUniformData {
   pub mouse_position: [f32; 2],
   __padding: [u32; 2],
}

pub struct GlobalUniform {
   pub stable_data: StableGlobalUniformData,
   pub stable_buffer_offset: u64,
   pub dynamic_data: DynamicGlobalUniformData,
   pub dynamic_buffer_offset: u64,
   pub uniform_buffer: UniformBuffer,
   pub bind_group_info: BindGroupInfo,
   pub pending_stable_write_gpu: bool,
}

impl GlobalUniform {
   pub fn new(device: &wgpu::Device) -> Self {
      let _t = ScopedTimer::new("GlobalUniform::new");
      let uniform_visibility = ShaderStages::FRAGMENT | ShaderStages::VERTEX;
      let uniform_usage = wgpu::BufferUsages::COPY_DST;
      let stable_buffer_offset = 0;
      let dynamic_buffer_offset = device.limits().min_uniform_buffer_offset_alignment as u64;

      const STABLE_DATA_SIZE: u64 = std::mem::size_of::<StableGlobalUniformData>() as u64;
      const DYNAMIC_DATA_SIZE: u64 = std::mem::size_of::<DynamicGlobalUniformData>() as u64;

      let uniform_buffer = Buffer::new_uniform_size(
         &device, dynamic_buffer_offset + DYNAMIC_DATA_SIZE,
          uniform_usage, Some("Demo Bind Buffer"));
      let bind_group = BindGroupInfo::builder()
         .with_uniform_buffer_range(0, uniform_visibility,
            &uniform_buffer.buffer, (stable_buffer_offset, STABLE_DATA_SIZE))
         .with_uniform_buffer_range(1, uniform_visibility,
            &uniform_buffer.buffer, (dynamic_buffer_offset, DYNAMIC_DATA_SIZE))
         .build(&device, Some("Demo Bind Group"), None);

      let stable_data = StableGlobalUniformData {
        color_attachment_size: [1, 1],
        aspect_ratio: 1.0,
        is_debug: 0.0,
      };
      let dynamic_data = DynamicGlobalUniformData {
        mouse_position: [-1.0, -1.0],
        __padding: Default::default(),
      };

      Self {
         stable_data,
         stable_buffer_offset,
         dynamic_data,
         dynamic_buffer_offset,
         uniform_buffer,
         bind_group_info: bind_group,
         pending_stable_write_gpu: false,
      }
   }

   pub fn update_cpu(&mut self, state: &ExternalState) {
      self.stable_data.color_attachment_size = state.screen_size().into();
      self.stable_data.aspect_ratio = state.aspect_ratio();
      self.stable_data.is_debug = state.debug_mode().map_or(0.0, f32::from);
      self.dynamic_data.mouse_position = state.mouse_unit_position().into();
      self.pending_stable_write_gpu = self.pending_stable_write_gpu || state.is_stable_updated();
   }

   pub fn update_gpu(&mut self, queue: &wgpu::Queue) {
      if self.pending_stable_write_gpu {
         self.uniform_buffer.write(
            queue,
            self.stable_buffer_offset,
            &[self.stable_data]);
         self.pending_stable_write_gpu = false;
      }
      self.uniform_buffer.write(
         queue,
         self.dynamic_buffer_offset,
         &[self.dynamic_data]);
   }
}