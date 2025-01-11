use winit::{
    dpi::{LogicalSize, PhysicalSize},
    window::Window,
};

pub(super) struct Surface<'a> {
    pub(super) inner: wgpu::Surface<'a>,
    pub(super) config: wgpu::SurfaceConfiguration,
    pub(super) window: &'a Window,
}

pub(super) struct Context<'a> {
    pub(super) device: wgpu::Device,
    pub(super) queue: wgpu::Queue,
    pub(super) surface: Surface<'a>,
}

impl<'a> Surface<'a> {
    async fn new(inner: wgpu::Surface<'a>, adapter: &wgpu::Adapter, window: &'a Window) -> Self {
        let size = window.inner_size();
        let surface_caps = inner.get_capabilities(adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        Self {
            inner,
            config,
            window,
        }
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }

    pub fn set_dimensions(&mut self, new_dims: PhysicalSize<u32>) {
        let new_dims = LogicalSize::<f64>::from_physical(new_dims, self.window.scale_factor())
            .to_physical::<u32>(1.0);
        self.config.width = new_dims.width;
        self.config.height = new_dims.height;
    }
}

impl<'a> Context<'a> {
    pub(super) async fn new(window: &'a Window) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance
            .create_surface(window)
            .expect("Failed to create surface on window");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .expect("Failed to find adapter");

        let surface = Surface::new(surface, &adapter, window).await;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .expect("Failed to create device and queue");

        Self {
            device,
            queue,
            surface,
        }
    }

    pub fn configure_surface(&self) {
        self.surface
            .inner
            .configure(&self.device, &self.surface.config);
    }
}
