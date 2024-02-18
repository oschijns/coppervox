use super::{wire::WireState, GateInputs, GateOuputs, Value};
use crate::Element;
use bevy::prelude::*;
use std::{
    cmp::{max, min},
    ops::{AddAssign, BitAndAssign, BitOrAssign, MulAssign, Not},
};

/// Apply an operation to generate a value.
/// This gate apply an operation to all of its inputs.
/// The resulting value will be sentto all of its outputs.
/// The operation must be associative and commutative.
/// It can be a bitwise logic operation (OR, AND, NOR, NAND)
/// It can be arithmetic (add and multiply)
/// It can be min or max
#[derive(Component)]
pub struct OperationGate<O> {
    /// operation function applied to all the input values
    /// to compute a single result
    operation: O,
}

/// Used to construct a operation gate
#[derive(Bundle)]
pub struct OperationGateBundle<O: Element> {
    gate: OperationGate<O>,
    inputs: GateInputs,
    outputs: GateOuputs,
}

/// For each operation gate in the simulation.
/// Apply the operation to each of its inputs to compute a single value.
/// Set each of its outputs to the single value.
pub fn process<V: Element + Copy + Value, C: Element, O: Element + Fn(&[V]) -> V>(
    gate_query: Query<(&OperationGate<O>, &GateInputs, &GateOuputs)>,
    mut wire_query: Query<&mut WireState<V, C>>,
) {
    // for each operation gate
    for (gate, ref_inputs, ref_outputs) in gate_query.iter() {
        // gather a list of input values to process
        let mut values = Vec::<V>::with_capacity(ref_inputs.len());

        // for each wire input references
        for &id in ref_inputs.wire_references.iter() {
            if let Ok(wire) = wire_query.get(id) {
                values.push(wire.get_value());
            }
        }

        // apply the operation stored in the gate to every values in the list
        let result = (gate.operation)(&values);

        // set all the ouputs to the computed result
        for &id in ref_outputs.iter() {
            if let Ok(mut wire) = wire_query.get_mut(id) {
                wire.fuse_value(&result);
            }
        }
    }
}

/// bitwise OR operation
pub fn bitwise_or<V: Copy + Value + BitOrAssign>(values: &[V]) -> V {
    let mut result = V::default();
    for &v in values.iter() {
        result |= v;
    }
    result
}

/// bitwise AND operation
pub fn bitwise_and<V: Copy + Value + BitAndAssign>(values: &[V]) -> V {
    let mut result = V::default();
    for &v in values.iter() {
        result &= v;
    }
    result
}

/// bitwise NOR operation
pub fn bitwise_nor<V: Copy + Value + BitOrAssign + Not<Output = V>>(values: &[V]) -> V {
    let mut result = V::default();
    for &v in values.iter() {
        result |= v;
    }
    !result
}

/// bitwise NAND operation
pub fn bitwise_nand<V: Copy + Value + BitAndAssign + Not<Output = V>>(values: &[V]) -> V {
    let mut result = V::default();
    for &v in values.iter() {
        result &= v;
    }
    !result
}

/// arithmetic ADD operation
pub fn arithmetic_add<V: Copy + Value + AddAssign>(values: &[V]) -> V {
    let mut result = V::default();
    for &v in values.iter() {
        result += v;
    }
    result
}

/// arithmetic MULTIPLY operation
pub fn arithmetic_mul<V: Copy + Value + MulAssign>(values: &[V]) -> V {
    let mut result = V::default();
    for &v in values.iter() {
        result *= v;
    }
    result
}

/// find the minimum operation
pub fn find_min<V: Copy + Value + Ord>(values: &[V]) -> V {
    if values.is_empty() {
        V::default()
    } else {
        let mut result = values[0];
        for &v in values.iter() {
            result = min(result, v);
        }
        result
    }
}

/// find the maximum operation
pub fn find_max<V: Copy + Value + Ord>(values: &[V]) -> V {
    if values.is_empty() {
        V::default()
    } else {
        let mut result = values[0];
        for &v in values.iter() {
            result = max(result, v);
        }
        result
    }
}
