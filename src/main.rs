use nalgebra::Vector3;
use std::*;
mod blocking;
mod gpu_resource;
mod loader;
mod node;
mod renderer;
mod scene;
mod utils;

pub struct App<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    renderer: renderer::Renderer,
    glb: scene::Glb,
}

impl<'a> App<'a> {
    pub fn new(window: &'a winit::window::Window, glb: scene::Glb) -> Result<Self, Box<dyn error::Error>> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            //backends: wgpu::Backends::DX12,
            flags: wgpu::InstanceFlags::ALLOW_UNDERLYING_NONCOMPLIANT_ADAPTER,
            ..Default::default()
        });
        let surface = instance.create_surface(window)?;
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

        let mut renderer = renderer::Renderer::new(&device, 4)?;
        renderer.update(&device, &queue, &glb);
        renderer.set_projection_scale(1.0 / 3.0);

        Ok(App {
            surface: surface,
            device: device,
            queue: queue,
            renderer: renderer,
            glb: glb,
        })
    }

    pub fn resize(&mut self, w: u32, h: u32) {
        let w = w.max(1);
        let h = h.max(1);
        self.surface.configure(
            &self.device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: renderer::Renderer::FORMAT,
                width: w,
                height: h,
                present_mode: wgpu::PresentMode::AutoVsync,
                desired_maximum_frame_latency: 2,
                alpha_mode: wgpu::CompositeAlphaMode::Opaque,
                view_formats: Vec::new(),
            },
        );
        self.renderer.resize(&self.device, w, h);
    }

    pub fn render(&self) {
        let frame = self.surface.get_current_texture().unwrap();
        let frame_view = frame.texture.create_view(&Default::default());

        let time = time::Instant::now();
        let mut encoder = self.device.create_command_encoder(&Default::default());
        self.renderer.render(
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

        self.queue.submit(Some(command_buffer));
        frame.present();
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

    let looper = winit::event_loop::EventLoop::new()?;
    let window = winit::window::WindowBuilder::new().with_visible(false).build(&looper)?;
    let mut app = App::new(&window, glb)?;

    looper.run(|ev, target| {
        use winit::event::*;
        match ev {
            Event::NewEvents(StartCause::Init) => {
                window.set_visible(true);
            }
            Event::WindowEvent { event, window_id } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => target.exit(),
                WindowEvent::Resized(size) => app.resize(size.width, size.height),
                WindowEvent::RedrawRequested => app.render(),
                _ => (),
            },
            _ => (),
        }
    })?;

    Ok(())
}
