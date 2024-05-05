use nalgebra::{Matrix4, UnitQuaternion, Vector3};
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
    pub weights: Option<Vec<f32>>,
}

#[derive(Debug)]
pub enum Element {
    None,
    Mesh(usize),
}

#[derive(Debug)]
pub struct Node {
    pub name: String,
    pub children: Vec<usize>,
    pub translation: Vector3<f32>,
    pub rotation: UnitQuaternion<f32>,
    pub scale: Vector3<f32>,
    pub element: Element,
}

#[derive(Debug)]
pub struct Image {
    pub dims: [u32; 3],
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

impl default::Default for Node {
    fn default() -> Self {
        Self {
            name: String::new(),
            children: Vec::new(),
            translation: nalgebra::zero(),
            rotation: UnitQuaternion::identity(),
            scale: Vector3::new(1.0, 1.0, 1.0),
            element: Element::None,
        }
    }
}

impl Node {
    pub fn transform(&self) -> Matrix4<f32> {
        let mt = Matrix4::new_translation(&self.translation);
        let mr = self.rotation.to_homogeneous();
        let ms = Matrix4::new_nonuniform_scaling(&self.scale);
        mt * mr * ms
    }
}
