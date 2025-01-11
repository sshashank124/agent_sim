use crate::{context::Context, state};

mod config {
    pub(super) const PASS_NAME: &str = "Draw World";
    pub(super) const SHADER_SOURCE: wgpu::ShaderModuleDescriptor =
        wgpu::include_wgsl!("draw_world.wgsl");
}

pub(crate) struct DrawWorld {
    pipeline: wgpu::RenderPipeline,
    common_bind_group: wgpu::BindGroup,
    canvas_bind_group: [wgpu::BindGroup; 2],
}

impl DrawWorld {
    pub fn new(ctx: &Context, state: &state::State) -> Self {
        let common_bind_group_layout =
            ctx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some(&format!("{} Common Bind Group Layout", config::PASS_NAME)),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    }],
                });

        let canvas_bind_group_layout =
            ctx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some(&format!("{} Canvas Bind Group Layout", config::PASS_NAME)),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    }],
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
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Sampler(&state.canvas_sampler),
            }],
        });

        let canvas_bind_group = core::array::from_fn(|i| {
            ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&format!("{} Canvas Bind Group #{}", config::PASS_NAME, i)),
                layout: &canvas_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&state.canvas_view[(i + 1) % 2]),
                }],
            })
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
                    buffers: &[],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(ctx.surface.config.format.into())],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });

        Self {
            pipeline,
            common_bind_group,
            canvas_bind_group,
        }
    }

    pub fn run(&self, render_pass: &mut wgpu::RenderPass, state: &state::State) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.common_bind_group, &[]);
        render_pass.set_bind_group(1, &self.canvas_bind_group[state.frame_number % 2], &[]);
        render_pass.draw(0..3, 0..1);
    }
}
