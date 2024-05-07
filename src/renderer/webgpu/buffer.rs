use std::ops::RangeBounds;

use wgpu::util::DeviceExt;

pub struct Buffer {
    pub buffer: wgpu::Buffer,
}

#[allow(unused)]
impl<'a> Buffer {
    pub fn new(device: &wgpu::Device, size: u64, usage: wgpu::BufferUsages, label: Option<&str>) -> wgpu::Buffer {
        let mapped_at_creation = false;
        device.create_buffer(
            &wgpu::BufferDescriptor { label, mapped_at_creation, size, usage }
        )
    }

    pub fn new_init(device: &wgpu::Device, contents: &[u8], usage: wgpu::BufferUsages, label: Option<&str>) -> wgpu::Buffer {
        let usage = wgpu::BufferUsages::VERTEX.union(usage);
        device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor { label, contents, usage }
        )
    }

    pub fn new_vertex(device: &wgpu::Device, size: u64, usage: wgpu::BufferUsages, label: Option<&str>) -> VertexBuffer {
        let usage = usage.union(wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST);
        VertexBuffer{
            buffer: Self::new(device ,size, usage, label),
        }
    }

    pub fn new_vertex_init(device: &wgpu::Device, contents: &[u8], usage: wgpu::BufferUsages, label: Option<&str>) -> VertexBuffer {
        let usage = usage.union(wgpu::BufferUsages::VERTEX);
        VertexBuffer{
            buffer: Self::new_init(device, contents, usage, label),
        }
    }

    pub fn new_uniform<T: Sized>(device: &wgpu::Device, usage: wgpu::BufferUsages, label: Option<&str>) -> UniformBuffer {
        Self::new_uniform_size(device, std::mem::size_of::<T>() as u64, usage, label)
    }

    pub fn new_uniform_size(device: &wgpu::Device, size: u64, usage: wgpu::BufferUsages, label: Option<&str>) -> UniformBuffer {
        let usage = usage.union(wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST);
        UniformBuffer{
            buffer: Self::new(device, size, usage, label),
        }
    }

    pub fn new_uniform_init(device: &wgpu::Device, contents: &[u8], usage: wgpu::BufferUsages, label: Option<&str>) -> UniformBuffer {
        let usage = usage.union(wgpu::BufferUsages::UNIFORM);
        UniformBuffer{
            buffer: Self::new_init(device, contents, usage, label),
        }
    }

    pub fn new_index_init(device: &wgpu::Device, contents: &[u8], dtype: wgpu::IndexFormat, usage: wgpu::BufferUsages, label: Option<&str>) -> IndexBuffer {
        let usage = wgpu::BufferUsages::INDEX.union(usage);
        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor { label, contents, usage }
        );
        let num_indices = contents.len() as u32 / match dtype {
            wgpu::IndexFormat::Uint16 => 2,
            wgpu::IndexFormat::Uint32 => 4,
        };
        IndexBuffer {
            buffer, dtype, num_indices
        }
    }

}

pub struct VertexBuffer {
    pub buffer: wgpu::Buffer,
}

#[allow(unused)]
impl<'a> VertexBuffer {
    pub fn new(buffer: wgpu::Buffer) -> Self {
        Self { buffer }
    }

    pub fn bind(&'a self, render_pass: &mut wgpu::RenderPass<'a>, slot: u32) {
        render_pass.set_vertex_buffer(slot, self.buffer.slice(..));
    }

    pub fn bind_slice<S>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, slot: u32, bounds: S) where S: RangeBounds<wgpu::BufferAddress> {
        render_pass.set_vertex_buffer(slot, self.buffer.slice(bounds));
    }
}

pub struct UniformBuffer {
    pub buffer: wgpu::Buffer,
}

impl UniformBuffer {
    // pub fn write(&self, queue: &wgpu::Queue, offset: u64, data: &[u8]) {
    //     queue.write_buffer(&self.buffer, offset, data);
    // }
    pub fn write<T>(&self, queue: &wgpu::Queue, offset: u64, data: &[T]) where T:  bytemuck::Pod {
        queue.write_buffer(&self.buffer, offset, bytemuck::cast_slice(data));
    }
}
pub struct IndexBuffer {
    pub buffer: wgpu::Buffer,
    pub dtype: wgpu::IndexFormat,
    pub num_indices: u32,
}

#[allow(unused)]
impl<'a> IndexBuffer {
    pub fn bind(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_index_buffer(self.buffer.slice(..), self.dtype);
    }

    pub fn bind_dtype(&'a self, render_pass: &'a mut wgpu::RenderPass<'a>, dtype: wgpu::IndexFormat) {
        render_pass.set_index_buffer(self.buffer.slice(..), dtype);
    }

    pub fn bind_slice<S>(&'a self, render_pass: &'a mut wgpu::RenderPass<'a>, bounds: S) where S: RangeBounds<wgpu::BufferAddress> {
        render_pass.set_index_buffer(self.buffer.slice(bounds), self.dtype);
    }
}

fn vertex_layout<T: Sized>(attributes: &[wgpu::VertexAttribute]) -> wgpu::VertexBufferLayout {
    wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<T>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes,
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VertexPosUv {
    pub position: [f32; 3],
    pub uv: [f32; 2],
}

#[allow(unused)]
impl VertexPosUv {
   const ATTRIBS: [wgpu::VertexAttribute; 2] =
       wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

   pub fn layout() -> wgpu::VertexBufferLayout<'static> {
       vertex_layout::<Self>(Self::ATTRIBS.as_slice())
   }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VertexPosUvNormal {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub normal: [f32; 3],
}

#[allow(unused)]
impl VertexPosUvNormal {
    const ATTRIBS: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x3];
 
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        vertex_layout::<Self>(Self::ATTRIBS.as_slice())
    }
}