use super::{VoxIndex, VoxModel};
use crate::Cell;
use num::Zero;
use pathfinding::directed::astar::astar_bag;
use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
    ops::Add,
};

/// Path of indexes in a voxel model
pub type VoxPath = Vec<VoxIndex>;

/// Run the A* pathfinding algorithm to find all path from start to end passign
/// through empty cells and avoiding changing direction too often.
pub fn find_path<C: Cell, const D: usize>(
    model: &VoxModel<C, D>,
    start_index: &VoxIndex,
    end_index: &VoxIndex,
) -> Result<Vec<VoxPath>, ()> {
    // create an initial node
    let start_node = NodePath {
        position: *start_index,
        direction: Direction::None,
    };

    if let Some((solutions, _cost)) = astar_bag(
        &start_node,
        |node_path| find_successors(model, node_path),
        |node_path| Cost {
            distance: manhattan_distance(&node_path.position, end_index),
            direction_changes: 0,
        },
        |node_path| &node_path.position == end_index,
    ) {
        // unwrap the result into a list of paths
        Ok(solutions
            .map(|path| path.iter().map(|node| node.position).collect())
            .collect())
    } else {
        Err(())
    }
}

/// Node of a path
#[derive(Clone, Copy)]
struct NodePath {
    /// index of the cell
    position: VoxIndex,

    /// direction the node is moving toward to
    direction: Direction,
}

/// generate a hash for the node using only its position
impl Hash for NodePath {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.position.hash(state);
    }
}

/// test for equality between nodes using only the position
impl PartialEq for NodePath {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position
    }
}
impl Eq for NodePath {}

/// The direction the pathfinder can move
#[repr(C)]
#[derive(Default, PartialEq, Clone, Copy)]
enum Direction {
    /// initial direction
    #[default]
    None = 0,

    /// move along X-axis
    XAxis,

    /// move along Y-axis
    YAxis,

    /// move along Z-axis
    ZAxis,
}

impl Direction {
    /// Given the current direction and a new one,
    /// return the cost of changing direction.
    fn cost(&self, other: Self) -> usize {
        if *self == other {
            0
        } else {
            1
        }
    }
}

/// Estimate the cost of a path
#[derive(Default, Clone, Copy)]
struct Cost {
    /// manhattan distance from the target
    distance: usize,

    /// count the number of times the path had to change direction
    direction_changes: usize,
}

/// Set initial values for cost
impl Zero for Cost {
    /// set initial cost
    fn zero() -> Self {
        Self {
            distance: 0,
            direction_changes: 0,
        }
    }

    /// check if cost is zero
    fn is_zero(&self) -> bool {
        self.distance == 0 && self.direction_changes == 0
    }
}

/// add two costs toghether
impl Add for Cost {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            distance: self.distance + rhs.distance,
            direction_changes: self.direction_changes + rhs.direction_changes,
        }
    }
}

/// Compare the cost of two paths
impl PartialEq for Cost {
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance && self.direction_changes == other.direction_changes
    }
}

impl Eq for Cost {}

/// specify which cost is the lowest based on the distance and the number of direction changes
impl PartialOrd for Cost {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // compare direction changes first
        match self.direction_changes.partial_cmp(&other.direction_changes) {
            // if both pathes have changed direction equally, compare distances
            Some(Ordering::Equal) => self.distance.partial_cmp(&other.distance),
            ord => ord,
        }
    }
}

/// specify which cost is the lowest based on the distance and the number of direction changes
impl Ord for Cost {
    fn cmp(&self, other: &Self) -> Ordering {
        // compare direction changes first
        match self.direction_changes.cmp(&other.direction_changes) {
            // if both pathes have changed direction equally, compare distances
            Ordering::Equal => self.distance.cmp(&other.distance),
            ord => ord,
        }
    }
}

/// return the manhattan distance between two position
#[inline]
fn manhattan_distance(pos_a: &VoxIndex, pos_b: &VoxIndex) -> usize {
    let &[xa, ya, za] = pos_a;
    let &[xb, yb, zb] = pos_b;
    (xa.abs_diff(xb) + ya.abs_diff(yb) + za.abs_diff(zb)) as usize
}

/// find the successors of the current index provided
fn find_successors<C: Cell, const D: usize>(
    model: &VoxModel<C, D>,
    node_path: &NodePath,
) -> Vec<(NodePath, Cost)> {
    const NEIGHBORS: [(VoxIndex, Direction); 6] = [
        ([-1, 0, 0], Direction::XAxis),
        ([0, -1, 0], Direction::YAxis),
        ([0, 0, -1], Direction::ZAxis),
        ([1, 0, 0], Direction::XAxis),
        ([0, 1, 0], Direction::YAxis),
        ([0, 0, 1], Direction::ZAxis),
    ];

    // get the indexes of the neighboring cells
    let [x, y, z] = node_path.position;

    // store the index of free cells in a new vector
    let mut successors = Vec::<(NodePath, Cost)>::with_capacity(NEIGHBORS.len());

    // check each of the six possible neighboring cells
    for ([ox, oy, oz], direction) in NEIGHBORS {
        let index = [x + ox, y + oy, z + oz];

        // if the cell is empty, it is elected
        if model[index].is_empty() {
            successors.push((
                NodePath {
                    position: index,
                    direction,
                },
                Cost {
                    distance: 1,
                    direction_changes: node_path.direction.cost(direction),
                },
            ));
        }
    }

    successors
}
