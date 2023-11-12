use crate::{domain::Ray, math::Vector3f};
use core::fmt;
use elsa::FrozenVec;
use std::any::Any;
use std::fmt::Display;

pub enum ShapeType {
    Sphere,
    Cube,
}

impl Display for ShapeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShapeType::Sphere => write!(f, "Sphere"),
            ShapeType::Cube => write!(f, "Cube"),
        }
    }
}

pub trait Shape: Send + Sync + Display + Any {
    fn shape_type(&self) -> ShapeType;
    fn sdf(&self, p: &Vector3f) -> f64;
}

pub struct Sphere {
    pub center: Vector3f,
    pub radius: f64,
}

impl Shape for Sphere {
    fn shape_type(&self) -> ShapeType {
        ShapeType::Sphere
    }

    fn sdf(&self, p: &Vector3f) -> f64 {
        (&self.center - p).length() - self.radius
    }
}

impl Display for Sphere {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Sphere(c={}, o={})", self.center, self.radius)
    }
}

pub struct Cube {
    most_front_up_right: Vector3f,
    center: Vector3f,
}

impl Shape for Cube {
    fn shape_type(&self) -> ShapeType {
        ShapeType::Cube
    }

    fn sdf(&self, p: &Vector3f) -> f64 {
        let mut d_abs = p - &self.center;
        d_abs.x = f64::abs(d_abs.x);
        d_abs.y = f64::abs(d_abs.y);
        d_abs.z = f64::abs(d_abs.z);

        let d = &d_abs - &self.most_front_up_right;
        let mut d_clamped = d.clone();
        d_clamped.x = f64::max(d.x, 0.0);
        d_clamped.y = f64::max(d.y, 0.0);
        d_clamped.z = f64::max(d.z, 0.0);
        d_clamped.length() + f64::min(f64::max(f64::max(d.x, d.y), d.z), 0.0)
    }
}

impl Display for Cube {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Cube(c={}, mfur={})",
            self.center, self.most_front_up_right
        )
    }
}

pub enum ShapeOpType {
    Nop,
    Union,
    Subtraction,
    Intersection,
    // SmoothUnion
}

impl Display for ShapeOpType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShapeOpType::Union => write!(f, "Union"),
            ShapeOpType::Subtraction => write!(f, "Subtraction"),
            ShapeOpType::Intersection => write!(f, "Intersection"),
            ShapeOpType::Nop => write!(f, "Nop"),
        }
    }
}

pub struct ShapeOp<'a> {
    pub shape: Box<dyn Shape>,
    pub op: ShapeOpType,
    pub next: Option<&'a ShapeOp<'a>>,
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
            background_color: background_color,
            width,
            height,
            fov,
            sample_per_pixel,
        }
    }

    pub fn add_leaf_node(&'a self, shape: Box<dyn Shape>) -> &'a ShapeOp<'a> {
        let idx = self.nodes.len();
        self.nodes.push(Box::new(ShapeOp {
            shape,
            op: ShapeOpType::Nop,
            next: None,
        }));
        &self.nodes[idx]
    }

    pub fn add_node(
        &'a self,
        shape: Box<dyn Shape>,
        op: ShapeOpType,
        next: Option<&'a ShapeOp<'a>>,
    ) -> &'a ShapeOp<'a> {
        let idx = self.nodes.len();
        self.nodes.push(Box::new(ShapeOp { shape, op, next }));
        &self.nodes[idx]
    }

    pub fn add_root_node(&'a self, node: &'a ShapeOp<'a>) {
        self.root_nodes.push(node);
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

    pub fn cast_ray(&'a self, origin_ray: &Ray) -> Vector3f {
        let n_reflection = 1;
        let mut color = Vector3f::zero();
        let mut ray = Ray::new(&origin_ray.origin, &origin_ray.direction, 0.0);
        for _ in 0..n_reflection {
            let hit = self.ray_march(&ray);
            if hit.shape_op.is_some() {
                let p = ray.eval(hit.distance);
                let normal = self.normal(&hit, &p);

                // FIXME: naive blinn-phong
                let ambient = Vector3f::new(0.2, 0.2, 0.2);
                let light_color = Vector3f::new(0.8, 0.0, 0.0);
                let light_dir = Vector3f::new(1.0, 1.0, 1.0).normalize();
                let view = &ray.origin - &p;
                let light = -&light_dir;
                let half: Vector3f = ((view + light) / 2.0).normalize();
                let factor = f64::clamp(half.dot(&normal), 0.0, 1.0);
                color += light_color * factor + ambient;

                // FIXME: reflection direction
                ray.origin = p + normal.clone() * 0.05;
                ray.direction = -&normal;
            } else {
                // not hit anything, use background color
                // FIXME: operator overload
                color += self.background_color.clone();
            }
        }

        // HDR
        // color.x = color.x / (color.x + 1.0);
        // color.y = color.y / (color.y + 1.0);
        // color.z = color.z / (color.z + 1.0);

        // Gamma
        return color;
    }

    pub fn ray_march(&'a self, ray: &Ray) -> HitResult<'a> {
        let max_steps = 300;
        let max_dist = 1e5;
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
        let mut next = self.next;
        while let Some(op) = next {
            let sdf_i = op.shape.sdf(p);
            sdf_f = Self::op_sdf(sdf_f, &self.op, sdf_i);
            next = op.next;
        }
        sdf_f
    }

    fn op_sdf(sdf_a: f64, op: &ShapeOpType, sdf_b: f64) -> f64 {
        match op {
            ShapeOpType::Union => f64::min(sdf_a, sdf_b),
            ShapeOpType::Subtraction => f64::max(sdf_a, -sdf_b),
            ShapeOpType::Intersection => f64::max(sdf_a, sdf_b),
            ShapeOpType::Nop => panic!("invalid operation"),
        }
    }
}
