pub mod format_conversion;
pub mod generate;
pub mod macros;
pub mod model;

/// Trait for cells in a matrix
pub trait Cell: Default + Copy + Eq {
    /// Return true if the cell is empty
    fn is_empty(&self) -> bool;
}

/// Container for storing data structures that are shared by multiple objects
type Shared<T> = std::rc::Rc<std::cell::RefCell<T>>;
