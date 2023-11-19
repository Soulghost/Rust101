use crate::material::PBRMaterial;
use crate::math::Vector2f;
use crate::{domain::Ray, math::Vector3f};
use core::fmt;
use elsa::FrozenVec;
use std::any::Any;
use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;

pub enum ShapeType {
    Sphere,
    Cube,
    CubeFrame,
    Torus,
}

impl Display for ShapeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShapeType::Sphere => write!(f, "Sphere"),
            ShapeType::Cube => write!(f, "Cube"),
            ShapeType::CubeFrame => write!(f, "CubeFrame"),
            ShapeType::Torus => write!(f, "Torus"),
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
    pub most_front_up_right: Vector3f,
    pub center: Vector3f,
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

        let d = d_abs - self.most_front_up_right;
        let mut d_clamped = d;
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

pub struct CubeFrame {
    pub center: Vector3f,
    pub bounds: Vector3f,
    pub thinkness: f64,
}

impl Shape for CubeFrame {
    fn shape_type(&self) -> ShapeType {
        ShapeType::CubeFrame
    }

    // float sdBoxFrame( vec3 p, vec3 b, float e )
    // {
    //        p = abs(p  )-b;
    //   vec3 q = abs(p+e)-e;

    //   return min(
    //    min(length(max(vec3(p.x,q.y,q.z),0.0))+min(max(p.x,max(q.y,q.z)),0.0),
    //        length(max(vec3(q.x,p.y,q.z),0.0))+min(max(q.x,max(p.y,q.z)),0.0)
    //    ),
    //    length(max(vec3(q.x,q.y,p.z),0.0))+min(max(q.x,max(q.y,p.z)),0.0));
    // }

    fn sdf(&self, p: &Vector3f) -> f64 {
        let mut p = p - &self.center;
        p.x = f64::abs(p.x) - self.bounds.x;
        p.y = f64::abs(p.y) - self.bounds.y;
        p.z = f64::abs(p.z) - self.bounds.z;

        let mut q = p;
        q.x = f64::abs(q.x + self.thinkness) - self.thinkness;
        q.y = f64::abs(q.y + self.thinkness) - self.thinkness;
        q.z = f64::abs(q.z + self.thinkness) - self.thinkness;

        min(
            min(
                Vector3f::max_scalar(&Vector3f::new(p.x, q.y, q.z), 0.0).length()
                    + min(max(p.x, max(q.y, q.z)), 0.0),
                Vector3f::max_scalar(&Vector3f::new(q.x, p.y, q.z), 0.0).length()
                    + min(max(q.x, max(p.y, q.z)), 0.0),
            ),
            Vector3f::max_scalar(&Vector3f::new(q.x, q.y, p.z), 0.0).length()
                + min(max(q.x, max(q.y, q.z)), 0.0),
        )
    }
}

impl Display for CubeFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Cube(center={}, bounds={}, thinkness={})",
            self.center, self.bounds, self.thinkness
        )
    }
}

pub struct Torus {
    pub center: Vector3f,
    pub outer_radius: f64,
    pub inner_radius: f64,
}

impl Shape for Torus {
    fn shape_type(&self) -> ShapeType {
        ShapeType::Torus
    }

    fn sdf(&self, p: &Vector3f) -> f64 {
        Vector2f::new(
            Vector2f::new(p.x - self.center.x, p.z - self.center.z).length() - self.outer_radius,
            p.y - self.center.y,
        )
        .length()
            - self.inner_radius
    }
}

impl Display for Torus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Torus(center={}, r0={}, r1={})",
            self.center, self.outer_radius, self.inner_radius
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
    pub material: Rc<PBRMaterial>,
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

    pub fn cast_ray(&'a self, origin_ray: &Ray) -> Vector3f {
        let mut color = self._cast_ray(origin_ray, 0, None);

        // HDR
        color.x = color.x / (color.x + 1.0);
        color.y = color.y / (color.y + 1.0);
        color.z = color.z / (color.z + 1.0);

        color
    }

    fn _cast_ray(&'a self, ray: &Ray, depth: u32, source_op: Option<&'a ShapeOp<'a>>) -> Vector3f {
        if depth > 1 {
            return Vector3f::zero();
        }

        // let mut ray = Ray::new(&origin_ray.origin, &origin_ray.direction, 0.0);
        // let mut view_material: Option<Rc<PBRMaterial>> = None;
        let hit = self.ray_march(ray, 1e5);
        let ambient_intensity = 0.15;
        let light_intensity = 2.0;
        if let Some(op) = hit.shape_op {
            if let Some(orig_op) = source_op {
                if std::ptr::eq(op, orig_op) {
                    return Vector3f::zero();
                }
            }
            let p = ray.eval(hit.distance);
            let normal = self.normal(&hit, &p);
            let material = Rc::clone(&op.material);

            // FIXME: naive blinn-phong
            let ambient = Vector3f::new(1.0, 1.0, 1.0) * ambient_intensity;
            let light_color = Vector3f::new(1.0, 1.0, 1.0) * light_intensity;
            let light_dir = Vector3f::new(0.32, -0.77, 0.56);
            let view = (ray.origin - p).normalize();
            let light = -&light_dir;
            let half: Vector3f = ((view + light) / 2.0).normalize();
            // return Vector3f::new(light.dot(&normal), light.dot(&normal), light.dot(&normal));

            let albedo = if !self.is_ground(op) {
                material.kd
            } else {
                // ground color
                if ((p.x * 0.5 + self.width as f64) as u32 + (p.z * 0.5 + 1000.0) as u32) & 1 != 0 {
                    Vector3f::new(1.0, 1.0, 1.0) * 0.8
                } else {
                    Vector3f::new(1.0, 1.0, 1.0) * 0.3
                }
            };

            // shadow
            let shadow_check_dis = 1e4;
            let shadow_orig = p + normal * 1e-3;
            let shadow_dir = light;
            let shadow_ray = Ray::new(&shadow_orig, &shadow_dir, 0.0);
            let shadow_hit = self.ray_march(&shadow_ray, shadow_check_dis);
            let shadow_attenuation = if shadow_hit.shape_op.is_none() {
                1.0
            } else {
                0.0
            };

            // diffuse
            let diffuse_factor =
                f64::max(light.dot(&normal), 0.0) * material.roughness * shadow_attenuation;
            let diffuse = &light_color * &albedo * diffuse_factor;

            // specular
            let spec_factor = f64::powf(f64::max(half.dot(&normal), 0.0), 16.0)
                * material.metalness
                * shadow_attenuation;
            let specular = light_color * spec_factor;

            // FIXME: reflection direction
            // view + reflection = 2 * normal;
            let reflection_dir = normal * 2 * normal.dot(&view) - view;
            let reflection_orig = if normal.dot(&reflection_dir) >= 0.0 {
                p + normal * 1e-3
            } else {
                p - normal * 1e-3
            };
            let reflection_ray = Ray::new(&reflection_orig, &reflection_dir, 0.0);
            let reflection_factor = reflection_dir.dot(&normal) * material.metalness;
            let reflection =
                self._cast_ray(&reflection_ray, depth + 1, hit.shape_op) * reflection_factor;
            return ambient + diffuse + specular + material.emission + reflection;
        } else if depth > 0 {
            return Vector3f::zero();
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

fn min(a: f64, b: f64) -> f64 {
    f64::min(a, b)
}

fn max(a: f64, b: f64) -> f64 {
    f64::max(a, b)
}
