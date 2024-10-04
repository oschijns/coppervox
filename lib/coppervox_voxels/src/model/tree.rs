use super::{transform_index, VoxModel};
use crate::{Cell, Shared};
use nalgebra::Matrix4;
use slab_tree::{NodeRef, Tree};

/// Priority trait to define which cells should
/// overwrite and which should be overwritten.
pub trait Priority: Default + Copy + Ord {
    /// combine this priority with another the relative one
    fn combine(&self, relative_priority: Self) -> Self;
}

/// A node in a tree structure of voxel models
pub struct VoxNode<C, const D: usize, P> {
    /// The relative priority of the node over other nodes.
    /// Specify if the voxels of this node should overwrite
    /// or being overwritten by the voxels of other nodes.
    relative_priority: P,

    /// The relative transformation matrix to apply to  
    /// this voxel model and all of its children.
    relative_transform: Matrix4<isize>,

    /// The model assigned to this node.
    /// Multiple nodes can share the same voxel model.
    model: Shared<VoxModel<C, D>>,
}

/// Combine all the models in the tree into a single model made of voxels
pub fn merge_tree<C: Cell, const D: usize, P: Priority>(
    tree: &Tree<VoxNode<C, D, P>>,
) -> VoxModel<(C, P), D> {
    // the canvas where to draw all of the models contained in the tree
    let mut canvas = VoxModel::<(C, P), D>::default();

    // Start at the root and iterate over all of
    // its children and write them to the canvas.
    if let Some(root) = tree.root() {
        let identity = Matrix4::<isize>::identity();
        write_node_to_canvas(P::default(), &identity, root, &mut canvas);
    }

    canvas
}

/// Apply the matrix stored in this node and its children to the canvas provided.
/// cumulated priority and transform are the result of recursive calls to the
/// `apply_node` function on nodes and their children.
fn write_node_to_canvas<C: Cell, const D: usize, P: Priority>(
    cumulated_priority: P,
    cumulated_transform: &Matrix4<isize>,
    node: NodeRef<'_, VoxNode<C, D, P>>,
    canvas: &mut VoxModel<(C, P), D>,
) {
    // compute the absolute priority and transform for further applications
    let node_data = node.data();
    let current_priority = cumulated_priority.combine(node_data.relative_priority);
    let current_transform = cumulated_transform * node_data.relative_transform;

    // write the current node to the canvas
    write_model_to_canvas(
        current_priority,
        &current_transform,
        &node_data.model,
        canvas,
    );

    // iterate over each of the children and apply them to the canvas
    for child in node.children() {
        write_node_to_canvas(current_priority, &current_transform, child, canvas);
    }
}

/// Write a model to the canvas using the given absolute
/// priority and the given absolute transform.
fn write_model_to_canvas<C: Cell, const D: usize, P: Priority>(
    priority: P,
    transform: &Matrix4<isize>,
    model: &Shared<VoxModel<C, D>>,
    canvas: &mut VoxModel<(C, P), D>,
) {
    // [!] For this instruction to compile properly,
    // the `Borrow` type must be excluded from this file.
    for (index, &cell) in model.borrow().iter() {
        // transform the index to find where to write the cell in the canvas
        let c_index = transform_index(transform, &index);

        // Access the current value of the canvas.
        // Based on the priority stored, figure out
        // if the value should be overwritten or not.
        let (c_cell, c_priority) = &mut canvas[c_index];
        if priority >= *c_priority {
            *c_cell = cell;
            *c_priority = priority;
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
