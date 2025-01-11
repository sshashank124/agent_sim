use wgpu::util::DeviceExt;

use crate::{agent::Agent, config::Config, context::Context, state};

use super::params::Params;

mod config {
    pub(super) const PASS_NAME: &str = "Draw Agents";
    pub(super) const SHADER_SOURCE: wgpu::ShaderModuleDescriptor =
        wgpu::include_wgsl!("draw_agents.wgsl");
}

#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct ParamsData {
    scale: f32,
}

pub(crate) struct DrawAgents {
    pipeline: wgpu::RenderPipeline,
    common_bind_group: wgpu::BindGroup,

    agent_mesh: wgpu::Buffer,
    agent_num_vertices: u32,

    #[allow(unused)]
    params: Params<ParamsData>,
}

impl DrawAgents {
    pub fn new(ctx: &Context, config: &Config) -> Self {
        let params = Params::new(ctx, config::PASS_NAME, ParamsData::from(config));

        let common_bind_group_layout =
            ctx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some(&format!("{} Common Bind Group Layout", config::PASS_NAME)),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: params.binding_type(),
                        count: None,
                    }],
                });

        let pipeline_layout = ctx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(&format!("{} Pipeline Layout", config::PASS_NAME)),
                bind_group_layouts: &[&common_bind_group_layout],
                ..Default::default()
            });

        let common_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{} Common Bind Group", config::PASS_NAME)),
            layout: &common_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: params.buffer.as_entire_binding(),
            }],
        });

        let shader = ctx.device.create_shader_module(config::SHADER_SOURCE);

        let pipeline = ctx
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(&format!("{} Pipeline", config::PASS_NAME)),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[
                        wgpu::VertexBufferLayout {
                            array_stride: core::mem::size_of::<Agent>() as wgpu::BufferAddress,
                            step_mode: wgpu::VertexStepMode::Instance,
                            attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32],
                        },
                        wgpu::VertexBufferLayout {
                            array_stride: core::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &wgpu::vertex_attr_array![2 => Float32x2],
                        },
                    ],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(ctx.surface.config.format.into())],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleStrip,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });

        let (agent_mesh, agent_num_vertices) = {
            let triangle_strip = [[-0.02f32, 0.01], [-0.01, 0.], [0.02, 0.], [-0.02, -0.01]];

            let buffer = ctx
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Agent Mesh Vertex Buffer"),
                    contents: bytemuck::cast_slice(&triangle_strip),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                });

            (buffer, triangle_strip.len() as _)
        };

        Self {
            pipeline,
            common_bind_group,

            agent_mesh,
            agent_num_vertices,

            params,
        }
    }

    pub fn run(&self, render_pass: &mut wgpu::RenderPass, state: &state::State) {
        if self.params.data.scale == 0. {
            return;
        }

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.common_bind_group, &[]);
        render_pass.set_vertex_buffer(0, state.agents.slice(..));
        render_pass.set_vertex_buffer(1, self.agent_mesh.slice(..));
        render_pass.draw(0..self.agent_num_vertices, 0..state.num_agents);
    }
}

impl From<&Config> for ParamsData {
    fn from(config: &Config) -> Self {
        Self {
            scale: config.agent.draw_scale,
        }
    }
}
