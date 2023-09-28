use std::sync::Arc;

use crate::domain::domain::Axis;
use crate::mesh::object::Object;
use crate::bvh::bounds::Bounds3;

pub struct BVH {
    pub primitives: Vec<Arc<Object>>,
    root: Option<Box<BVHNode>>
}

impl BVH {
    pub fn new(primitives: Vec<Arc<Object>>) -> Arc<BVH> {
        Arc::new(BVH {
            root: None,
            primitives
        })
    }

    pub fn build(&mut self) {
        self.root = Some(self.build_recursively())
    }

    fn build_recursively(&mut self) -> Box<BVHNode> {
        let root = BVHNode::new();
        return root;
    }
}

pub struct BVHNode {
    pub bounds: Bounds3,
    pub left: Option<Box<BVHNode>>,
    pub right: Option<Box<BVHNode>>,
    pub object: Option<Box<Object>>,
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