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
) -> Option<Vec<VoxPath>> {
    // create an initial node
    let start_node = PathNode {
        position: *start_index,
        direction: Direction::None,
    };

    if let Some((solutions, _cost)) = astar_bag(
        &start_node,
        |node| find_successors(model, node, end_index),
        |node| Cost {
            distance: manhattan_distance(&node.position, end_index),
            direction_changes: 0,
        },
        |node| &node.position == end_index,
    ) {
        // unwrap the result into a list of paths
        Some(
            solutions
                .map(|path| path.iter().map(|node| node.position).collect())
                .collect(),
        )
    } else {
        None
    }
}

/// Node of a path
#[derive(Clone, Copy)]
struct PathNode {
    /// index of the cell
    position: VoxIndex,

    /// direction the node is moving toward to
    direction: Direction,
}

/// generate a hash for the node using only its position
impl Hash for PathNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.position.hash(state);
    }
}

/// test for equality between nodes using only the position
impl PartialEq for PathNode {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position
    }
}
impl Eq for PathNode {}

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
        Some(self.cmp(other))
    }
}

/// specify which cost is the lowest based on the distance and the number of direction changes
impl Ord for Cost {
    fn cmp(&self, other: &Self) -> Ordering {
        // compare distances first
        match self.distance.cmp(&other.distance) {
            // if both pathes have the same distance, compare direction changes
            Ordering::Equal => self.direction_changes.cmp(&other.direction_changes),
            ord => ord,
        }
    }
}

/// return the manhattan distance between two position
#[inline]
fn manhattan_distance(pos_a: &VoxIndex, pos_b: &VoxIndex) -> usize {
    let &[xa, ya, za] = pos_a;
    let &[xb, yb, zb] = pos_b;
    xa.abs_diff(xb) + ya.abs_diff(yb) + za.abs_diff(zb)
}

/// find the successors of the current index provided
fn find_successors<C: Cell, const D: usize>(
    model: &VoxModel<C, D>,
    node: &PathNode,
    end_index: &VoxIndex,
) -> Vec<(PathNode, Cost)> {
    // specify the offset of all the edges of a cube
    const EDGES: [VoxIndex; 12] = [
        [-1, -1, 0], // 0
        [0, -1, -1], // 1
        [-1, 0, -1], // 2
        [1, -1, 0],  // 3
        [0, 1, -1],  // 4
        [-1, 0, 1],  // 5
        [-1, 1, 0],  // 6
        [0, -1, 1],  // 7
        [1, 0, -1],  // 8
        [1, 1, 0],   // 9
        [0, 1, 1],   // 10
        [1, 0, 1],   // 11
    ];

    // define how find the neighbors of a cell
    const NEIGHBORS: [(VoxIndex, Direction, [usize; 4]); 6] = [
        ([-1, 0, 0], Direction::XAxis, [0, 2, 5, 6]),
        ([0, -1, 0], Direction::YAxis, [0, 1, 3, 7]),
        ([0, 0, -1], Direction::ZAxis, [1, 2, 4, 8]),
        ([1, 0, 0], Direction::XAxis, [3, 8, 9, 11]),
        ([0, 1, 0], Direction::YAxis, [4, 6, 9, 10]),
        ([0, 0, 1], Direction::ZAxis, [5, 7, 10, 11]),
    ];

    // the index of the current cell
    let [x, y, z] = node.position;

    // get the value of the cells at the specified offsets
    let mut edges = Vec::<bool>::with_capacity(EDGES.len());
    for [ox, oy, oz] in EDGES {
        // if the target cell is in reach, ignore neighbors
        let index = [x + ox, y + oy, z + oz];
        edges.push(*end_index == index || model[index].is_empty());
    }

    // store the index of free cells in a new vector
    let mut successors = Vec::<(PathNode, Cost)>::with_capacity(NEIGHBORS.len());
    'out_loop: for ([ox, oy, oz], direction, edge_indexes) in NEIGHBORS {
        // check that the actual cell is empty
        // but if the target cell is in reach, ignore neighbors
        let index = [x + ox, y + oy, z + oz];
        if *end_index == index || model[index].is_empty() {
            // check that the edges are empty too
            for edge_index in edge_indexes {
                if !edges[edge_index] {
                    continue 'out_loop;
                }
            }

            // finally check that the further outside cell is empty too
            // if the target cell is in reach, ignore neighbors
            let outer_index = [x + ox * 2, y + oy * 2, z + oz * 2];
            if *end_index == outer_index || model[outer_index].is_empty() {
                // there is room for the path without interferences
                successors.push((
                    PathNode {
                        position: index,
                        direction,
                    },
                    Cost {
                        distance: 1,
                        direction_changes: node.direction.cost(direction),
                    },
                ));
            }
        }
    }

    successors
}
