use std::sync::{Arc, Weak};

use crate::{material::material::Material, bvh::bounds::Bounds3, domain::domain::{Ray, Intersection}, math::{vector::Vector3f, Math}};
use super::object::Object;

pub struct Triangle {
    pub v0: Vector3f,
    pub v1: Vector3f,
    pub v2: Vector3f,
    pub e1: Vector3f,
    pub e2: Vector3f,
    pub normal: Vector3f,
    pub area: f32,
    pub material: Arc<dyn Material>,
    // weak_self: Weak<Triangle>
}

impl Triangle {
    pub fn new(v0: &Vector3f, v1: &Vector3f, v2: &Vector3f, material: Arc<dyn Material>) -> Triangle {
        let e1 = v1 - v0;
        let e2 = v2 - v0; 
        let mut s = Triangle { 
            v0: v0.clone(),
            v1: v1.clone(),
            v2: v2.clone(),
            normal: e1.cross(&e2).normalize(), 
            area: e1.cross(&e2).length() * 0.5, 
            // weak_self: Weak::new(),
            material,
            e1, e2,
        };

        // let weak_s = Arc::downgrade(&s);
        // // Arc::make_mut(&mut s).weak_self = weak_s;
        // let _ = std::mem::replace(&mut Arc::get_mut(&mut s).unwrap().weak_self, weak_s);
        // println!("area of the triangle {}", s.weak_self.upgrade().as_ref().unwrap().area);
        // s   
        s
    }

    pub fn clone(&self) -> Triangle {
        Triangle { 
            v0: self.v0.clone(), 
            v1: self.v1.clone(),
            v2: self.v2.clone(), 
            e1: self.e1.clone(),
            e2: self.e2.clone(), 
            normal: self.normal.clone(), 
            area: self.area, 
            material: Arc::clone(&self.material),
            // weak_self: Weak::clone(&self.weak_self)
        }
    }
}

impl Object for Triangle {
    fn get_bounds(&self) -> Bounds3 {
        let mut b = Bounds3::from_points(&self.v0, &self.v1);
        b.union_point(&self.v2);
        return b;
    }

    fn get_area(&self) -> f32 {
        return self.area;
    }

    fn intersect(&self, ray: &Ray) -> Intersection {
        // backface culling
        if ray.direction.dot(&self.normal) > 0.0 {
            return Intersection::new();
        }

        let pvec = ray.direction.cross(&self.e2);
        let det = self.e1.dot(&pvec);
        if f32::abs(det) < f32::EPSILON {
            return Intersection::new();
        }

        let det_inv = 1.0 / det;
        let tvec = &ray.origin - &self.v0;
        let u = tvec.dot(&pvec) * det_inv;
        if u < 0.0 || u > 1.0 {
            return Intersection::new();
        }
        
        let qvec = tvec.cross(&self.e1);
        let v = ray.direction.dot(&qvec) * det_inv;
        if v < 0.0 || u + v > 1.0 {
            return Intersection::new();
        }

        let t = self.e2.dot(&qvec) * det_inv;
        if t > 0.0 {
            let mut inter = Intersection::new();
            inter.hit = true;
            inter.coords = &ray.origin + &(&ray.direction * t);
            inter.normal = self.normal.clone();
            inter.distance = t;
            inter.material = Some(Arc::clone(&self.material));
            // inter.obj = 
            inter
        } else {
            Intersection::new()
        }
    }

    fn sample(&self) -> (Intersection, f32) {
        let x = f32::sqrt(Math::sample_uniform_distribution(0.0, 1.0));
        let y = Math::sample_uniform_distribution(0.0, 1.0);
        let mut inter = Intersection::new();
        inter.coords = &self.v0 * (1.0 - x) 
                               + &self.v1 * (x * (1.0 - y))
                               + &self.v2 * (x * y);
        inter.normal = self.normal.clone();
        (inter, 1.0 / self.area)
    }
}