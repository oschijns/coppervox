pub mod format_conversion;
pub mod generate;
pub mod macros;
pub mod model;

/// Trait for cells in a matrix
pub trait Cell {
    /// Return true if the cell is empty
    fn is_empty(&self) -> bool;
}

/// Implement the cell trait for all primitive types
macro_rules! impl_cell {
    ($type:ty) => {
        impl Cell for $type {
            fn is_empty(&self) -> bool {
                *self == 0
            }
        }
    };
}

impl_cell![u8];
impl_cell![u16];
impl_cell![u32];
impl_cell![u64];
impl_cell![usize];
impl_cell![i8];
impl_cell![i16];
impl_cell![i32];
impl_cell![i64];
impl_cell![isize];

/// Container for storing data structures that are shared by multiple objects
type Shared<T> = std::rc::Rc<std::cell::RefCell<T>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell() {
        let empty_cell = 0u32;
        let filled_cell = 1u32;

        assert_eq![empty_cell.is_empty(), true];
        assert_eq![filled_cell.is_empty(), false];
    }
}
