use super::{wire::WireState, GateInputs, GateOuputs, Value};
use crate::Element;
use bevy::prelude::*;

/// Behave as a connector between this simulation and the outside world.
/// Outputs will allow to inject data from the outside world.
/// Inputs will allow to send data to the outside world.
#[derive(Component)]
pub struct IOBus;

/// Used to construct a operation gate
#[derive(Bundle)]
pub struct IOBusBundle {
    io_bus: IOBus,
    inputs: GateInputs,
    outputs: GateOuputs,
}

/// For each multiplexer in the simulation.
/// Read the inputs and combine them into a single value.
/// Set each of its outputs to the single value.
pub fn process<V: Element + Value, C: Element + Copy + Sized>(
    bus_query: Query<(&GateInputs, &GateOuputs), With<IOBus>>,
    mut wire_query: Query<&mut WireState<V, C>>,
) {
    // for each IO bus
    for (ref_inputs, ref_outputs) in bus_query.iter() {
        // for each wire input references
        for &id in ref_inputs.iter() {
            if let Ok(_wire) = wire_query.get(id) {
                todo!();
            }
        }

        // set all the ouputs to the computed result
        for &id in ref_outputs.iter() {
            if let Ok(mut _wire) = wire_query.get_mut(id) {
                todo!();
            }
        }
    }
}
