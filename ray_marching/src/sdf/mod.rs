use crate::math::Vector3f;
use core::fmt;
use elsa::FrozenVec;
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

pub trait Shape: Send + Sync + Display {
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
                "HitResult {{ 
                    distance: {}, shape: None }}",
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
}

impl<'a> Scene<'a> {
    pub fn new() -> Scene<'a> {
        Scene {
            nodes: FrozenVec::new(),
            root_nodes: FrozenVec::new(),
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
}

impl<'a> Default for Scene<'a> {
    fn default() -> Self {
        Scene::new()
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
