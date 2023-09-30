use std::cmp::Ordering;
use std::sync::Arc;

use crate::domain::domain::{Axis, Intersection, Ray};
use crate::mesh::object::Object;
use crate::bvh::bounds::Bounds3;

pub struct BVH {
    pub primitives: Vec<Arc<dyn Object>>,
    root: Option<Box<BVHNode>>
}

impl BVH {
    pub fn new(primitives: Vec<Arc<dyn Object>>) -> Arc<BVH> {
        Arc::new(BVH {
            root: None,
            primitives
        })
    }

    pub fn build(&mut self) {
        let tmp = self.primitives.clone();
        self.root = Some(self.build_recursively(tmp))
    }

    pub fn intersect(&self, ray: &Ray) -> Intersection {
        if self.root.is_none() {
            return Intersection::new();
        }
        return BVH::intersect_internal(self.root.as_ref(), ray);
    }

    fn build_recursively(&self, mut primitives: Vec<Arc<dyn Object>>) -> Box<BVHNode> {
        let mut root = BVHNode::new();
        let mut bounds = Bounds3::zero();
        for object in primitives.iter() {
            bounds.union(&object.get_bounds());
        }
        
        let n_objs = primitives.len();
        if n_objs == 1 {
            let obj = &primitives[0];
            root.bounds = obj.get_bounds();
            root.object = Some(Arc::clone(obj));
            root.left = None;
            root.right = None;
            root.area = obj.get_area();
        } else if n_objs == 2 {
            let left = vec![Arc::clone(&primitives[0])];
            root.left = Some(self.build_recursively(left));

            let right = vec![Arc::clone(&primitives[1])];
            root.right = Some(self.build_recursively(right));

            root.bounds = Bounds3::union2(
                &root.left.as_ref().unwrap().bounds,
                &root.right.as_ref().unwrap().bounds);
            root.area = root.left.as_ref().unwrap().area
                      + root.right.as_ref().unwrap().area;
        } else {
            let mut max_bounds = Bounds3::zero();
            for primitive in primitives.iter() {
                max_bounds.union(&primitive.get_bounds());
            }
            let max_axis = max_bounds.max_extent_axis();
            match max_axis {
                Axis::X => {
                    primitives.sort_by(|a, b| {
                        let o1 = a.get_bounds().center().x;
                        let o2 = b.get_bounds().center().x;
                        if o1 < o2 {
                            return Ordering::Less;
                        } else if o1 == o2 {
                            return Ordering::Equal;
                        }
                        return Ordering::Greater;
                    })
                }
                Axis::Y => {
                    primitives.sort_by(|a, b| {
                        let o1 = a.get_bounds().center().y;
                        let o2 = b.get_bounds().center().y;
                        if o1 < o2 {
                            return Ordering::Less;
                        } else if o1 == o2 {
                            return Ordering::Equal;
                        }
                        return Ordering::Greater;
                    })
                }
                Axis::Z => {
                    primitives.sort_by(|a, b| {
                        let o1 = a.get_bounds().center().z;
                        let o2 = b.get_bounds().center().z;
                        if o1 < o2 {
                            return Ordering::Less;
                        } else if o1 == o2 {
                            return Ordering::Equal;
                        }
                        return Ordering::Greater;
                    })
                }
                Axis::Nil => {
                    panic!("invalid axis type");
                }
            }
            let middle_index = primitives.len() / 2;
            let left = primitives[0..middle_index].to_vec();
            let right = primitives[middle_index..].to_vec();
            root.left = Some(self.build_recursively(left));
            root.right = Some(self.build_recursively(right));
            root.bounds = Bounds3::union2(&root.left.as_ref().unwrap().bounds, 
                                          &root.right.as_ref().unwrap().bounds);
            root.area = root.left.as_ref().unwrap().area +
                        root.right.as_ref().unwrap().area;
        }
        return root;
    }

    fn intersect_internal(root: Option<&Box<BVHNode>>, ray: &Ray) -> Intersection {
        if root.is_none() {
            return Intersection::new();
        }

        let node = root.unwrap();
        if !node.bounds.intersect(ray) {
            return Intersection::new();
        }

        // leaf node
        if node.left.is_none() && node.right.is_none() {
            return node.object.as_ref().unwrap().intersect(ray);
        }

        let left = BVH::intersect_internal(node.left.as_ref(), ray);
        let right = BVH::intersect_internal(node.right.as_ref(), ray);
        if left.distance < right.distance {
            left
        } else {
            right
        }
    }
}

pub struct BVHNode {
    pub bounds: Bounds3,
    pub left: Option<Box<BVHNode>>,
    pub right: Option<Box<BVHNode>>,
    pub object: Option<Arc<dyn Object>>,
    pub area: f32,
    pub split_axis: Axis,
    pub first_primitive_offset: i32,
    pub n_primitives: i32
}

impl BVHNode {
    pub fn new() -> Box<BVHNode> {
        Box::new(BVHNode { 
            bounds: Bounds3::zero(), 
            left: None, 
            right: None, 
            object: None, 
            area: 0.0, 
            split_axis: Axis::Nil, 
            first_primitive_offset: 0, 
            n_primitives: 0 
        })
    }
}