use super::{wire::WireState, GateInputs, GateOuputs, Value, ValueBuffer};
use crate::{render::Render, Element};
use bevy::prelude::*;

/// Combine multiple input values into a single value.
#[derive(Component)]
pub struct Multiplexer<F> {
    /// function used to combine multiple values into
    /// a single one based on channels
    combine_function: F,
}

impl<F> Multiplexer<F> {
    pub fn new(func: F) -> Self {
        Self {
            combine_function: func,
        }
    }
}

/// Used to construct a multiplexer
#[derive(Bundle)]
pub struct MultiplexerBundle<R: Element, F: Element> {
    render: Render<R>,
    multiplexer: Multiplexer<F>,
    inputs: GateInputs,
    outputs: GateOuputs,
}

/// Split a single value over multiple outputs.
#[derive(Component)]
pub struct Demultiplexer<F> {
    /// function used to split a value into
    /// multiple ones based on channels
    split_function: F,
}

impl<F> Demultiplexer<F> {
    pub fn new(func: F) -> Self {
        Self {
            split_function: func,
        }
    }
}

/// Used to construct a demultiplexer
#[derive(Bundle)]
pub struct DemultiplexerBundle<R: Element, F: Element> {
    render: Render<R>,
    demultiplexer: Demultiplexer<F>,
    inputs: GateInputs,
    outputs: GateOuputs,
}

/// For each multiplexer in the simulation.
/// Read the inputs and combine them into a single value.
/// Set each of its outputs to the single value.
pub fn process_multiplex<
    R: Element + for<'a> From<&'a V>,
    V: Element + Copy + Value,
    C: Element + Copy,
    B: ValueBuffer<V, C>,
    F: Element + Fn(&B) -> V,
>(
    mut mux_query: Query<(&mut Render<R>, &Multiplexer<F>, &GateInputs, &GateOuputs)>,
    mut wire_query: Query<&mut WireState<V, C>>,
) {
    // for each multiplexer
    for (mut render, mux, ref_inputs, ref_outputs) in mux_query.iter_mut() {
        // gather a list of input values to process
        let mut buffer = B::default();

        // for each wire input references
        for &id in ref_inputs.iter() {
            if let Ok(wire) = wire_query.get(id) {
                buffer.fuse_with(&wire.get_channel(), &wire.get_value());
            }
        }

        // combine the values together
        let result = (mux.combine_function)(&buffer);
        render.refresh(&result);

        // set all the ouputs to the computed result based on their channel
        for &id in ref_outputs.iter() {
            if let Ok(mut wire) = wire_query.get_mut(id) {
                wire.fuse_value(&result);
            }
        }
    }
}

/// For each demultiplexer in the simulation.
/// Apply the operation to each of its inputs to compute a single value.
/// Set each of its outputs to the single value.
pub fn process_demultiplex<
    R: Element + for<'a> From<&'a V>,
    V: Element + Copy + Value + Default,
    C: Element + Copy,
    B: ValueBuffer<V, C>,
    F: Element + Fn(&V) -> B,
>(
    mut demux_query: Query<(&mut Render<R>, &Demultiplexer<F>, &GateInputs, &GateOuputs)>,
    mut wire_query: Query<&mut WireState<V, C>>,
) {
    // for each demultiplexer
    for (mut render, demux, ref_inputs, ref_outputs) in demux_query.iter_mut() {
        // combine all of the inputs into a single value
        let mut value = V::default();

        // for each wire input references
        for &id in ref_inputs.wire_references.iter() {
            if let Ok(wire) = wire_query.get(id) {
                value.fuse_with(&wire.get_value());
            }
        }

        // Split the values and store the result in a buffer
        let buffer = (demux.split_function)(&value);
        render.refresh(&value);

        // set the outputs to values splitted based on their channels
        for &id in ref_outputs.wire_references.iter() {
            if let Ok(mut wire) = wire_query.get_mut(id) {
                let channel = wire.get_channel();
                wire.fuse_value(&buffer.get(&channel));
            }
        }
    }
}
