use nalgebra::Vector3;
use std::*;
use winit::{event, event_loop, window};
mod blocking;
mod gpu_resource;
mod loader;
//mod node;
mod renderer;
mod scene;
mod utils;

struct WgpuWindow {
    window: sync::Arc<window::Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

struct App {
    window: Option<WgpuWindow>,
    renderer: Option<renderer::Renderer>,
    glb: scene::Glb,
}

impl WgpuWindow {
    pub fn new(event_loop: &event_loop::ActiveEventLoop) -> Result<Self, Box<dyn error::Error>> {
        let window =
            sync::Arc::new(event_loop.create_window(window::Window::default_attributes().with_visible(false))?);
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            //backends: wgpu::Backends::DX12,
            flags: wgpu::InstanceFlags::ALLOW_UNDERLYING_NONCOMPLIANT_ADAPTER,
            ..Default::default()
        });
        let surface = instance.create_surface(window.clone())?;
        let adapter = blocking::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        }))
        .ok_or("request_adapter()")?;
        let (device, queue) = blocking::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::PUSH_CONSTANTS
                    | wgpu::Features::STORAGE_RESOURCE_BINDING_ARRAY
                    | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
                required_limits: wgpu::Limits {
                    max_push_constant_size: 128,
                    ..Default::default()
                },
            },
            None,
        ))?;
        window.set_visible(true);

        Ok(Self {
            window: window,
            surface: surface,
            device: device,
            queue: queue,
        })
    }

    pub fn resize(&mut self, w: u32, h: u32, format: wgpu::TextureFormat) {
        self.surface.configure(
            &self.device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: format,
                width: w,
                height: h,
                present_mode: wgpu::PresentMode::AutoVsync,
                desired_maximum_frame_latency: 2,
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: Vec::new(),
            },
        );
    }
}

impl App {
    fn new(glb: scene::Glb) -> Self {
        Self {
            window: None,
            renderer: None,
            glb: glb,
        }
    }
}

impl winit::application::ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &event_loop::ActiveEventLoop) {
        let window = WgpuWindow::new(event_loop).unwrap();
        let mut renderer = renderer::Renderer::new(&window.device, 4).unwrap();
        renderer.update(&window.device, &window.queue, &self.glb);
        renderer.set_projection_scale(1.0 / 3.0);
        self.window = Some(window);
        self.renderer = Some(renderer);
    }

    fn window_event(
        &mut self,
        event_loop: &event_loop::ActiveEventLoop,
        window_id: window::WindowId,
        event: event::WindowEvent,
    ) {
        let window = self.window.as_mut().unwrap();
        let renderer = self.renderer.as_mut().unwrap();
        if window_id != window.window.id() {
            return;
        }
        match event {
            event::WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            event::WindowEvent::Resized(size) => {
                let w = size.width.max(1);
                let h = size.height.max(1);
                window.resize(w, h, renderer::Renderer::FORMAT);
                renderer.resize(&window.device, w, h);
            }
            event::WindowEvent::RedrawRequested => {
                let frame = window.surface.get_current_texture().unwrap();
                let frame_view = frame.texture.create_view(&Default::default());

                let time = time::Instant::now();
                let mut encoder = window.device.create_command_encoder(&Default::default());
                renderer.render(
                    &mut encoder,
                    &self.glb,
                    &frame_view,
                    &scene::Node {
                        translation: Vector3::new(0.0, 0.75, -3.0),
                        ..Default::default()
                    },
                );
                let command_buffer = encoder.finish();
                println!("{:?}", time.elapsed());

                window.queue.submit(Some(command_buffer));
                frame.present();
            }
            _ => (),
        }
    }
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let glb = {
        let data = fs::read(env::args().skip(1).next().ok_or("")?)?;
        let time = time::Instant::now();
        let glb = loader::load(io::Cursor::new(data))?;
        println!("loader::load(): {:?}", time.elapsed());
        glb
    };

    event_loop::EventLoop::new()?.run_app(&mut App::new(glb))?;

    Ok(())
}
