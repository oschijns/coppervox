use super::{wire::WireState, GateInputs, GateOuputs, MuxPacker, Value};
use crate::Element;
use bevy::prelude::*;
use std::ops::Index;

/// Allow to split a single value into multiple ones based on channels
pub trait Split<V, C>: Index<C, Output = V> {}

/// Implement Split for MuxPacker
impl<V, C, const N: usize> Split<V, C> for MuxPacker<V, C, N> where usize: From<C> {}

/// Split a single value over multiple outputs.
#[derive(Component)]
pub struct Demultiplexer<S> {
    /// function used to split a value into
    /// multiple ones based on channels
    split_function: S,
}

/// Used to construct a demultiplexer
#[derive(Bundle)]
pub struct DemultiplexerBundle<S: Element> {
    demultiplexer: Demultiplexer<S>,
    inputs: GateInputs,
    outputs: GateOuputs,
}

/// For each demultiplexer in the simulation.
/// Apply the operation to each of its inputs to compute a single value.
/// Set each of its outputs to the single value.
pub fn process<V: Element + Value, C: Element + Copy, B: Split<V, C>, S: Element + Fn(V) -> B>(
    demux_query: Query<(&Demultiplexer<S>, &GateInputs, &GateOuputs)>,
    mut wire_query: Query<&mut WireState<V, C>>,
) {
    // for each demultiplexer
    for (demux, ref_inputs, ref_outputs) in demux_query.iter() {
        // combine all of the inputs into a single value
        let mut value = V::default();

        // for each wire input references
        for &id in ref_inputs.wire_references.iter() {
            if let Ok(wire) = wire_query.get(id) {
                value.fuse_with(wire.get_value());
            }
        }

        // Split the values and store the result in a buffer
        let buffer = (demux.split_function)(value);

        // set the outputs to values splitted based on their channels
        for &id in ref_outputs.wire_references.iter() {
            if let Ok(mut wire) = wire_query.get_mut(id) {
                let channel = wire.get_channel();
                wire.set_value(buffer[channel]);
            }
        }
    }
}
