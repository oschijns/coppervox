pub mod logic;
pub mod render;

use std::marker::{Send, Sync};

/// Regroup the traits that must be implemented by an element of a component.
/// This allow us to define components with generic types.
pub trait Element: Send + Sync + 'static {}

/// Implement this trait if the value can be null/zero
pub trait Nillable {
    fn is_nil(&self) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {}
}
