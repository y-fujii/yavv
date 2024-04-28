use crate::*;
use collections::HashMap;
use nalgebra::{Quaternion, UnitQuaternion, Vector3};

fn u32_from_slice(buf: &[u8]) -> u32 {
    u32::from_le_bytes(buf[..4].try_into().unwrap())
}

fn get_usize(json: &tinyjson::JsonValue) -> Option<usize> {
    json.get::<f64>().map(|x| *x as usize)
}

fn get_vec32f<const N: usize>(json: &tinyjson::JsonValue) -> Option<[f32; N]> {
    let json: &Vec<_> = json.get()?;
    if json.len() != N {
        return None;
    }
    let mut dst = [0.0; N];
    for i in 0..N {
        dst[i] = *json[i].get::<f64>()? as f32;
    }
    Some(dst)
}

fn load_attributes(json_attributes: &tinyjson::JsonValue) -> Option<scene::Attributes> {
    let json_attributes: &HashMap<_, _> = json_attributes.get()?;
    let position = match json_attributes.get("POSITION") {
        Some(e) => Some(get_usize(e)?),
        None => None,
    };
    let normal = match json_attributes.get("NORMAL") {
        Some(e) => Some(get_usize(e)?),
        None => None,
    };
    let texcoord_0 = match json_attributes.get("TEXCOORD_0") {
        Some(e) => Some(get_usize(e)?),
        None => None,
    };
    let texcoord_1 = match json_attributes.get("TEXCOORD_1") {
        Some(e) => Some(get_usize(e)?),
        None => None,
    };
    Some(scene::Attributes {
        position: position,
        normal: normal,
        texcoord_0: texcoord_0,
        texcoord_1: texcoord_1,
    })
}

fn load_root(json_root: &tinyjson::JsonValue, blob: Vec<u8>) -> Option<scene::Glb> {
    let json_root: &HashMap<_, _> = json_root.get()?;

    let mut views = Vec::new();
    for json_views in json_root.get("bufferViews")?.get::<Vec<_>>()? {
        let json_views: &HashMap<_, _> = json_views.get()?;
        let offset = match json_views.get("byteOffset") {
            Some(e) => get_usize(e)?,
            None => 0,
        };
        let length = match json_views.get("byteLength") {
            Some(e) => get_usize(e)?,
            None => 0,
        };
        let stride = match json_views.get("byteStride") {
            Some(e) => Some(get_usize(e)?),
            None => None,
        };

        views.push((offset, length, stride));
    }

    let mut accessors = Vec::new();
    for json_accessors in json_root.get("accessors")?.get::<Vec<_>>()? {
        let json_accessors: &HashMap<_, _> = json_accessors.get()?;
        let view = get_usize(json_accessors.get("bufferView")?)?;
        let offset = match json_accessors.get("byteOffset") {
            Some(e) => get_usize(e)?,
            None => 0,
        };
        let count = get_usize(json_accessors.get("count")?)?;
        let component_type = get_usize(json_accessors.get("componentType")?)?;
        let component_count = match json_accessors.get("type")?.get::<String>()?.as_str() {
            "SCALAR" => 1,
            "VEC2" => 2,
            "VEC3" => 3,
            "VEC4" => 4,
            "MAT4" => 16,
            _ => return None,
        };
        accessors.push(scene::Accessor {
            offset: views[view].0 + offset,
            count: count,
            stride: views[view].2,
            component_type: component_type,
            component_count: component_count,
        });
    }

    let mut meshes = Vec::new();
    for json_mesh in json_root.get("meshes")?.get::<Vec<_>>()? {
        let json_mesh: &HashMap<_, _> = json_mesh.get()?;
        let mut primitives = Vec::new();
        for json_primitive in json_mesh.get("primitives")?.get::<Vec<_>>()? {
            let json_primitive: &HashMap<_, _> = json_primitive.get()?;
            let attributes = load_attributes(json_primitive.get("attributes")?)?;
            let targets = match json_primitive.get("targets") {
                Some(json_targets) => {
                    let mut targets = Vec::new();
                    for json_target in json_targets.get::<Vec<_>>()? {
                        targets.push(load_attributes(json_target)?);
                    }
                    targets
                }
                None => Vec::new(),
            };
            let indices = match json_primitive.get("indices") {
                Some(e) => Some(get_usize(e)?),
                None => None,
            };
            let material = match json_primitive.get("material") {
                Some(e) => Some(get_usize(e)?),
                None => None,
            };
            primitives.push(scene::Primitive {
                attributes: attributes,
                targets: targets,
                indices: indices,
                material: material,
            });
        }
        let weights = match json_mesh.get("weights") {
            Some(json_weights) => {
                let mut weights = Vec::new();
                for json_weight in json_weights.get::<Vec<_>>()? {
                    weights.push(*json_weight.get::<f64>()? as f32);
                }
                Some(weights)
            }
            None => None,
        };
        meshes.push(scene::Mesh {
            primitives: primitives,
            weights: weights,
        });
    }

    let mut nodes = Vec::new();
    for json_node in json_root.get("nodes")?.get::<Vec<_>>()? {
        let json_node: &HashMap<_, _> = json_node.get()?;
        let name = match json_node.get("name") {
            Some(e) => &e.get::<String>()?,
            None => "",
        };
        let children = match json_node.get("children") {
            Some(json_children) => {
                let mut children = Vec::new();
                for json_child in json_children.get::<Vec<_>>()? {
                    children.push(get_usize(json_child)?);
                }
                children
            }
            None => Vec::new(),
        };
        let translation = match json_node.get("translation") {
            Some(e) => get_vec32f(e)?,
            None => [0.0, 0.0, 0.0],
        };
        let rotation = match json_node.get("rotation") {
            Some(e) => get_vec32f(e)?,
            None => [0.0, 0.0, 0.0, 1.0],
        };
        let scale = match json_node.get("scale") {
            Some(e) => get_vec32f(e)?,
            None => [1.0, 1.0, 1.0],
        };
        let mesh = match json_node.get("mesh") {
            Some(e) => Some(get_usize(e)?),
            None => None,
        };
        nodes.push(scene::Node {
            name: name.to_string(),
            children: children,
            translation: Vector3::from(translation),
            rotation: UnitQuaternion::from_quaternion(Quaternion::from(rotation)),
            scale: Vector3::from(scale),
            mesh: mesh,
        });
    }

    let mut roots = Vec::new();
    for json_scene in json_root.get("scenes")?.get::<Vec<_>>()? {
        let json_scene: &HashMap<_, _> = json_scene.get()?;
        for node in json_scene.get("nodes")?.get::<Vec<_>>()? {
            roots.push(get_usize(node)?);
        }
    }

    let mut images = Vec::new();
    if let Some(json_images) = json_root.get("images") {
        for json_image in json_images.get::<Vec<_>>()? {
            let json_image: &HashMap<_, _> = json_image.get()?;
            let image = match json_image.get("bufferView") {
                Some(view) => {
                    let (offset, length, _) = *views.get(get_usize(view)?)?;
                    /*
                    let image = zune_image::image::Image::read(
                        &blob[offset..offset + length],
                        zune_core::options::DecoderOptions::new_fast()
                            .png_set_add_alpha_channel(true)
                            .jpeg_set_out_colorspace(zune_core::colorspace::ColorSpace::RGBA),
                    )
                    .ok()?;
                    let frame = image.frames_ref().get(0)?;
                    Some(scene::Image {
                        dims: [image.dimensions().0 as u32, image.dimensions().1 as u32, 4],
                        buffer: frame.flatten(zune_core::colorspace::ColorSpace::RGBA),
                    })
                    */
                    let image = image::load_from_memory(&blob[offset..offset + length]).ok()?;
                    Some(scene::Image {
                        dims: [image.width(), image.height(), 4],
                        buffer: image.into_rgba8().into_vec(),
                    })
                }
                None => None,
            };
            images.push(image);
        }
    }

    let mut textures = Vec::new();
    if let Some(json_textures) = json_root.get("textures") {
        for json_texture in json_textures.get::<Vec<_>>()? {
            let json_texture: &HashMap<_, _> = json_texture.get()?;
            // XXX: sampler.
            let source = get_usize(json_texture.get("source")?)?;
            textures.push(source);
        }
    }

    let mut materials = Vec::new();
    if let Some(json_materials) = json_root.get("materials") {
        for json_material in json_materials.get::<Vec<_>>()? {
            let json_material: &HashMap<_, _> = json_material.get()?;
            let material = match json_material.get("pbrMetallicRoughness") {
                Some(json_pbr) => {
                    let json_pbr: &HashMap<_, _> = json_pbr.get()?;
                    let base_color_factor = match json_pbr.get("baseColorFactor") {
                        Some(e) => get_vec32f(e)?,
                        None => [1.0, 1.0, 1.0, 1.0],
                    };
                    let base_color_texture = match json_pbr.get("baseColorTexture") {
                        Some(e) => {
                            let e: &HashMap<_, _> = e.get()?;
                            let index = get_usize(e.get("index")?)?;
                            let texcoord = match e.get("texCoord") {
                                Some(e) => get_usize(e)?,
                                None => 0,
                            };
                            Some(scene::Texture {
                                // XXX
                                wrap_s: true,
                                wrap_t: true,
                                texcoord: texcoord,
                                image: textures[index],
                            })
                        }
                        None => None,
                    };
                    scene::Material {
                        base_color_factor: base_color_factor,
                        base_color_texture: base_color_texture,
                    }
                }
                None => scene::Material {
                    base_color_factor: [1.0, 1.0, 1.0, 1.0],
                    base_color_texture: None,
                },
            };
            materials.push(material);
        }
    }

    Some(scene::Glb {
        materials: materials,
        accessors: accessors,
        meshes: meshes,
        nodes: nodes,
        roots: roots,
        blob: blob,
        images: images,
    })
}

pub fn load(mut f: impl io::Read) -> Result<scene::Glb, Box<dyn error::Error>> {
    let mut header = [0; 12 + 8];
    f.read_exact(&mut header)?;
    if header[0..4] != *b"glTF" {
        return Err("".into());
    }
    if u32_from_slice(&header[4..]) != 2 {
        return Err("".into());
    }

    let chunk_len = u32_from_slice(&header[12..]);
    if header[16..20] != *b"JSON" {
        return Err("".into());
    }
    let mut buf = vec![0; chunk_len as usize];
    f.read_exact(&mut buf)?;
    let json = tinyjson::JsonParser::new(str::from_utf8(&buf)?.chars()).parse()?;

    let mut chunk_header = [0; 8];
    f.read_exact(&mut chunk_header)?;
    let chunk_len = u32_from_slice(&chunk_header[0..]);
    if chunk_header[4..8] != *b"BIN\0" {
        return Err("".into());
    }
    let mut blob = vec![0; chunk_len as usize];
    f.read_exact(&mut blob)?;

    let gltf = load_root(&json, blob).ok_or("")?;

    Ok(gltf)
}
