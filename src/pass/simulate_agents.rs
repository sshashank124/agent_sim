use crate::{config::Config, context::Context, state};

use super::params::Params;

mod config {
    pub(super) const PASS_NAME: &str = "Simulate Agents";
    pub(super) const SHADER_SOURCE: wgpu::ShaderModuleDescriptor =
        wgpu::include_wgsl!("simulate_agents.wgsl");

    pub(super) const SHADER_WORKGROUP_SIZE: u32 = 64;
}

#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct ParamsData {
    speed: f32,
    turning_speed: f32,
    sensor_distance: f32,
    sensor_angle: f32,
    sensor_radius: u32,
    frame_number: u32,
}

pub(crate) struct SimulateAgents {
    pipeline: wgpu::ComputePipeline,
    common_bind_group: wgpu::BindGroup,
    canvas_bind_group: [wgpu::BindGroup; 2],

    #[allow(unused)]
    params: Params<ParamsData>,
}

impl SimulateAgents {
    pub fn new(ctx: &Context, config: &Config, state: &state::State) -> Self {
        let params = Params::new(ctx, config::PASS_NAME, ParamsData::from(config));

        let common_bind_group_layout =
            ctx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some(&format!("{} Common Bind Group Layout", config::PASS_NAME)),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: params.binding_type(),
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: false },
                                has_dynamic_offset: false,
                                min_binding_size: Some(state.agents.size().try_into().unwrap()),
                            },
                            count: None,
                        },
                    ],
                });

        let canvas_bind_group_layout =
            ctx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some(&format!("{} Canvas Bind Group Layout", config::PASS_NAME)),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::StorageTexture {
                                access: wgpu::StorageTextureAccess::WriteOnly,
                                format: state::config::CANVAS_FORMAT,
                                view_dimension: wgpu::TextureViewDimension::D2,
                            },
                            count: None,
                        },
                    ],
                });

        let pipeline_layout = ctx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(&format!("{} Pipeline Layout", config::PASS_NAME)),
                bind_group_layouts: &[&common_bind_group_layout, &canvas_bind_group_layout],
                ..Default::default()
            });

        let common_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{} Common Bind Group", config::PASS_NAME)),
            layout: &common_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params.buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: state.agents.as_entire_binding(),
                },
            ],
        });

        let canvas_bind_group = core::array::from_fn(|i| {
            ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&format!("{} Canvas Bind Group #{}", config::PASS_NAME, i)),
                layout: &canvas_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&state.canvas_view[i]),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(
                            &state.canvas_view[(i + 1) % 2],
                        ),
                    },
                ],
            })
        });

        let shader = ctx.device.create_shader_module(config::SHADER_SOURCE);

        let pipeline = ctx
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(&format!("{} Pipeline", config::PASS_NAME)),
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: Some("main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            });

        Self {
            pipeline,
            common_bind_group,
            canvas_bind_group,

            params,
        }
    }

    pub fn run(
        &mut self,
        ctx: &Context,
        compute_pass: &mut wgpu::ComputePass,
        state: &state::State,
    ) {
        self.params.update(ctx, |p| {
            p.frame_number = (state.frame_number % u32::MAX as usize) as _;
        });

        compute_pass.set_pipeline(&self.pipeline);
        compute_pass.set_bind_group(0, &self.common_bind_group, &[]);
        compute_pass.set_bind_group(1, &self.canvas_bind_group[state.frame_number % 2], &[]);
        compute_pass.dispatch_workgroups(
            state.num_agents.div_ceil(config::SHADER_WORKGROUP_SIZE),
            1,
            1,
        );
    }
}

impl From<&Config> for ParamsData {
    fn from(config: &Config) -> Self {
        Self {
            speed: config.agent.speed,
            turning_speed: config.agent.turning_speed,
            sensor_distance: config.agent.sensor_distance,
            sensor_angle: config.agent.sensor_angle,
            sensor_radius: config.agent.sensor_radius,
            frame_number: 0,
        }
    }
}
