use super::{wire::WireState, GateInputs, GateOuputs, Value};
use crate::Element;
use bevy::prelude::*;
use std::{
    marker::PhantomData,
    ops::{Index, IndexMut},
};

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

/// Default structure for handling combining and splitting values
/// used by multiplexer and demultiplexer to pack or depack values
pub struct MuxdePack<V, C, const N: usize> {
    values: [V; N],
    phantom: PhantomData<C>,
}

/// Allow to combine multiple values into a single one based on their channels
pub trait MuxCombine<V, C>: Default + IndexMut<C, Output = V> + Into<V> {}

/// Allow to split a single value into multiple ones based on channels
pub trait DemuxSplit<V, C>: Index<C, Output = V> {}

/// For each multiplexer in the simulation.
/// Read the inputs and combine them into a single value.
/// Set each of its outputs to the single value.
pub fn process_multiplex<
    V: Element + Value + From<P>,
    C: Element + Copy + Sized,
    P: MuxCombine<V, C>,
>(
    mux_query: Query<(&GateInputs, &GateOuputs), With<Multiplexer>>,
    mut wire_query: Query<&mut WireState<V, C>>,
) {
    // for each multiplexer
    for (ref_inputs, ref_outputs) in mux_query.iter() {
        // gather a list of input values to process
        let mut pack = P::default();

        // for each wire input references
        for &id in ref_inputs.iter() {
            if let Ok(wire) = wire_query.get(id) {
                pack[wire.get_channel()].fuse_with(wire.get_value());
            }
        }

        // combine the values together
        let result = V::from(pack);

        // set all the ouputs to the computed result based on their channel
        for &id in ref_outputs.iter() {
            if let Ok(mut wire) = wire_query.get_mut(id) {
                wire.set_value(result);
            }
        }
    }
}

/// For each demultiplexer in the simulation.
/// Apply the operation to each of its inputs to compute a single value.
/// Set each of its outputs to the single value.
pub fn process_demultiplex<
    V: Element + Value,
    C: Element + Copy,
    P: DemuxSplit<V, C>,
    S: Element + Fn(V) -> P,
>(
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
        let pack = (demux.split_function)(value);

        // set the outputs to values splitted based on their channels
        for &id in ref_outputs.wire_references.iter() {
            if let Ok(mut wire) = wire_query.get_mut(id) {
                let channel = wire.get_channel();
                wire.set_value(pack[channel]);
            }
        }
    }
}

/// Implement default for `MuxdePack`
impl<V: Default + Copy, C, const N: usize> Default for MuxdePack<V, C, N> {
    fn default() -> Self {
        Self {
            values: [Default::default(); N],
            phantom: Default::default(),
        }
    }
}

/// implement indexing operator for `MuxdePack`
impl<V, C, const N: usize> Index<C> for MuxdePack<V, C, N>
where
    usize: From<C>,
{
    type Output = V;

    fn index(&self, index: C) -> &Self::Output {
        &self.values[usize::from(index)]
    }
}

/// implement mutable indexing operator for `MuxdePack`
impl<V, C, const N: usize> IndexMut<C> for MuxdePack<V, C, N>
where
    usize: From<C>,
{
    fn index_mut(&mut self, index: C) -> &mut Self::Output {
        &mut self.values[usize::from(index)]
    }
}

/// Implement Combine for MuxPacker
impl<V: Default + Copy + From<Self>, C: Default, const N: usize> MuxCombine<V, C>
    for MuxdePack<V, C, N>
where
    usize: From<C>,
{
}

/// Implement Split for MuxPacker
impl<V, C, const N: usize> DemuxSplit<V, C> for MuxdePack<V, C, N> where usize: From<C> {}
