use super::{wire::WireState, GateInputs, GateOuputs, MuxPacker, Value};
use crate::Element;
use bevy::prelude::*;
use std::ops::IndexMut;

/// Allow to combine multiple values into a single one based on their channels
pub trait Combine<V, C>: Default + IndexMut<C, Output = V> + Into<V> {}

/// Implement Combine for MuxPacker
impl<V: Default + Clone + From<Self>, C, const N: usize> Combine<V, C> for MuxPacker<V, C, N> where
    usize: From<C>
{
}

/// Combine multiple input values into a single value.
#[derive(Component)]
pub struct Multiplexer;

/// Used to construct a multiplexer
#[derive(Bundle)]
pub struct MultiplexerBundle {
    multiplexer: Multiplexer,
    inputs: GateInputs,
    outputs: GateOuputs,
}

/// For each multiplexer in the simulation.
/// Read the inputs and combine them into a single value.
/// Set each of its outputs to the single value.
pub fn process<V: Element + Value + From<B>, C: Element + Copy + Sized, B: Combine<V, C>>(
    mux_query: Query<(&GateInputs, &GateOuputs), With<Multiplexer>>,
    mut wire_query: Query<&mut WireState<V, C>>,
) {
    // for each multiplexer
    for (ref_inputs, ref_outputs) in mux_query.iter() {
        // gather a list of input values to process
        let mut buffer = B::default();

        // for each wire input references
        for &id in ref_inputs.iter() {
            if let Ok(wire) = wire_query.get(id) {
                buffer[wire.get_channel()].fuse_with(wire.get_value());
            }
        }

        // combine the values together
        let result = V::from(buffer);

        // set all the ouputs to the computed result based on their channel
        for &id in ref_outputs.iter() {
            if let Ok(mut wire) = wire_query.get_mut(id) {
                wire.set_value(result);
            }
        }
    }
}
