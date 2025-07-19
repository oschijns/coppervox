use std::ops::Add;

use super::VoxModel;
use crate::{Cell, Shared, model::transform::Matrix};
use gridmap::transform::Transform;
use num::{Zero, traits::ConstZero};
use slab_tree::{NodeRef, Tree};

/// A node in a tree structure of voxel models
pub struct VoxNode<C> {
    /// The model assigned to this node.
    /// Multiple nodes can share the same voxel model.
    model: Shared<VoxModel<C>>,

    /// The relative transformation matrix to apply to
    /// this voxel model and all of its children.
    transform: Matrix,

    /// The relative depth/priority of the node over other nodes.
    /// Specify if the voxels of this node should overwrite
    /// or being overwritten by the voxels of other nodes.
    depth: Depth,
}

/// Combine all the models in the tree into a single model made of voxels
pub fn merge_tree<C>(tree: &Tree<VoxNode<C>>) -> VoxModel<Pair<C>>
where
    C: Cell + Default + Copy + ConstZero,
{
    // the canvas where to draw all of the models contained in the tree
    let mut canvas = VoxModel::<Pair<C>>::default();

    // Start at the root and iterate over all of
    // its children and write them to the canvas.
    if let Some(root) = tree.root() {
        let identity = Matrix::identity();
        write_node_to_canvas(root, identity, 0, &mut canvas);
    }

    canvas
}

/// Apply the matrix stored in this node and its children to the canvas provided.
/// cumulated priority and transform are the result of recursive calls to the
/// `apply_node` function on nodes and their children.
fn write_node_to_canvas<C>(
    node: NodeRef<'_, VoxNode<C>>,
    cumul_transform: Matrix,
    cumul_depth: Depth,
    canvas: &mut VoxModel<Pair<C>>,
) where
    C: Cell + Default + Copy + ConstZero,
{
    // compute the absolute priority and transform for further applications
    let node_data = node.data();
    let curr_transform = cumul_transform * node_data.transform;
    let curr_depth = cumul_depth + node_data.depth;

    // write the current node to the canvas
    write_model_to_canvas(&node_data.model, curr_transform, curr_depth, canvas);

    // iterate over each of the children and apply them to the canvas
    for child in node.children() {
        write_node_to_canvas(child, curr_transform, curr_depth, canvas);
    }
}

/// Write a model to the canvas using the given absolute
/// priority and the given absolute transform.
fn write_model_to_canvas<C>(
    model: &Shared<VoxModel<C>>,
    cumul_transform: Matrix,
    cumul_depth: Depth,
    canvas: &mut VoxModel<Pair<C>>,
) where
    C: Cell + Default + Copy + ConstZero,
{
    // [!] For this instruction to compile properly,
    // the `Borrow` type must be excluded from this file.
    for (mut index, &cell) in model.borrow().0.indexed_iter() {
        // transform the index to find where to write the cell in the canvas
        //let mut c_index = index;
        cumul_transform.apply(&mut index);

        // Access the current value of the canvas.
        // Based on the priority stored, figure out
        // if the value should be overwritten or not.
        canvas.0[index].write(cell, cumul_depth);
    }
}

/// Type used to encode the priority/depth of a cell
pub type Depth = isize;

/// cell item with associated depth value
#[derive(Debug, Clone, Copy)]
pub struct Pair<C> {
    /// Actual value of the cell
    value: C,

    /// Depth/priority of the value
    depth: Depth,
}

impl<C> Pair<C> {
    /// Create a new pair of value and depth
    #[inline]
    pub fn new(value: C, depth: Depth) -> Self {
        Self { value, depth }
    }

    /// Write the new value if the depth is higher
    pub fn write(&mut self, value: C, depth: Depth)
    where
        C: Cell,
    {
        // current cell is actually empty or we are writing
        // a new cell with higher priority which is not empty.
        if self.value.is_empty() || (self.depth <= depth && !value.is_empty()) {
            self.value = value;
            self.depth = depth;
        }
    }
}

/// Default value for this pair
impl<C> ConstZero for Pair<C>
where
    C: ConstZero,
{
    const ZERO: Self = Self {
        value: C::ZERO,
        depth: Depth::MIN,
    };
}

/// Default value for this pair
impl<C> Default for Pair<C>
where
    C: Default,
{
    #[inline]
    fn default() -> Self {
        Self {
            value: C::default(),
            depth: Depth::MIN,
        }
    }
}

/// Needed to be implemented to implement ConstZero
impl<C> Zero for Pair<C>
where
    C: Zero,
{
    #[inline]
    fn zero() -> Self {
        Self {
            value: C::zero(),
            depth: Depth::MIN,
        }
    }

    #[inline]
    fn is_zero(&self) -> bool {
        self.value.is_zero()
    }
}

/// Needed to be implemented to implement Zero
impl<C> Add for Pair<C>
where
    C: Add<Output = C>,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value + rhs.value,
            depth: self.depth + rhs.depth,
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
