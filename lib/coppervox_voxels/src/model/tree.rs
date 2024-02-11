use super::VoxModel;
use crate::Shared;
use nalgebra::Matrix4;

/// A node in a tree structure of voxel models
pub struct VoxNode<C, const D: usize> {
    /// The transformation matrix to apply to  
    /// this voxel model and all of its children.
    transform: Matrix4<isize>,

    /// The model assigned to this node.
    /// Multiple nodes can share the same voxel model.
    model: Shared<VoxModel<C, D>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use slab_tree::*;

    #[test]
    fn it_works() {
        let root_node = VoxNode::<i32, 12> {
            transform: Matrix4::identity(),

            model: Default::default(),
        };

        let mut tree = TreeBuilder::new().with_root(root_node).build();
        let root_id = tree.root_id().unwrap();
        let _root = tree.get_mut(root_id).unwrap();

        //root.append(data);
    }
}
