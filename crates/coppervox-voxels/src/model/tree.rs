use super::VoxModel;
use crate::{Cell, Shared, model::transform::Matrix};
use gridmap::transform::Transform;
use num::traits::ConstZero;
use slab_tree::{NodeRef, Tree};

/// A node in a tree structure of voxel models
pub struct VoxNode<C> {
    /// The relative priority of the node over other nodes.
    /// Specify if the voxels of this node should overwrite
    /// or being overwritten by the voxels of other nodes.
    relative_priority: isize,

    /// The relative transformation matrix to apply to
    /// this voxel model and all of its children.
    relative_transform: Matrix,

    /// The model assigned to this node.
    /// Multiple nodes can share the same voxel model.
    model: Shared<VoxModel<C>>,
}

/// Combine all the models in the tree into a single model made of voxels
pub fn merge_tree<C>(tree: &Tree<VoxNode<C>>) -> VoxModel<(C, isize)>
where
    C: Cell + Default + Copy + ConstZero,
    (C, isize): ConstZero,
{
    // the canvas where to draw all of the models contained in the tree
    let mut canvas = VoxModel::<(C, isize)>::default();

    // Start at the root and iterate over all of
    // its children and write them to the canvas.
    if let Some(root) = tree.root() {
        let identity = Matrix::default();
        write_node_to_canvas(0, &identity, root, &mut canvas);
    }

    canvas
}

/// Apply the matrix stored in this node and its children to the canvas provided.
/// cumulated priority and transform are the result of recursive calls to the
/// `apply_node` function on nodes and their children.
fn write_node_to_canvas<C>(
    cumulated_priority: isize,
    cumulated_transform: &Matrix,
    node: NodeRef<'_, VoxNode<C>>,
    canvas: &mut VoxModel<C>,
) where
    C: Cell + Default + Copy + ConstZero,
{
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
fn write_model_to_canvas<C>(
    priority: isize,
    transform: &Matrix,
    model: &Shared<VoxModel<C>>,
    canvas: &mut VoxModel<(C, isize)>,
) where
    C: Cell + Default + Copy + ConstZero,
    (C, isize): Cell + ConstZero,
{
    // [!] For this instruction to compile properly,
    // the `Borrow` type must be excluded from this file.
    for (index, &cell) in model.borrow().0.indexed_iter() {
        // transform the index to find where to write the cell in the canvas
        let mut c_index = index;
        transform.apply(&mut c_index);

        // Access the current value of the canvas.
        // Based on the priority stored, figure out
        // if the value should be overwritten or not.
        let (c_cell, c_priority) = &mut canvas.0[c_index];
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
