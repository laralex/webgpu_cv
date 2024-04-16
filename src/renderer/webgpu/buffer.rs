use std::ops::RangeBounds;

use wgpu::{util::DeviceExt, BufferAddress, VertexAttribute};

pub struct VertexBuffer {
    pub buffer: wgpu::Buffer,
}

impl<'a> VertexBuffer {
    pub fn new(device: &wgpu::Device, size: u64, usage: wgpu::BufferUsages, label: Option<&str>) -> Self {
        let usage = wgpu::BufferUsages::VERTEX.union(usage);
        let mapped_at_creation = false;
        let buffer = device.create_buffer(
            &wgpu::BufferDescriptor { label, mapped_at_creation, size, usage }
        );
        Self {
            buffer,
        }
    }

    pub fn new_init(device: &wgpu::Device, contents: &[u8], usage: wgpu::BufferUsages, label: Option<&str>) -> Self {
        let usage = wgpu::BufferUsages::VERTEX.union(usage);
        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor { label, contents, usage }
        );
        Self {
            buffer,
        }
    }

    pub fn bind(&'a self, render_pass: &mut wgpu::RenderPass<'a>, slot: u32) {
        render_pass.set_vertex_buffer(slot, self.buffer.slice(..));
    }

    pub fn bind_slice<S>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, slot: u32, bounds: S) where S: RangeBounds<BufferAddress> {
        render_pass.set_vertex_buffer(slot, self.buffer.slice(bounds));
    }
}

pub struct IndexBuffer {
    pub buffer: wgpu::Buffer,
    pub dtype: wgpu::IndexFormat,
    pub num_indices: u32,
}

impl<'a> IndexBuffer {
    pub fn new(device: &wgpu::Device, contents: &[u8], dtype: wgpu::IndexFormat, usage: wgpu::BufferUsages, label: Option<&str>) -> Self {
        let usage = wgpu::BufferUsages::INDEX.union(usage);
        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor { label, contents, usage }
        );
        let num_indices = contents.len() as u32 / match dtype {
            wgpu::IndexFormat::Uint16 => 2,
            wgpu::IndexFormat::Uint32 => 4,
        };
        Self {
            buffer, dtype, num_indices
        }
    }

    pub fn bind(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_index_buffer(self.buffer.slice(..), self.dtype);
    }

    pub fn bind_dtype(&'a self, render_pass: &'a mut wgpu::RenderPass<'a>, dtype: wgpu::IndexFormat) {
        render_pass.set_index_buffer(self.buffer.slice(..), dtype);
    }

    pub fn bind_slice<S>(&'a self, render_pass: &'a mut wgpu::RenderPass<'a>, bounds: S) where S: RangeBounds<BufferAddress> {
        render_pass.set_index_buffer(self.buffer.slice(bounds), self.dtype);
    }
}

fn vertex_layout<T: Sized>(attributes: &[VertexAttribute]) -> wgpu::VertexBufferLayout {
    wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<T>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes,
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VertexPosUv {
    position: [f32; 3],
    uv: [f32; 2],
}

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
    position: [f32; 3],
    uv: [f32; 2],
    normal: [f32; 3],
}

impl VertexPosUvNormal {
    const ATTRIBS: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x3];
 
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        vertex_layout::<Self>(Self::ATTRIBS.as_slice())
    }
}