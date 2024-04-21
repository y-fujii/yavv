use crate::*;
use nalgebra::{Matrix4, Quaternion, UnitQuaternion, Vector3};

pub struct Renderer {
    viewport_scale: [f32; 2],
    pipeline: wgpu::RenderPipeline,
    _empty_buffer: wgpu::Buffer,
    // XXX
    depth_texture: wgpu::Texture,
    pub depth_texture_view: wgpu::TextureView,
}

#[repr(C)]
struct VsConstants {
    projection: [[f32; 4]; 4],
}

impl Renderer {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        gpu: &gpu_resource::GpuResource,
    ) -> Result<Self, Box<dyn error::Error>> {
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
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: 4 * 3,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x3,
                        }],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: 4 * 3,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x3,
                        }],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: 4 * 2,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 2,
                            format: wgpu::VertexFormat::Float32x2,
                        }],
                    },
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(format.into())],
            }),
            primitive: Default::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: Default::default(),
            multiview: None,
        });

        let empty_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 0,
            usage: wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        let (depth_texture, depth_texture_view) = Self::create_depth_texture(device, 1, 1);

        Ok(Renderer {
            viewport_scale: [1.0, 1.0],
            pipeline: pipeline,
            _empty_buffer: empty_buffer,
            depth_texture: depth_texture,
            depth_texture_view: depth_texture_view,
        })
    }

    pub fn resize(&mut self, device: &wgpu::Device, w: u32, h: u32) {
        let wf = w as f32;
        let hf = h as f32;
        let nf = f32::sqrt(wf * hf);
        self.viewport_scale = [nf / wf, nf / hf];

        let (depth_texture, depth_texture_view) = Self::create_depth_texture(device, w, h);
        self.depth_texture = depth_texture;
        self.depth_texture_view = depth_texture_view;
    }

    pub fn render<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>, glb: &scene::Glb, gpu: &'a gpu_resource::GpuResource) {
        pass.set_pipeline(&self.pipeline);
        for n in glb.roots.iter() {
            self.render_nodes(pass, glb, gpu, *n, &Matrix4::identity());
        }
    }

    pub fn render_nodes<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        glb: &scene::Glb,
        gpu: &'a gpu_resource::GpuResource,
        root: usize,
        transform: &Matrix4<f32>,
    ) {
        let root_node = &glb.nodes[root];
        let mt = Matrix4::new_translation(&Vector3::from(root_node.translation));
        let mr = UnitQuaternion::from_quaternion(Quaternion::from(root_node.rotation)).to_homogeneous();
        let ms = Matrix4::new_nonuniform_scaling(&Vector3::from(root_node.scale));
        let transform = transform * mt * mr * ms;
        if let Some(mesh) = root_node.mesh {
            self.render_mesh(pass, glb, gpu, mesh, &transform);
        }
        for n in root_node.children.iter() {
            self.render_nodes(pass, glb, gpu, *n, &transform);
        }
    }

    pub fn render_mesh<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        glb: &scene::Glb,
        gpu: &'a gpu_resource::GpuResource,
        mesh: usize,
        transform: &Matrix4<f32>,
    ) {
        for primitive in glb.meshes[mesh].primitives.iter() {
            let Some(position) = primitive.attributes.position else {
                continue;
            };
            let Some(normal) = primitive.attributes.normal else {
                continue;
            };
            let Some(texcoord_0) = primitive.attributes.texcoord_0 else {
                continue;
            };
            let Some(indices) = primitive.indices else { continue };
            let index_fmt = match glb.accessors[indices].component_type {
                5123 => wgpu::IndexFormat::Uint16,
                5125 => wgpu::IndexFormat::Uint32,
                _ => continue,
            };
            pass.set_bind_group(0, &gpu.materials[primitive.material.unwrap()].0, &[]);
            let blob = gpu.blob.as_ref().unwrap();
            pass.set_vertex_buffer(0, blob.slice(glb.accessors[position].offset as u64..));
            pass.set_vertex_buffer(1, blob.slice(glb.accessors[normal].offset as u64..));
            pass.set_vertex_buffer(2, blob.slice(glb.accessors[texcoord_0].offset as u64..));
            pass.set_index_buffer(blob.slice(glb.accessors[indices].offset as u64..), index_fmt);

            let projection = Matrix4::from([
                [self.viewport_scale[0], 0.0, 0.0, 0.0],
                [0.0, self.viewport_scale[1], 0.0, 0.0],
                [0.0, 0.0, 0.1, 0.0],
                [0.0, 0.0, 0.5, 1.0],
            ]);
            let camera_inv = Matrix4::from([
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, -1.0, 0.0, 1.0],
            ]);
            let buf = VsConstants {
                projection: *(projection * camera_inv * transform).as_ref(),
            };
            pass.set_push_constants(wgpu::ShaderStages::VERTEX, 0, unsafe { utils::as_bytes(&buf) });

            pass.draw_indexed(0..glb.accessors[indices].count as u32, 0, 0..1);
        }
    }

    fn create_depth_texture(device: &wgpu::Device, w: u32, h: u32) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (texture, view)
    }
}
