use super::{wire::WireState, GateInputs, GateOuputs, Value, ValueBuffer};
use crate::Element;
use bevy::prelude::*;

/// Behave as a connector between this simulation and the outside world.
/// Outputs will allow to inject data from the outside world.
/// Inputs will allow to send data to the outside world.
#[derive(Component)]
pub struct IOBus<B: Element + TypePath> {
    buffer_handle: Handle<IOBuffer<B>>,
}

/// Used to construct a IO bus
#[derive(Bundle)]
pub struct IOBusBundle<B: Element + TypePath> {
    io_bus: IOBus<B>,
    inputs: GateInputs,
    outputs: GateOuputs,
}

/// store the buffers assigned to IO buses in the circuit
#[derive(Asset, TypePath)]
pub struct IOBuffer<B: Element + TypePath> {
    /// buffer containing data from outside the circuit
    input: B,

    /// buffer to send data outside of the circuit
    output: B,
}

/// For each multiplexer in the simulation.
/// Read the inputs and combine them into a single value.
/// Set each of its outputs to the single value.
pub fn process<
    V: Element + Copy + Value,
    C: Element + Copy,
    B: Element + TypePath + ValueBuffer<V, C>,
>(
    bus_query: Query<(&IOBus<B>, &GateInputs, &GateOuputs)>,
    mut buffers: ResMut<Assets<IOBuffer<B>>>,
    mut wire_query: Query<&mut WireState<V, C>>,
) {
    // for each IO bus
    for (io_bus, ref_inputs, ref_outputs) in bus_query.iter() {
        // recover the shared buffer for this IO bus
        if let Some(buffer) = buffers.get_mut(&io_bus.buffer_handle) {
            // for each wire input references,
            // transmit the value to the output buffer
            for &id in ref_inputs.iter() {
                if let Ok(wire) = wire_query.get(id) {
                    buffer
                        .output
                        .fuse_with(&wire.get_channel(), &wire.get_value());
                }
            }

            // for each wire output,
            // pull a value from the input buffer
            for &id in ref_outputs.iter() {
                if let Ok(mut wire) = wire_query.get_mut(id) {
                    let channel = wire.get_channel();
                    wire.fuse_value(&buffer.input.get(&channel));
                }
            }
        }
    }
}
