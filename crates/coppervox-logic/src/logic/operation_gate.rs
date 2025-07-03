use super::{wire::WireState, GateInputs, GateOuputs, Value};
use crate::{render::Render, Element};
use bevy::prelude::*;
use num::traits::{WrappingAdd, WrappingMul, WrappingNeg};
use std::{
    cmp::{max, min},
    ops::{BitAndAssign, BitOrAssign, Not},
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

impl<O> OperationGate<O> {
    /// Create a new operation gate
    pub fn new(operation: O) -> Self {
        Self { operation }
    }
}

/// Used to construct a operation gate
#[derive(Bundle)]
pub struct OperationGateBundle<R: Element, O: Element> {
    render: Render<R>,
    gate: OperationGate<O>,
    inputs: GateInputs,
    outputs: GateOuputs,
}

impl<R: Element + Default, O: Element> OperationGateBundle<R, O> {
    /// Create a bundle given some inputs and outputs
    pub fn new(operation: O, inputs: &[Entity], outputs: &[Entity]) -> Self {
        Self {
            render: Default::default(),
            gate: OperationGate { operation },
            inputs: GateInputs {
                wire_references: Vec::from(inputs),
            },
            outputs: GateOuputs {
                wire_references: Vec::from(outputs),
            },
        }
    }
}

/// For each operation gate in the simulation.
/// Apply the operation to each of its inputs to compute a single value.
/// Set each of its outputs to the single value.
pub fn process<
    R: Element + for<'a> From<&'a V>,
    V: Element + Copy + Value,
    C: Element,
    O: Element + Fn(&[V]) -> V,
>(
    mut gate_query: Query<(&mut Render<R>, &OperationGate<O>, &GateInputs, &GateOuputs)>,
    mut wire_query: Query<&mut WireState<V, C>>,
) {
    // for each operation gate
    for (mut render, gate, ref_inputs, ref_outputs) in gate_query.iter_mut() {
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
        render.refresh(&result);

        // set all the ouputs to the computed result
        for &id in ref_outputs.iter() {
            if let Ok(mut wire) = wire_query.get_mut(id) {
                wire.fuse_value(&result);
            }
        }
    }
}

/// bitwise OR operation
pub fn bitwise_or<V: Default + Copy + Value + BitOrAssign>(values: &[V]) -> V {
    let mut result = V::default();
    for &v in values.iter() {
        result |= v;
    }
    result
}

/// bitwise AND operation
pub fn bitwise_and<V: Default + Copy + Value + BitAndAssign + Not<Output = V>>(values: &[V]) -> V {
    let mut result = !V::default();
    for &v in values.iter() {
        result &= v;
    }
    result
}

/// bitwise NOR operation
pub fn bitwise_nor<V: Default + Copy + Value + BitOrAssign + Not<Output = V>>(values: &[V]) -> V {
    let mut result = V::default();
    for &v in values.iter() {
        result |= v;
    }
    !result
}

/// bitwise NAND operation
pub fn bitwise_nand<V: Default + Copy + Value + BitAndAssign + Not<Output = V>>(values: &[V]) -> V {
    let mut result = !V::default();
    for &v in values.iter() {
        result &= v;
    }
    !result
}

/// arithmetic ADD operation
pub fn arithmetic_add<V: Default + Copy + Value + WrappingAdd>(values: &[V]) -> V {
    let mut result = V::default();
    for v in values.iter() {
        result = result.wrapping_add(v);
    }
    result
}

/// arithmetic MULTIPLY operation
pub fn arithmetic_multiply<V: Default + Copy + Value + WrappingMul>(values: &[V]) -> V {
    if let Some((&first, others)) = values.split_first() {
        let mut result = first;
        for v in others {
            result = result.wrapping_mul(v);
        }
        result
    } else {
        V::default()
    }
}

/// arithmetic NEGATIVE operation
pub fn arithmetic_negate<V: Default + Copy + Value + WrappingAdd + WrappingNeg>(values: &[V]) -> V {
    let mut result = V::default();
    for v in values.iter() {
        result = result.wrapping_add(v);
    }
    result.wrapping_neg()
}

/*
/// arithmetic INVERT operation
pub fn arithmetic_invert<V: Default + Copy + Value + WrappingMul + Zero>(values: &[V]) -> V {
    if let Some((&first, others)) = values.split_first() {
        let mut result = first;
        for v in others {
            result = result.wrapping_mul(v);
        }
        if result.is_zero() {
            1.wrapping_div(result)
        } else {
            V::default()
        }
    } else {
        V::default()
    }
}
*/

/// find the minimum operation
pub fn find_min<V: Default + Copy + Value + Ord>(values: &[V]) -> V {
    if let Some((&first, others)) = values.split_first() {
        let mut result = first;
        for &v in others {
            result = min(result, v);
        }
        result
    } else {
        V::default()
    }
}

/// find the maximum operation
pub fn find_max<V: Default + Copy + Value + Ord>(values: &[V]) -> V {
    if let Some((&first, others)) = values.split_first() {
        let mut result = first;
        for &v in others {
            result = max(result, v);
        }
        result
    } else {
        V::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call_op_gate<V: Copy, O: Fn(&[V]) -> V>(gate: &OperationGate<O>, values: &[V]) -> V {
        (gate.operation)(&values)
    }

    #[test]
    fn test_call() {
        // create the gates
        macro_rules! create {
            ($func:ident) => {
                OperationGate::new(&$func::<u8>)
            };
        }
        let or_gate = create![bitwise_or];
        let and_gate = create![bitwise_and];
        let nor_gate = create![bitwise_nor];
        let nand_gate = create![bitwise_nand];
        let add_gate = create![arithmetic_add];
        let mul_gate = create![arithmetic_multiply];
        let neg_gate = create![arithmetic_negate];
        let min_gate = create![find_min];
        let max_gate = create![find_max];

        // a list of values to operate on
        let values = vec![0b1001, 0b1010, 0b1000];
        assert_eq![0b_____1011, call_op_gate(&or_gate, &values)];
        assert_eq![0b_____1000, call_op_gate(&and_gate, &values)];
        assert_eq![0b1111_0100, call_op_gate(&nor_gate, &values)];
        assert_eq![0b1111_0111, call_op_gate(&nand_gate, &values)];
        assert_eq![0b___1_1011, call_op_gate(&add_gate, &values)];
        assert_eq![0b1101_0000, call_op_gate(&mul_gate, &values)];
        assert_eq![0b1110_0101, call_op_gate(&neg_gate, &values)];
        assert_eq![0b_____1000, call_op_gate(&min_gate, &values)];
        assert_eq![0b_____1010, call_op_gate(&max_gate, &values)];
    }
}
