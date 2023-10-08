use std::{sync::Arc, rc::Rc};
use tobj;

use crate::{
    bvh::{bvh::BVH, bounds::Bounds3}, material::material::Material, math::vector::Vector3f, mesh::triangle::Triangle, domain::domain::Intersection,
};

use super::object::Object;

pub struct Model {
    pub triangles: Vec<Rc<Triangle>>,
    pub material: Arc<dyn Material>,
    pub bvh: Option<BVH>,
    pub area: f32,
    pub bounds: Bounds3
}

impl Model {
    pub fn new(path: &str, material: Arc<dyn Material>) -> Model {
        let mut model = Model {
            triangles: vec![],
            material: Arc::clone(&material),
            bvh: None,
            area: 0.0,
            bounds: Bounds3::zero()
        };
        model.load(path);
        return model;
    }

    fn load(&mut self, path: &str) {
        let obj = tobj::load_obj(path, &tobj::GPU_LOAD_OPTIONS);
        let (models, _) = obj.expect(&format!("Failed to load OBJ file {}", path));
        if models.len() != 1 {
            panic!("Invalid OBJ format: only single mesh models are supported");
        }
        let mut p_min = Vector3f::new(f32::MAX, f32::MAX, f32::MAX);
        let mut p_max = Vector3f::new(f32::MIN, f32::MIN, f32::MIN);
        let mesh = &models[0].mesh;
        let mut vertices: Vec<Vector3f> = vec![];
        let positions = &mesh.positions;
        for i in (0..positions.len()).step_by(3) {
            let vertex = Vector3f::new(positions[i], 
                                                 positions[i + 1], 
                                                 positions[i + 2]);

            p_min.x = f32::min(p_min.x, vertex.x);
            p_min.y = f32::min(p_min.y, vertex.y);
            p_min.z = f32::min(p_min.z, vertex.z);
            p_max.x = f32::max(p_max.x, vertex.x);
            p_max.y = f32::max(p_max.y, vertex.y);
            p_max.z = f32::max(p_max.z, vertex.z);

            vertices.push(vertex);
        }

        let indicies = &mesh.indices;
        for i in (0..indicies.len()).step_by(3) {
            let v0 = vertices[indicies[i] as usize].clone();
            let v1 = vertices[indicies[i + 1] as usize].clone();
            let v2 = vertices[indicies[i + 2] as usize].clone();
            self.triangles.push(
                Triangle::new(&v0, &v1, &v2, Arc::clone(&self.material))
            );
        }

        self.bounds = Bounds3 { p_min, p_max };

        let mut area: f32 = 0.0;
        let primitives = self.triangles.iter()
            .map(|triangle| {
                let obj: Rc<dyn Object> = Rc::clone(triangle) as _;
                area += obj.get_area();
                obj
            })
            .collect();
        let mut bvh = BVH::new(primitives);
        bvh.build();
        self.bvh = Some(bvh);
    }
}

impl Object for Model {
    fn get_area(&self) -> f32 {
        return self.area
    }

    fn get_bounds(&self) -> Bounds3 {
        return self.bounds.clone();
    }

    fn intersect(&self, ray: &crate::domain::domain::Ray) -> crate::domain::domain::Intersection {
        if let Some(bvh) = self.bvh.as_ref() {
            return bvh.intersect(ray);
        }
        return Intersection::new();
    }

    fn sample(&self) -> (Intersection, f32) {
        if self.bvh.is_none() {
            return (Intersection::new(), 0.0)
        }

        let (mut inter, area) = self.bvh.as_ref().unwrap().sample();
        inter.emit = self.material.get_emission();
        return (inter, area);

    }
}