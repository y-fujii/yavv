use std::*;
mod blocking;
mod loader;
mod utils;
//mod node;
mod gpu_resource;
mod renderer;
mod scene;

pub struct App<'a> {
    surface: wgpu::Surface<'a>,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    renderer: renderer::Renderer,
    glb: scene::Glb,
    gpu: gpu_resource::GpuResource,
}

impl<'a> App<'a> {
    pub fn new(window: &'a winit::window::Window) -> Result<Self, Box<dyn error::Error>> {
        let instance = wgpu::Instance::default();
        /*
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::DX12,
            ..Default::default()
        });
        */
        let surface = instance.create_surface(window)?;
        let adapter = blocking::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        }))
        .ok_or("request_adapter()")?;
        let (device, queue) = blocking::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::PUSH_CONSTANTS,
                required_limits: wgpu::Limits {
                    max_push_constant_size: 128,
                    ..Default::default()
                },
            },
            None,
        ))?;

        let format = surface.get_capabilities(&adapter).formats[0];
        assert!(
            format == wgpu::TextureFormat::Bgra8UnormSrgb || //,
            format == wgpu::TextureFormat::Rgba8UnormSrgb ||
            format == wgpu::TextureFormat::Rgba16Float ||
            format == wgpu::TextureFormat::Rgba32Float
        );

        //let glb = loader::load(fs::File::open("models/AliciaSolid_vrm-0.51.vrm")?)?;
        let glb = loader::load(fs::File::open("models/hakka.vrm")?)?;
        let mut gpu = gpu_resource::GpuResource::new(&device);
        gpu.update(&device, &queue, &glb);
        let renderer = renderer::Renderer::new(&device, format, &gpu)?;

        let mut this = App {
            surface: surface,
            adapter: adapter,
            device: device,
            queue: queue,
            renderer: renderer,
            glb: glb,
            gpu: gpu,
        };
        let size = window.inner_size();
        this.resize(size.width, size.height);

        Ok(this)
    }

    pub fn resize(&mut self, w: u32, h: u32) {
        let w = w.max(1);
        let h = h.max(1);
        self.surface.configure(
            &self.device,
            &self.surface.get_default_config(&self.adapter, w, h).unwrap(),
        );
        self.renderer.resize(&self.device, w, h);
    }

    pub fn render(&self) {
        let frame = self.surface.get_current_texture().unwrap();
        let view = frame.texture.create_view(&Default::default());
        let mut encoder = self.device.create_command_encoder(&Default::default());

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.renderer.depth_texture_view, // XXX
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });
            self.renderer.render(&mut pass, &self.glb, &self.gpu);
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}

fn main() -> Result<(), Box<dyn error::Error>> {
    //node::test();
    /*
    for entry in fs::read_dir("models/")? {
        let _glb = loader::load(fs::File::open(entry?.path())?)?;
        //dbg!(_glb.accessors);
    }
    */

    let looper = winit::event_loop::EventLoop::new()?;
    let window = winit::window::WindowBuilder::new()
        .with_transparent(true)
        .with_decorations(false)
        .with_window_level(winit::window::WindowLevel::AlwaysOnTop)
        .build(&looper)?;
    let mut app = App::new(&window)?;

    looper.run(|ev, elwt| {
        use winit::event::*;
        match ev {
            Event::WindowEvent { event, window_id } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => elwt.exit(),
                WindowEvent::Resized(size) => app.resize(size.width, size.height),
                WindowEvent::RedrawRequested => app.render(),
                _ => (),
            },
            _ => (),
        }
    })?;

    Ok(())
}
