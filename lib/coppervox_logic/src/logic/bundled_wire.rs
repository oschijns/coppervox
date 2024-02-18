use super::{wire::WireState, GateInputs, GateOuputs, Value, ValueBuffer};
use crate::Element;
use bevy::prelude::*;

/// Emit a constant value.
/// This gate has only outputs.
#[derive(Component)]
pub struct BundledWire;

/// Used to construct a constant gate
#[derive(Bundle)]
pub struct BundeledWireBundle {
    bundled_wire: BundledWire,
    inputs: GateInputs,
    outputs: GateOuputs,
}

/// For each constant gate in the simulation,
/// output its value.
pub fn process<V: Element + Copy + Value, C: Element + Copy, B: ValueBuffer<V, C>>(
    bundled_query: Query<(&GateInputs, &GateOuputs), With<BundledWire>>,
    mut wire_query: Query<&mut WireState<V, C>>,
) {
    // for each bundled wire
    for (ref_inputs, ref_outputs) in bundled_query.iter() {
        // read all of the input values and transmit
        // them to the outputs based on channels
        let mut buffer = B::default();

        // for each wire input references,
        // store the value in the temporary buffer
        for &id in ref_inputs.wire_references.iter() {
            if let Ok(wire) = wire_query.get(id) {
                buffer.fuse_with(&wire.get_channel(), &wire.get_value());
            }
        }

        // for each output reference,
        // recover the corresponding value from the buffer
        for &id in ref_outputs.wire_references.iter() {
            if let Ok(mut wire) = wire_query.get_mut(id) {
                let channel = wire.get_channel();
                wire.fuse_value(&buffer.get(&channel));
            }
        }
    }
}
