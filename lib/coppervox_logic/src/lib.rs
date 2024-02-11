pub mod logic;

use std::marker::{Send, Sync};

/// Regroup the traits that must be implemented by an element of a component.
/// This allow us to define components with generic types.
pub trait Element: Send + Sync + 'static {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {}
}
