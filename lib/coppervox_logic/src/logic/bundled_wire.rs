use super::{wire::WireState, GateInputs, GateOuputs, Value, ValueBuffer};
use crate::{render::Render, Element};
use bevy::prelude::*;

/// Emit a constant value.
/// This gate has only outputs.
#[derive(Component)]
pub struct BundledWire;

/// Used to construct a constant gate
#[derive(Bundle)]
pub struct BundeledWireBundle<R: Element> {
    render: Render<R>,
    bundled_wire: BundledWire,
    inputs: GateInputs,
    outputs: GateOuputs,
}

/// For each constant gate in the simulation,
/// output its value.
pub fn process<
    R: Element + for<'a> From<&'a B>,
    V: Element + Copy + Value,
    C: Element + Copy,
    B: ValueBuffer<V, C>,
>(
    mut bundled_query: Query<(&mut Render<R>, &GateInputs, &GateOuputs), With<BundledWire>>,
    mut wire_query: Query<&mut WireState<V, C>>,
) {
    // for each bundled wire
    for (mut render, ref_inputs, ref_outputs) in bundled_query.iter_mut() {
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

        // update the rendering of the bundled wire
        render.refresh(&buffer);

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
