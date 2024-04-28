use crate::*;
use wgpu::util::DeviceExt;

#[repr(C, align(16))]
struct MaterialUniform {
    base_color_factor: [f32; 4],
    base_color_texcoord: u32,
}

pub struct GpuResource {
    pub material_layout: wgpu::BindGroupLayout,
    pub sampler: wgpu::Sampler,
    pub blob: Option<wgpu::Buffer>,
    pub images: Vec<Option<(wgpu::TextureView, wgpu::Texture)>>,
    pub materials: Vec<(wgpu::BindGroup, wgpu::Buffer)>,
}

impl GpuResource {
    pub fn new(device: &wgpu::Device) -> Self {
        let material_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: None,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        GpuResource {
            material_layout: material_layout,
            sampler: sampler,
            blob: None,
            images: Vec::new(),
            materials: Vec::new(),
        }
    }

    pub fn vertex_layouts(&self) -> [wgpu::VertexBufferLayout; 4] {
        [
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
            wgpu::VertexBufferLayout {
                array_stride: 4 * 2,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x2,
                }],
            },
        ]
    }

    pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, scene: &scene::Glb) {
        self.blob = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: &scene.blob,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::INDEX,
        }));

        self.images.clear();
        for image in scene.images.iter() {
            let image = match image {
                Some(image) => {
                    let size = wgpu::Extent3d {
                        width: image.dims[0],
                        height: image.dims[1],
                        depth_or_array_layers: 1,
                    };
                    let texture = device.create_texture(&wgpu::TextureDescriptor {
                        size: size,
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: wgpu::TextureFormat::Rgba8UnormSrgb,
                        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                        label: None,
                        view_formats: &[],
                    });
                    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

                    queue.write_texture(
                        wgpu::ImageCopyTexture {
                            texture: &texture,
                            mip_level: 0,
                            origin: wgpu::Origin3d::ZERO,
                            aspect: wgpu::TextureAspect::All,
                        },
                        &image.buffer,
                        wgpu::ImageDataLayout {
                            offset: 0,
                            bytes_per_row: Some(4 * image.dims[0]),
                            rows_per_image: Some(image.dims[1]),
                        },
                        size,
                    );

                    Some((view, texture))
                }
                None => None,
            };
            self.images.push(image);
        }

        self.materials.clear();
        for material in scene.materials.iter() {
            let uniform = MaterialUniform {
                base_color_factor: material.base_color_factor,
                base_color_texcoord: material.base_color_texture.as_ref().unwrap().texcoord as u32,
            };
            let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: unsafe { utils::as_bytes(&uniform) },
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
            let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.material_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(
                            &self.images[material.base_color_texture.as_ref().unwrap().image]
                                .as_ref()
                                .unwrap()
                                .0,
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
                label: None,
            });
            self.materials.push((group, buffer));
        }
    }

    pub fn draw_mesh<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>, glb: &scene::Glb, mesh: usize, material_id: u32) {
        let blob = self.blob.as_ref().unwrap();
        for primitive in glb.meshes[mesh].primitives.iter() {
            let Some(position) = primitive.attributes.position else {
                continue;
            };
            let Some(normal) = primitive.attributes.normal else {
                continue;
            };
            let texcoord_0 = match primitive.attributes.texcoord_0 {
                Some(texcoord_0) => texcoord_0,
                None => position, // dummy.
            };
            let texcoord_1 = match primitive.attributes.texcoord_1 {
                Some(texcoord_1) => texcoord_1,
                None => position, // dummy.
            };
            let Some(indices) = primitive.indices else { continue };
            let index_fmt = match glb.accessors[indices].component_type {
                5123 => wgpu::IndexFormat::Uint16,
                5125 => wgpu::IndexFormat::Uint32,
                _ => continue,
            };
            let Some(material) = primitive.material else { continue };
            pass.set_bind_group(material_id, &self.materials[material].0, &[]);
            pass.set_vertex_buffer(0, blob.slice(glb.accessors[position].offset as u64..));
            pass.set_vertex_buffer(1, blob.slice(glb.accessors[normal].offset as u64..));
            pass.set_vertex_buffer(2, blob.slice(glb.accessors[texcoord_0].offset as u64..));
            pass.set_vertex_buffer(3, blob.slice(glb.accessors[texcoord_1].offset as u64..));
            pass.set_index_buffer(blob.slice(glb.accessors[indices].offset as u64..), index_fmt);
            pass.draw_indexed(0..glb.accessors[indices].count as u32, 0, 0..1);
        }
    }
}
