pub mod connectivity;
pub mod island;
pub mod labeling;
pub mod mesh;
pub mod pattern;

use super::Cell;

/// Trait for labeling
pub trait Label: Cell + Ord + std::hash::Hash {
    /// Create an initial label for the algorithm
    fn start() -> Self;

    /// Move on to the next usable label
    fn next(&mut self) -> &Self;
}
