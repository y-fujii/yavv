use crate::*;
use wgpu::util::DeviceExt;

#[repr(C)]
struct MaterialUniform {
    base_color_factor: [f32; 4],
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
            address_mode_w: wgpu::AddressMode::Repeat,
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
}
