use bytemuck::NoUninit;
use wgpu::util::DeviceExt;

use crate::context::Context;

pub(crate) struct Params<D> {
    pub(crate) data: D,
    pub(crate) buffer: wgpu::Buffer,
}

impl<D: NoUninit> Params<D> {
    pub(crate) fn new(ctx: &Context, name: &str, data: D) -> Self {
        let buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{} Params Buffer", name)),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                contents: bytemuck::bytes_of(&data),
            });

        Self { data, buffer }
    }

    #[allow(unused)]
    pub(crate) fn update(&mut self, ctx: &Context, mut f: impl FnMut(&mut D)) {
        f(&mut self.data);
        ctx.queue
            .write_buffer(&self.buffer, 0, bytemuck::bytes_of(&self.data));
    }

    pub(crate) fn binding_type(&self) -> wgpu::BindingType {
        wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: Some(self.buffer.size().try_into().unwrap()),
        }
    }
}
