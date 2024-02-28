use super::Value;
use crate::{render::Render, Element};
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

impl<V: Default + Copy, C> WireState<V, C> {
    /// create a new wirestate component
    pub fn new(channel: C) -> Self {
        Self {
            channel,
            previous_value: V::default(),
            next_value: V::default(),
        }
    }

    /// Reset inner stored values to prepare for next tick.
    pub fn reset_value(&mut self) {
        self.previous_value = self.next_value;
        self.next_value = V::default();
    }
}

impl<V: Value, C> WireState<V, C> {
    /// Set the value of this wire for the current tick
    pub fn fuse_value(&mut self, value: &V) {
        self.next_value.fuse_with(value);
    }
}

impl<V: Copy, C> WireState<V, C> {
    /// Get the value of this wire for computation in the current tick
    pub fn get_value(&self) -> V {
        self.previous_value
    }
}

impl<V, C: Copy> WireState<V, C> {
    /// Get the channel of the wire
    pub fn get_channel(&self) -> C {
        self.channel
    }
}

/// For each wire in the simulation,
/// this function will copy the next state
/// value into its previous state value.
pub fn prepare_next_frame<V: Element + Default + Copy, C: Element>(
    mut query: Query<&mut WireState<V, C>>,
) {
    query.iter_mut().for_each(|mut wire| {
        wire.reset_value();
    });
}

/// update the value associated with the rendering of the wires
pub fn refresh_rendered_value<R: Element + for<'a> From<&'a V>, V: Element, C: Element>(
    mut query: Query<(&mut Render<R>, &WireState<V, C>)>,
) {
    query.iter_mut().for_each(|(mut render, wire)| {
        render.refresh(&wire.next_value);
    });
}
