use super::{wire::WireState, GateInputs, GateOuputs, Value, ValueBuffer};
use crate::Element;
use bevy::prelude::*;

/// Behave as a connector between this simulation and the outside world.
/// Outputs will allow to inject data from the outside world.
/// Inputs will allow to send data to the outside world.
#[derive(Component)]
pub struct IOBus<B> {
    /// buffer containing data from outside the circuit
    input_buffer: B,

    /// buffer to send data outside of the circuit
    output_buffer: B,
}

/// Used to construct a operation gate
#[derive(Bundle)]
pub struct IOBusBundle<B: Element> {
    io_bus: IOBus<B>,
    inputs: GateInputs,
    outputs: GateOuputs,
}

/// For each multiplexer in the simulation.
/// Read the inputs and combine them into a single value.
/// Set each of its outputs to the single value.
pub fn process<V: Element + Copy + Value, C: Element + Copy, B: Element + ValueBuffer<V, C>>(
    mut bus_query: Query<(&mut IOBus<B>, &GateInputs, &GateOuputs)>,
    mut wire_query: Query<&mut WireState<V, C>>,
) {
    // for each IO bus
    for (mut bus, ref_inputs, ref_outputs) in bus_query.iter_mut() {
        // for each wire input references,
        // transmit the value to the output buffer
        for &id in ref_inputs.iter() {
            if let Ok(wire) = wire_query.get(id) {
                bus.output_buffer
                    .fuse_with(&wire.get_channel(), &wire.get_value());
            }
        }

        // for each wire output,
        // pull a value from the input buffer
        for &id in ref_outputs.iter() {
            if let Ok(mut wire) = wire_query.get_mut(id) {
                let channel = wire.get_channel();
                wire.fuse_value(&bus.input_buffer.get(&channel));
            }
        }
    }
}
