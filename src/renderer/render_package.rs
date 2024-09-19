use crate::node::view_port::ViewPort;

use super::mesh::Mesh;

pub struct RenderPackage<'ctx, 'vp> {
    pub packets: Vec<RenderPacket<'ctx, 'vp>>,
}

pub struct RenderPacket<'ctx, 'vp> {
    pub view_port: &'vp ViewPort,
    pub meshes: Vec<Mesh<'ctx>>,
}

impl<'ctx, 'vp> RenderPacket<'ctx, 'vp> {
    pub fn new(view_port: &'vp ViewPort) -> Self {
        Self {
            view_port,
            meshes: Vec::new(),
        }
    }
}
