use crate::{container::FreeList, math::transform::Transform};

pub struct SceneGraph {
    nodes: FreeList<SceneGraphNode>,
}

#[derive(Default)]
struct SceneGraphNode {
    transform: Transform,
    first_child: Option<usize>,
    next_sibling: Option<usize>,
}

impl SceneGraph {
    pub fn new() -> Self {
        Self::default()
    }
}

impl SceneGraph {
    pub fn add(&mut self, parent: usize) -> Option<usize> {
        assert!(self.nodes.is_occupied(parent));

        let idx = self.nodes.push_first(SceneGraphNode::default());

        // Find spot for child.
        if self.nodes.at(parent)?.first_child.is_some() {
            let last_child = self.child_iter(parent).last()?;

            self.nodes.at_mut(last_child)?.next_sibling = Some(idx);
        } else {
            self.nodes.at_mut(parent)?.first_child = Some(idx);
        }
        
        Some(idx)
    }

    pub fn get_transform(&self, id: usize) -> Option<&Transform> {
        Some(&self.nodes.at(id)?.transform)
    }

    pub fn get_mut_transform(&mut self, id: usize) -> Option<&mut Transform> {
        Some(&mut self.nodes.at_mut(id)?.transform)
    }

    pub fn child_iter(&self, parent: usize) -> SceneGraphChildIterator {
        SceneGraphChildIterator::new(self, parent)
    }
}

impl Default for SceneGraph {
    fn default() -> Self {
        let mut nodes = FreeList::new();
        
        nodes.push_first(SceneGraphNode::default());

        Self {
            nodes,
        }
    }
}

pub struct SceneGraphChildIterator<'a> {
    current_child: Option<usize>,
    scene_graph: &'a SceneGraph,
}
impl<'a> SceneGraphChildIterator<'a> {
    fn new(scene_graph: &'a SceneGraph, parent: usize) -> Self {
        let current_child = scene_graph.nodes.at(parent).unwrap().first_child;

        Self {
            current_child,
            scene_graph,
        }
    }
}

impl<'a> Iterator for SceneGraphChildIterator<'a> {
    type Item = usize;
    
    fn next(&mut self) -> Option<Self::Item> {
        let child = self.current_child;

        if let Some(current_child) = self.current_child {
            self.current_child = self.scene_graph.nodes.at(current_child)?.next_sibling;
        }

        child
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn scene_graph_node_adding_and_child_iterator() {
        let mut scene_graph = SceneGraph::new();
        let node_1 = scene_graph.add(0).unwrap();
        let node_2 = scene_graph.add(0).unwrap();
        let node_1_1 = scene_graph.add(node_1).unwrap();

        let mut root_iter = scene_graph.child_iter(0);
        assert_eq!(root_iter.next(), Some(node_1));
        assert_eq!(root_iter.next(), Some(node_2));
        assert_eq!(root_iter.next(), None);

        let mut node_1_iter = scene_graph.child_iter(node_1);
        assert_eq!(node_1_iter.next(), Some(node_1_1));
        assert_eq!(node_1_iter.next(), None);
    }
}
