use crate::material::PBRMaterial;
use crate::math::Vector3f;
use crate::node::camera::Camera;
use core::fmt;
use elsa::FrozenVec;
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Display;
use std::mem::transmute;
use std::rc::Rc;

pub mod primitive;

pub enum ShapeType {
    Sphere,
    Cube,
    CubeFrame,
    Torus,
    DeathStar,
    Helix,
}

impl ShapeType {
    pub fn to_index(&self) -> i32 {
        match self {
            ShapeType::Sphere => 0,
            ShapeType::Cube => 1,
            ShapeType::CubeFrame => 2,
            ShapeType::Torus => 3,
            ShapeType::DeathStar => 4,
            ShapeType::Helix => 5,
        }
    }
}

impl Display for ShapeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShapeType::Sphere => write!(f, "Sphere"),
            ShapeType::Cube => write!(f, "Cube"),
            ShapeType::CubeFrame => write!(f, "CubeFrame"),
            ShapeType::Torus => write!(f, "Torus"),
            ShapeType::DeathStar => write!(f, "DeathStar"),
            ShapeType::Helix => write!(f, "Helix"),
        }
    }
}

pub trait Shape: Send + Sync + Display + Any {
    fn shape_type(&self) -> ShapeType;
    fn to_bytes(&self) -> [u8; 32] {
        [0; 32]
    }
}

pub enum ShapeOpType {
    Nop,
    Union,
    Subtraction,
    Intersection,
    SmoothUnion,
}

impl ShapeOpType {
    pub fn to_index(&self) -> i32 {
        match self {
            ShapeOpType::Nop => 0,
            ShapeOpType::Union => 1,
            ShapeOpType::Subtraction => 2,
            ShapeOpType::Intersection => 3,
            ShapeOpType::SmoothUnion => 4,
        }
    }
}

impl Display for ShapeOpType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShapeOpType::Union => write!(f, "Union"),
            ShapeOpType::Subtraction => write!(f, "Subtraction"),
            ShapeOpType::Intersection => write!(f, "Intersection"),
            ShapeOpType::SmoothUnion => write!(f, "SmoothUnion"),
            ShapeOpType::Nop => write!(f, "Nop"),
        }
    }
}

pub struct ShapeOp<'a> {
    pub index: i32,
    pub shape: Box<dyn Shape>,
    pub op: ShapeOpType,
    pub material: Rc<PBRMaterial>,
    pub next: Option<&'a ShapeOp<'a>>,
}

impl<'a> ShapeOp<'a> {
    pub fn to_bytes(&self) -> [u8; 48] {
        let type_index: i32 = self.shape.shape_type().to_index();
        let material_index = self.material.get_index();
        let op_index = self.op.to_index();
        let next_index = if let Some(next) = self.next {
            next.index
        } else {
            -1
        };
        let mut bytes = [0u8; 48];
        let type_bytes = type_index.to_le_bytes();
        let material_bytes = material_index.to_le_bytes();
        let op_index_bytes = op_index.to_le_bytes();
        let next_index_bytes = next_index.to_le_bytes();
        let data_bytes = self.shape.to_bytes();
        bytes[0..4].copy_from_slice(&type_bytes);
        bytes[4..8].copy_from_slice(&material_bytes);
        bytes[8..12].copy_from_slice(&op_index_bytes);
        bytes[12..16].copy_from_slice(&next_index_bytes);
        bytes[16..48].copy_from_slice(&data_bytes);
        bytes
    }
}

impl<'a> Display for ShapeOp<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(op) = self.next {
            write!(f, "{} ==> {}", self.shape, op)
        } else {
            write!(f, "{}", self.shape)
        }
    }
}

pub struct HitResult<'a> {
    pub distance: f64,
    pub shape_op: Option<&'a ShapeOp<'a>>,
}

impl<'a> HitResult<'a> {
    pub fn new() -> HitResult<'a> {
        HitResult {
            distance: f64::MAX,
            shape_op: None,
        }
    }
}

impl<'a> Display for HitResult<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(op) = self.shape_op {
            write!(
                f,
                "HitResult {{ distance: {}, shape: {} }}",
                self.distance, op
            )
        } else {
            write!(
                f,
                "HitResult {{ distance: {}, shape: None }}",
                self.distance
            )
        }
    }
}

impl<'a> Default for HitResult<'a> {
    fn default() -> Self {
        HitResult::new()
    }
}

pub struct DirectionalLight {
    pub direction: Vector3f,
    pub color: Vector3f,
}

pub struct Scene<'a> {
    pub nodes: FrozenVec<Box<ShapeOp<'a>>>,
    pub root_nodes: FrozenVec<&'a ShapeOp<'a>>,
    pub background_color: Vector3f,
    pub width: u32,
    pub height: u32,
    pub camera: Camera,
    pub main_light: DirectionalLight,

    // material
    material2index: RefCell<HashMap<u64, i32>>,
    materials: RefCell<Vec<Rc<PBRMaterial>>>,
}

impl<'a> Scene<'a> {
    pub fn new(
        width: u32,
        height: u32,
        camera: Camera,
        background_color: Vector3f,
        main_light: DirectionalLight,
    ) -> Scene<'a> {
        Scene {
            nodes: FrozenVec::new(),
            root_nodes: FrozenVec::new(),
            material2index: RefCell::new(HashMap::new()),
            materials: RefCell::new(Vec::new()),
            background_color,
            width,
            height,
            camera,
            main_light,
        }
    }

    pub fn add_leaf_node(
        &'a self,
        shape: Box<dyn Shape>,
        material: Rc<PBRMaterial>,
    ) -> &'a ShapeOp<'a> {
        let idx = self.nodes.len();
        self.nodes.push(Box::new(ShapeOp {
            index: idx as i32,
            material: Rc::clone(&material),
            shape,
            op: ShapeOpType::Nop,
            next: None,
        }));
        self.add_material(Rc::clone(&material));
        &self.nodes[idx]
    }

    pub fn add_node(
        &'a self,
        shape: Box<dyn Shape>,
        material: Rc<PBRMaterial>,
        op: ShapeOpType,
        next: Option<&'a ShapeOp<'a>>,
    ) -> &'a ShapeOp<'a> {
        let idx = self.nodes.len();
        self.nodes.push(Box::new(ShapeOp {
            index: idx as i32,
            material: Rc::clone(&material),
            shape,
            op,
            next,
        }));
        self.add_material(Rc::clone(&material));
        &self.nodes[idx]
    }

    pub fn add_root_node(&'a self, node: &'a ShapeOp<'a>) {
        self.root_nodes.push(node);
    }

    pub fn get_scene_bytes(&'a self) -> Box<[u8]> {
        let mut buffer: Vec<u8> = Vec::new();
        unsafe {
            let pad4: [u8; 4] = [0; 4];

            // background color
            let background_color_bytes: [u8; 12] = transmute(self.background_color.to32());
            buffer.extend_from_slice(&background_color_bytes);
            buffer.extend_from_slice(&pad4);

            // main light
            let light_dir: [u8; 12] = transmute(self.main_light.direction.to32());
            let light_color: [u8; 12] = transmute(self.main_light.color.to32());
            buffer.extend_from_slice(&light_dir);
            buffer.extend_from_slice(&pad4);
            buffer.extend_from_slice(&light_color);
            buffer.extend_from_slice(&pad4);
        }

        // add root indices
        let sentinel: i32 = -1;
        let sentinel_bytes = sentinel.to_le_bytes();
        if !self.root_nodes.is_empty() {
            for node in self.root_nodes.iter() {
                let index = node.index.to_le_bytes();
                buffer.extend_from_slice(&index);
            }
            buffer.extend_from_slice(&sentinel_bytes);
        } else {
            buffer.extend_from_slice(&sentinel_bytes);
        }
        buffer.into_boxed_slice()
    }

    pub fn get_shape_bytes(&'a self) -> Box<[u8]> {
        let mut buffer: Vec<u8> = Vec::new();
        if !self.nodes.is_empty() {
            for node in self.nodes.iter() {
                let node_bytes: [u8; 48] = node.to_bytes();
                buffer.extend_from_slice(&node_bytes);
            }
        } else {
            let empty: [u8; 48] = [0; 48];
            buffer.extend_from_slice(&empty);
        }
        buffer.into_boxed_slice()
    }

    pub fn get_materials_bytes(&self) -> Box<[u8]> {
        let mut buffer: Vec<u8> = Vec::new();
        let materials = self.materials.borrow();
        if !materials.is_empty() {
            for material in materials.iter() {
                let material_bytes = material.to_bytes();
                buffer.extend_from_slice(&material_bytes);
            }
        } else {
            let empty: [u8; 48] = [0; 48];
            buffer.extend_from_slice(&empty);
        }
        buffer.into_boxed_slice()
    }

    fn add_material(&'a self, material: Rc<PBRMaterial>) {
        let ptr = Rc::as_ptr(&material) as u64;
        if self.material2index.borrow().contains_key(&ptr) {
            return;
        }
        let idx = self.materials.borrow().len() as i32;
        material.set_index(idx);
        self.material2index.borrow_mut().insert(ptr, idx);
        self.materials.borrow_mut().push(Rc::clone(&material));
    }
}

impl<'a> Default for Scene<'a> {
    fn default() -> Self {
        Scene::new(
            400,
            400,
            Camera {
                screen_size: (480.0, 320.0).into(),
                eye: (0.0, 1.0, -6.0).into(),
                target: (0.0, 0.0, 0.0).into(),
                up: cgmath::Vector3::unit_y(),
                aspect: 480.0 / 320.0,
                fovy: 60.0,
                znear: 0.1,
                zfar: 100.0,
            },
            Vector3f::zero(),
            DirectionalLight {
                direction: Vector3f::scalar(1.0),
                color: Vector3f::scalar(1.0),
            },
        )
    }
}
