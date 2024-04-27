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
}

impl<'a> App<'a> {
    pub fn new(window: &'a winit::window::Window, glb: scene::Glb) -> Result<Self, Box<dyn error::Error>> {
        /*
        let instance = wgpu::Instance::default();
        */
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::DX12,
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
                required_features: wgpu::Features::PUSH_CONSTANTS, // | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
                required_limits: wgpu::Limits {
                    max_push_constant_size: 128,
                    ..Default::default()
                },
            },
            None,
        ))?;

        let mut renderer = renderer::Renderer::new(&device, 4)?;
        renderer.update(&device, &queue, &glb);

        Ok(App {
            surface: surface,
            adapter: adapter,
            device: device,
            queue: queue,
            renderer: renderer,
            glb: glb,
        })
    }

    pub fn resize(&mut self, w: u32, h: u32) {
        let w = w.max(1);
        let h = h.max(1);
        let config = wgpu::SurfaceConfiguration {
            format: renderer::Renderer::FORMAT,
            ..self.surface.get_default_config(&self.adapter, w, h).unwrap()
        };
        self.surface.configure(&self.device, &config);
        self.renderer.resize(&self.device, w, h);
    }

    pub fn render(&self) {
        let frame = self.surface.get_current_texture().unwrap();
        let view = frame.texture.create_view(&Default::default());
        let mut encoder = self.device.create_command_encoder(&Default::default());
        self.renderer.render(&mut encoder, &self.glb, &view);
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

    //let glb = loader::load(fs::File::open("models/AliciaSolid_vrm-0.51.vrm")?)?;
    //let glb = loader::load(fs::File::open("models/vroid_10.vrm")?)?;
    let glb = loader::load(fs::File::open("../Kyoko_Original.vrm")?)?;
    //let glb = loader::load(fs::File::open("models/hakka.vrm")?)?;

    let looper = winit::event_loop::EventLoop::new()?;
    let window = winit::window::WindowBuilder::new()
        .with_visible(false)
        //.with_transparent(true)
        .build(&looper)?;
    let mut app = App::new(&window, glb)?;

    let mut time = time::Instant::now();
    looper.run(|ev, elwt| {
        use winit::event::*;
        match ev {
            Event::NewEvents(winit::event::StartCause::Init) => {
                window.set_visible(true);
            }
            Event::WindowEvent { event, window_id } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => elwt.exit(),
                WindowEvent::Resized(size) => app.resize(size.width, size.height),
                WindowEvent::RedrawRequested => {
                    println!("{:?}", time.elapsed());
                    time = time::Instant::now();
                    app.render();
                    window.request_redraw();
                }
                _ => (),
            },
            _ => (),
        }
    })?;

    Ok(())
}
