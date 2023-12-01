use crate::material::pbr::pbr_lighting;
use crate::material::PBRMaterial;
use crate::math::lerp;
use crate::{domain::Ray, math::Vector3f};
use cgmath::num_traits::{ToBytes, ToPrimitive};
use core::fmt;
use elsa::FrozenVec;
use std::any::Any;
use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;

pub mod ext;
pub mod primitive;

pub enum ShapeType {
    Sphere,
    Cube,
    CubeFrame,
    Torus,
    DeathStar,
    Helix,
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
    fn sdf(&self, p: &Vector3f) -> f64;
    fn rotate_ray(&self, ray: &Ray) -> Ray {
        *ray
    }
    fn to_bytes(&self) -> [u8; 40] {
        [0; 40]
    }
}

pub enum ShapeOpType {
    Nop,
    Union,
    Subtraction,
    Intersection,
    SmoothUnion,
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
    pub index: u32,
    pub shape: Box<dyn Shape>,
    pub op: ShapeOpType,
    pub material: Rc<PBRMaterial>,
    pub next: Option<&'a ShapeOp<'a>>,
}

impl<'a> ShapeOp<'a> {
    pub fn to_bytes(&self) -> [u8; 48] {
        let type_index: u32 = 0;
        let material_index: u32 = 1;
        let mut bytes = [0u8; 48];
        let type_bytes = type_index.to_le_bytes();
        let material_bytes = material_index.to_le_bytes();
        let data_bytes = self.shape.to_bytes();
        bytes[0..4].copy_from_slice(&type_bytes);
        bytes[4..8].copy_from_slice(&material_bytes);
        bytes[8..48].copy_from_slice(&data_bytes);
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

pub struct Scene<'a> {
    pub nodes: FrozenVec<Box<ShapeOp<'a>>>,
    pub root_nodes: FrozenVec<&'a ShapeOp<'a>>,
    pub ground_node: RefCell<Option<&'a ShapeOp<'a>>>,
    pub background_color: Vector3f,
    pub width: u32,
    pub height: u32,
    pub fov: f64,
    pub sample_per_pixel: u32,
}

impl<'a> Scene<'a> {
    pub fn new(
        width: u32,
        height: u32,
        fov: f64,
        sample_per_pixel: u32,
        background_color: Vector3f,
    ) -> Scene<'a> {
        Scene {
            nodes: FrozenVec::new(),
            root_nodes: FrozenVec::new(),
            ground_node: RefCell::new(None),
            background_color,
            width,
            height,
            fov,
            sample_per_pixel,
        }
    }

    pub fn add_leaf_node(
        &'a self,
        shape: Box<dyn Shape>,
        material: Rc<PBRMaterial>,
    ) -> &'a ShapeOp<'a> {
        let idx = self.nodes.len();
        self.nodes.push(Box::new(ShapeOp {
            index: idx as u32,
            shape,
            op: ShapeOpType::Nop,
            next: None,
            material,
        }));
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
            index: idx as u32,
            shape,
            material,
            op,
            next,
        }));
        &self.nodes[idx]
    }

    pub fn add_root_node(&'a self, node: &'a ShapeOp<'a>) {
        self.root_nodes.push(node);
    }

    pub fn set_ground(&'a self, node: &'a ShapeOp<'a>) {
        *self.ground_node.borrow_mut() = Some(node);
    }

    pub fn is_ground(&'a self, node: &'a ShapeOp<'a>) -> bool {
        if let Some(lhs) = *self.ground_node.borrow() {
            std::ptr::eq(lhs, node)
        } else {
            false
        }
    }

    pub fn sdf(&'a self, p: &Vector3f) -> HitResult<'a> {
        let mut result = HitResult::new();
        for node in &self.root_nodes {
            let dist = node.shape_sdf(p);
            if dist < result.distance {
                result.distance = dist;
                result.shape_op = Some(node);
            }
        }
        result
    }

    pub fn to_bytes(&'a self) -> Box<[u8]> {
        let mut buffer: Vec<u8> = Vec::new();
        if !self.root_nodes.is_empty() {
            // add root index
            let root_index = self.root_nodes[0].index;
            let root_index_bytes = root_index.to_le_bytes();

            let root_count = self.root_nodes.len();
            let root_count_bytes = root_count.to_u32().unwrap().to_le_bytes();
            buffer.extend_from_slice(&root_index_bytes);
            buffer.extend_from_slice(&root_count_bytes);

            let pad0: [u8; 8] = [0; 8];
            buffer.extend_from_slice(&pad0);
            for node in self.nodes.iter() {
                let node_bytes = node.to_bytes();
                buffer.extend_from_slice(&node_bytes);
            }
        } else {
            let empty: [u8; 48] = [0; 48];
            buffer.extend_from_slice(&empty);
        }
        buffer.into_boxed_slice()
    }

    pub fn cast_ray(&'a self, origin_ray: &Ray) -> Vector3f {
        let mut color = self._cast_ray(origin_ray, 0, None);

        // HDR
        color.x = color.x / (color.x + 1.0);
        color.y = color.y / (color.y + 1.0);
        color.z = color.z / (color.z + 1.0);

        color
    }

    fn _cast_ray(&'a self, ray: &Ray, depth: u32, _source_op: Option<&'a ShapeOp<'a>>) -> Vector3f {
        if depth > 1 {
            return Vector3f::zero();
        }

        // let mut ray = Ray::new(&origin_ray.origin, &origin_ray.direction, 0.0);
        // let mut view_material: Option<Rc<PBRMaterial>> = None;
        let hit = self.ray_march(ray, 1e5);
        let light_intensity = 10.0;
        if let Some(op) = hit.shape_op {
            // if let Some(orig_op) = _source_op {
            //     if std::ptr::eq(op, orig_op) {
            //         return Vector3f::zero();
            //     }
            // }
            let p = ray.eval(hit.distance);
            let normal = self.normal(&hit, &p);
            let material = Rc::clone(&op.material);

            // FIXME: naive blinn-phong
            let light_radiance = Vector3f::new(1.0, 1.0, 1.0) * light_intensity;
            let light_dir = Vector3f::new(0.32, -0.77, 0.56);
            let view = (ray.origin - p).normalize();
            let light = -&light_dir;

            let replace_albedo = if !self.is_ground(op) {
                None
            } else {
                // ground color
                if ((p.x * 0.5 + self.width as f64) as u32 + (p.z * 0.5 + 1000.0) as u32) & 1 != 0 {
                    Some(Vector3f::new(1.0, 1.0, 1.0) * 0.8)
                } else {
                    Some(Vector3f::new(1.0, 1.0, 1.0) * 0.3)
                }
            };

            // shadow
            let shadow_check_dis = 1e4;
            let shadow_orig = if normal.dot(&light) >= 0.0 {
                p + normal * 1e-1
            } else {
                p - normal * 1e-1
            };
            let shadow_dir = light;
            let shadow_ray = Ray::new(&shadow_orig, &shadow_dir, 0.0);
            let shadow_hit = self.ray_march(&shadow_ray, shadow_check_dis);
            let shadow_attenuation = if shadow_hit.shape_op.is_none() {
                1.0
            } else {
                0.0
            };

            // pbr direct lighting
            let direct_lighting = pbr_lighting(
                &hit,
                &view,
                &normal,
                &light,
                &light_radiance,
                replace_albedo,
            ) * shadow_attenuation;

            // indirect lighting
            let reflection_dir = normal * 2 * normal.dot(&view) - view;
            let reflection_orig = if normal.dot(&reflection_dir) >= 0.0 {
                p + normal * 1e-3
            } else {
                p - normal * 1e-3
            };
            let reflection_ray = Ray::new(&reflection_orig, &reflection_dir, 0.0);
            let reflection_factor = reflection_dir.dot(&normal) * material.metallic;
            let reflection =
                self._cast_ray(&reflection_ray, depth + 1, hit.shape_op) * reflection_factor;
            return direct_lighting + reflection;
        } else if depth > 0 {
            return self.background_color;
        }
        self.background_color
    }

    pub fn ray_march(&'a self, ray: &Ray, max_dist: f64) -> HitResult<'a> {
        let max_steps = 300;
        let mut dist = 0.0;
        let march_accuracy = 1e-3;
        for _ in 0..max_steps {
            let p = ray.eval(dist);
            let hit = self.sdf(&p);
            if hit.distance <= march_accuracy {
                // hit object
                return HitResult {
                    distance: dist,
                    shape_op: hit.shape_op,
                };
            }

            dist += hit.distance;
            if dist >= max_dist {
                break;
            }
        }
        return HitResult::new();
    }

    pub fn normal(&'a self, hit: &HitResult, p: &Vector3f) -> Vector3f {
        if hit.shape_op.is_none() {
            panic!("impossible");
        }

        let eps_grad = 1e-3;
        let p_x_p = p + &Vector3f::new(eps_grad, 0.0, 0.0);
        let p_x_m = p - &Vector3f::new(eps_grad, 0.0, 0.0);
        let p_y_p = p + &Vector3f::new(0.0, eps_grad, 0.0);
        let p_y_m = p - &Vector3f::new(0.0, eps_grad, 0.0);
        let p_z_p = p + &Vector3f::new(0.0, 0.0, eps_grad);
        let p_z_m = p - &Vector3f::new(0.0, 0.0, eps_grad);

        let shape_op = hit.shape_op.unwrap();
        let sdf_x_p = shape_op.shape_sdf(&p_x_p);
        let sdf_x_m = shape_op.shape_sdf(&p_x_m);
        let sdf_y_p = shape_op.shape_sdf(&p_y_p);
        let sdf_y_m = shape_op.shape_sdf(&p_y_m);
        let sdf_z_p = shape_op.shape_sdf(&p_z_p);
        let sdf_z_m = shape_op.shape_sdf(&p_z_m);
        Vector3f::new(sdf_x_p - sdf_x_m, sdf_y_p - sdf_y_m, sdf_z_p - sdf_z_m) / (2.0 * eps_grad)
    }
}

impl<'a> Default for Scene<'a> {
    fn default() -> Self {
        Scene::new(400, 400, 45.0, 1, Vector3f::zero())
    }
}

impl<'a> ShapeOp<'a> {
    pub fn shape_sdf(&self, p: &Vector3f) -> f64 {
        let mut sdf_f = self.shape.sdf(p);
        let mut cur = self;
        let mut next = self.next;
        while let Some(op) = next {
            let sdf_i = op.shape.sdf(p);
            sdf_f = Self::op_sdf(sdf_f, &cur.op, sdf_i);
            cur = op;
            next = op.next;
        }
        sdf_f
    }

    fn op_sdf(sdf_a: f64, op: &ShapeOpType, sdf_b: f64) -> f64 {
        match op {
            ShapeOpType::Union => f64::min(sdf_a, sdf_b),
            ShapeOpType::Subtraction => f64::max(sdf_a, -sdf_b),
            ShapeOpType::Intersection => f64::max(sdf_a, sdf_b),
            ShapeOpType::SmoothUnion => {
                let k = 1.0;
                let h = f64::clamp(0.5 + 0.5 * (sdf_b - sdf_a) / k, 0.0, 1.0);
                lerp(sdf_b, sdf_a, h) - k * h * (1.0 - h)
            }
            ShapeOpType::Nop => panic!("invalid operation {}", op),
        }
    }
}
