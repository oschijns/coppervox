use super::{wire::WireState, GateOuputs, Value};
use crate::Element;
use bevy::prelude::*;

/// Emit a constant value.
/// This gate has only outputs.
#[derive(Component)]
pub struct ConstantGate<V> {
    value: V,
}

/// Used to construct a constant gate
#[derive(Bundle)]
pub struct ConstantGateBundle<V: Element> {
    gate: ConstantGate<V>,
    outputs: GateOuputs,
}

/// For each constant gate in the simulation,
/// output its value.
pub fn process<V: Element + Value, C: Element + Copy>(
    gate_query: Query<(&ConstantGate<V>, &GateOuputs)>,
    mut wire_query: Query<&mut WireState<V, C>>,
) {
    // for each constant gate
    for (gate, ref_outputs) in gate_query.iter() {
        // for each wire output references
        for &id in ref_outputs.iter() {
            // find the wire entity to write to
            if let Ok(mut wire) = wire_query.get_mut(id) {
                // write the value to the wire
                wire.set_value(gate.value);
            }
        }
    }
}
