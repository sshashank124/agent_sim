mod agent;
mod config;
mod context;
mod pass;
mod state;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowBuilder},
};

use config::Config;
use context::Context;
use state::State;

struct App<'a> {
    #[allow(dead_code)]
    state: State,
    simulate_world_pass: pass::SimulateWorld,
    simulate_agents_pass: pass::SimulateAgents,
    draw_world_pass: pass::DrawWorld,

    #[allow(unused)]
    draw_agents_pass: pass::DrawAgents,

    ctx: Context<'a>,
}

impl<'a> App<'a> {
    async fn new(window: &'a Window) -> Self {
        let config = Config::load();

        let ctx = Context::new(window).await;

        let state = State::init(&ctx, &config);

        let simulate_world_pass = pass::SimulateWorld::new(&ctx, &config, &state);
        let simulate_agents_pass = pass::SimulateAgents::new(&ctx, &config, &state);
        let draw_world_pass = pass::DrawWorld::new(&ctx, &state);
        let draw_agents_pass = pass::DrawAgents::new(&ctx, &config);

        Self {
            state,

            simulate_world_pass,
            simulate_agents_pass,
            draw_world_pass,
            draw_agents_pass,

            ctx,
        }
    }

    fn resize(&mut self, new_size: Option<PhysicalSize<u32>>) {
        let new_size = new_size.unwrap_or(self.ctx.surface.dimensions().into());
        if new_size.width > 0 && new_size.height > 0 {
            self.ctx.surface.set_dimensions(new_size);
            self.ctx.configure_surface();
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.state.update();

        let frame = self.ctx.surface.inner.get_current_texture()?;

        let frame_view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Command Encoder"),
            });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute Pass"),
                ..Default::default()
            });

            self.simulate_world_pass.run(&mut compute_pass, &self.state);
            self.simulate_agents_pass
                .run(&self.ctx, &mut compute_pass, &self.state);
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &frame_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });

            self.draw_world_pass.run(&mut render_pass, &self.state);
            self.draw_agents_pass.run(&mut render_pass, &self.state);
        }

        self.ctx.queue.submit(Some(encoder.finish()));
        frame.present();

        Ok(())
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Info).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }

    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("Agent Sim")
        .build(&event_loop)
        .unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowExtWebSys;

        let web_window = web_sys::window().unwrap();

        let dimensions = PhysicalSize::new(
            web_window.inner_width().unwrap().as_f64().unwrap() as u32,
            web_window.inner_height().unwrap().as_f64().unwrap() as _,
        );

        let _ = window.request_inner_size(dimensions);

        let canvas = window.canvas().unwrap();
        canvas.set_width(dimensions.width);
        canvas.set_height(dimensions.height);

        let canvas = web_sys::Element::from(canvas);
        web_window
            .document()
            .unwrap()
            .get_element_by_id("wasm-frame")
            .unwrap()
            .append_child(&canvas)
            .ok()
            .unwrap();
    }

    let mut app = App::new(&window).await;

    let mut surface_configured = false;

    event_loop
        .run(move |event, control_flow| {
            if let Event::WindowEvent { ref event, .. } = event {
                match event {
                    WindowEvent::RedrawRequested => {
                        app.ctx.surface.window.request_redraw();

                        if !surface_configured {
                            return;
                        }

                        match app.render() {
                            Ok(_) => {}
                            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                                app.resize(None)
                            }
                            Err(wgpu::SurfaceError::OutOfMemory) => {
                                log::error!("Surface Error: Out of Memory");
                                control_flow.exit();
                            }
                            Err(wgpu::SurfaceError::Timeout) => {
                                log::warn!("Surface Error: Timeout")
                            }
                        }
                    }
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                state: ElementState::Pressed,
                                physical_key: PhysicalKey::Code(KeyCode::Escape),
                                ..
                            },
                        ..
                    } => control_flow.exit(),
                    WindowEvent::Resized(new_size) => {
                        app.resize(Some(*new_size));
                        surface_configured = true;
                    }
                    _ => {}
                }
            }
        })
        .expect("Failed to run event loop");
}
