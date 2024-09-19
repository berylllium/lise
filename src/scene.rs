use crate::{container::FreeList, math::transform::Transform};

pub struct Scene {
    nodes: FreeList<SceneNode>,
}

pub struct SceneNode {
    pub transform: Transform,

    pub first_child: Option<usize>,
    pub next_sibling: Option<usize>,

    pub id: usize,

    pub 
}
