use std::cmp::Ordering;
use std::sync::Arc;

use crate::domain::domain::{Axis, Intersection, Ray};
use crate::math::Math;
use crate::mesh::object::Object;
use crate::bvh::bounds::Bounds3;

pub struct BVH<'a> {
    pub primitives: Vec<&'a dyn Object>,
    root: Option<Box<BVHNode<'a>>>
}

impl<'a> BVH<'a> {
    pub fn new(primitives: Vec<&'a dyn Object>) -> BVH {
        BVH {
            root: None,
            primitives
        }
    }

    pub fn build(&mut self) {
        let mut tmp = self.primitives.clone();
        let node = self.build_recursively(&mut tmp);
        self.root = Some(node);
    }

    pub fn intersect(&self, ray: &Ray) -> Intersection {
        if self.root.is_none() {
            return Intersection::new();
        }
        return BVH::intersect_internal(self.root.as_ref(), ray);
    }

    pub fn sample(&self) -> (Intersection, f32) {
        let root_node = self.root.as_ref().unwrap();
        let p = f32::sqrt(Math::sample_uniform_distribution(0.0, 1.0)) * root_node.area;
        let (inter, mut pdf) = self.get_sample(root_node, p);
        pdf /= root_node.area;
        return (inter, pdf);
    }

    fn build_recursively(&self, primitives: &mut Vec<&'a dyn Object>) -> Box<BVHNode<'a>> {
        let mut root = BVHNode::new();
        let mut bounds = Bounds3::zero();
        for object in primitives.iter() {
            bounds.union(&object.get_bounds());
        }
        
        let n_objs = primitives.len();
        if n_objs == 1 {
            let obj = primitives[0];
            root.bounds = obj.get_bounds();
            root.object = Some(obj);
            root.left = None;
            root.right = None;
            root.area = obj.get_area();
        } else if n_objs == 2 {
            let mut left = vec![primitives[0]];
            root.left = Some(self.build_recursively(&mut left));

            let mut right = vec![primitives[1]];
            root.right = Some(self.build_recursively(&mut right));

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
            let mut left = primitives[0..middle_index].to_vec();
            let mut right = primitives[middle_index..].to_vec();
            root.left = Some(self.build_recursively(&mut left));
            root.right = Some(self.build_recursively(&mut right));
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

    fn get_sample(&self, node: &Box<BVHNode>, p: f32) -> (Intersection, f32) {
        if node.left.is_none() || node.right.is_none() {
            assert!(node.object.is_some());
            let (inter, mut pdf) = node.object.as_ref().unwrap().sample();
            pdf *= node.area;
            return (inter, pdf);
        }

        let left_node = node.left.as_ref().unwrap();
        let right_node = node.right.as_ref().unwrap();
        if p < left_node.area {
            return self.get_sample(left_node, p)
        } else {
            return self.get_sample(right_node, p - left_node.area);
        }
    }
}

pub struct BVHNode<'a> {
    pub bounds: Bounds3,
    pub left: Option<Box<BVHNode<'a>>>,
    pub right: Option<Box<BVHNode<'a>>>,
    pub object: Option<&'a dyn Object>,
    pub area: f32,
    pub split_axis: Axis,
    pub first_primitive_offset: i32,
    pub n_primitives: i32
}

impl<'a> BVHNode<'a> {
    pub fn new() -> Box<BVHNode<'a>> {
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