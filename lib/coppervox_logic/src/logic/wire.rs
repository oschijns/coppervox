use super::Value;
use crate::Element;
use bevy::prelude::*;

/// The logic component of a wire.
#[derive(Component)]
pub struct WireState<V, C> {
    /// channel used by this wire
    channel: C,

    /// value of the wire from the previous tick
    previous_value: V,

    /// value of the wire computed for the current tick
    next_value: V,
}

impl<V: Value, C: Copy> WireState<V, C> {
    /// Reset inner stored values to prepare for next tick.
    pub fn reset_value(&mut self) {
        self.previous_value = self.next_value;
        self.next_value = V::default();
    }

    /// Set the value of this wire for the current tick
    pub fn set_value(&mut self, value: V) {
        self.next_value.fuse_with(value);
    }

    /// Get the value of this wire for computation in the current tick
    pub fn get_value(&self) -> V {
        self.previous_value
    }

    /// Get the channel of the wire
    pub fn get_channel(&self) -> C {
        self.channel
    }
}

/// For each wire in the simulation,
/// this function will copy the next state
/// value into its previous state value.
pub fn prepare_next_frame<V: Element + Value, C: Element + Copy>(
    mut query: Query<&mut WireState<V, C>>,
) {
    query.for_each_mut(|mut wire| {
        wire.reset_value();
    });
}
