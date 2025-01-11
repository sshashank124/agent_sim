use fastrand::Rng;
use wgpu::util::DeviceExt;

use crate::{agent::Agent, config::Config, context::Context};

pub(crate) mod config {
    pub(crate) const CANVAS_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
}

pub(crate) struct State {
    pub(crate) dimensions: (u32, u32),
    pub(crate) num_agents: u32,
    #[allow(dead_code)]
    pub(crate) agents: wgpu::Buffer,

    #[allow(dead_code)]
    pub(crate) canvas: [wgpu::Texture; 2],
    pub(crate) canvas_view: [wgpu::TextureView; 2],
    pub(crate) canvas_sampler: wgpu::Sampler,

    pub(crate) frame_number: usize,
}

impl State {
    pub fn init(ctx: &Context, config: &Config) -> Self {
        let dimensions = (config.width, config.height);
        let num_agents = config.num_agents;

        let agents = {
            let mut rng = Rng::with_seed(config.random_seed);
            let initial_data = (0..num_agents)
                .map(|_| Agent::new(|| rng.f32()))
                .collect::<Vec<_>>();

            ctx.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Agents Buffer"),
                    contents: bytemuck::cast_slice(&initial_data),
                    usage: wgpu::BufferUsages::VERTEX
                        | wgpu::BufferUsages::STORAGE
                        | wgpu::BufferUsages::COPY_DST,
                })
        };

        let canvas = core::array::from_fn(|i| {
            ctx.device.create_texture(&wgpu::TextureDescriptor {
                label: Some(&format!("Canvas Texture #{}", i)),
                size: wgpu::Extent3d {
                    width: dimensions.0,
                    height: dimensions.1,
                    ..Default::default()
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: config::CANVAS_FORMAT,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
                view_formats: &[],
            })
        });

        let canvas_view = core::array::from_fn(|i| {
            canvas[i].create_view(&wgpu::TextureViewDescriptor {
                label: Some(&format!("Canvas Texture View #{}", i)),
                ..Default::default()
            })
        });

        let canvas_sampler = ctx.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Canvas Texture Sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        Self {
            dimensions,
            num_agents,

            agents,

            canvas,
            canvas_view,
            canvas_sampler,

            frame_number: 0,
        }
    }

    pub fn update(&mut self) {
        self.frame_number += 1;
    }
}
