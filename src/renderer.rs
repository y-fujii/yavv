use crate::*;
use nalgebra::Matrix4;

pub struct Renderer {
    sample_count: u32,
    viewport_scale: [f32; 2],
    pipeline: wgpu::RenderPipeline,
    color_texture: wgpu::Texture,
    color_texture_view: wgpu::TextureView,
    depth_texture: wgpu::Texture,
    depth_texture_view: wgpu::TextureView,
    gpu: gpu_resource::GpuResource,
}

#[repr(C)]
struct VsConstants {
    transform: [[f32; 4]; 4],
    projection: [[f32; 4]; 4],
}

impl Renderer {
    pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

    pub fn new(device: &wgpu::Device, sample_count: u32) -> Result<Self, Box<dyn error::Error>> {
        let gpu = gpu_resource::GpuResource::new(&device);

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&gpu.material_layout],
            push_constant_ranges: &[wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::VERTEX,
                range: 0..mem::size_of::<VsConstants>() as u32,
            }],
        });
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &gpu.vertex_layouts(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: Self::FORMAT,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: Default::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Greater,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: sample_count,
                mask: !0,
                alpha_to_coverage_enabled: true,
            },
            multiview: None,
        });

        let (color_tex, color_view, depth_tex, depth_view) = Self::create_textures(device, 1, 1, sample_count);

        Ok(Renderer {
            sample_count: sample_count,
            viewport_scale: [1.0, 1.0],
            pipeline: pipeline,
            color_texture: color_tex,
            color_texture_view: color_view,
            depth_texture: depth_tex,
            depth_texture_view: depth_view,
            gpu: gpu,
        })
    }

    pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, glb: &scene::Glb) {
        self.gpu.update(&device, &queue, &glb);
    }

    pub fn resize(&mut self, device: &wgpu::Device, w: u32, h: u32) {
        let wf = w as f32;
        let hf = h as f32;
        let nf = f32::sqrt(wf * hf);
        self.viewport_scale = [nf / wf, nf / hf];

        let (color_tex, color_view, depth_tex, depth_view) = Self::create_textures(device, w, h, self.sample_count);
        self.color_texture = color_tex;
        self.color_texture_view = color_view;
        self.depth_texture = depth_tex;
        self.depth_texture_view = depth_view;
    }

    pub fn render<'a>(
        &'a self,
        encoder: &mut wgpu::CommandEncoder,
        glb: &scene::Glb,
        view: &wgpu::TextureView,
        camera: &scene::Node,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.color_texture_view,
                resolve_target: Some(view),
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(0.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            ..Default::default()
        });
        pass.set_pipeline(&self.pipeline);

        let camera_matrix = camera.transform().try_inverse().unwrap();
        for n in glb.roots.iter() {
            self.render_nodes(&mut pass, glb, *n, &camera_matrix);
        }
    }

    pub fn render_nodes<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        glb: &scene::Glb,
        root: usize,
        transform: &Matrix4<f32>,
    ) {
        let root_node = &glb.nodes[root];
        let transform = transform * root_node.transform();
        if let Some(mesh) = root_node.mesh {
            let projection = Matrix4::from([
                [self.viewport_scale[0], 0.0, 0.0, 0.0],
                [0.0, self.viewport_scale[1], 0.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0, 0.0],
            ]);
            let buf = VsConstants {
                transform: *transform.as_ref(),
                projection: *projection.as_ref(),
            };
            pass.set_push_constants(wgpu::ShaderStages::VERTEX, 0, unsafe { utils::as_bytes(&buf) });
            self.gpu.draw_mesh(pass, glb, mesh, 0);
        }
        for n in root_node.children.iter() {
            self.render_nodes(pass, glb, *n, &transform);
        }
    }

    fn create_textures(
        device: &wgpu::Device,
        w: u32,
        h: u32,
        sample_count: u32,
    ) -> (wgpu::Texture, wgpu::TextureView, wgpu::Texture, wgpu::TextureView) {
        let size = wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        };

        let color_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: size,
            mip_level_count: 1,
            sample_count: sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: Self::FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let color_view = color_tex.create_view(&wgpu::TextureViewDescriptor::default());

        let depth_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: size,
            mip_level_count: 1,
            sample_count: sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth_view = depth_tex.create_view(&wgpu::TextureViewDescriptor::default());

        (color_tex, color_view, depth_tex, depth_view)
    }
}
