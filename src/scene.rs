use std::*;

#[derive(Debug)]
pub struct Accessor {
    pub offset: usize,
    pub count: usize,
    pub stride: Option<usize>,
    pub component_type: usize,
    pub component_count: usize,
}

#[derive(Debug)]
pub struct Attributes {
    pub position: Option<usize>,
    pub normal: Option<usize>,
    //pub tangent: Option<usize>,
    pub texcoord_0: Option<usize>,
    pub texcoord_1: Option<usize>,
    //pub color_0: Option<usize>,
    //pub joints_0: Option<usize>,
    //pub weights_0: Option<usize>,
}

#[derive(Debug)]
pub struct Texture {
    pub wrap_s: bool,
    pub wrap_t: bool,
    pub texcoord: usize,
    pub image: usize,
}

#[derive(Debug)]
pub struct Material {
    pub base_color_factor: [f32; 4],
    pub base_color_texture: Option<Texture>,
}

#[derive(Debug)]
pub struct Primitive {
    pub attributes: Attributes,
    pub targets: Vec<Attributes>,
    pub indices: Option<usize>,
    pub material: Option<usize>,
}

#[derive(Debug)]
pub struct Mesh {
    pub primitives: Vec<Primitive>,
    //pub weights: Vec<f32>,
}

#[derive(Debug)]
pub struct Node {
    pub name: String,
    pub children: Vec<usize>,
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
    pub mesh: Option<usize>,
}

#[derive(Debug)]
pub struct Image {
    pub dims: [u32; 2],
    pub depth: usize,
    pub buffer: Vec<u8>,
}

pub struct Glb {
    pub materials: Vec<Material>,
    pub accessors: Vec<Accessor>,
    pub meshes: Vec<Mesh>,
    pub nodes: Vec<Node>,
    pub roots: Vec<usize>,
    pub blob: Vec<u8>,
    pub images: Vec<Option<Image>>,
}
