pub mod constant_gate;
pub mod demultiplexer;
//pub mod event_handler_bus;
pub mod io_bus;
pub mod multiplexer;
pub mod operation_gate;
pub mod wire;

/// Handle the logc aspect of circuits on runtime
///
/// contains the following circuit components:
/// - constant: Emit a constant value to its outputs
/// - OR   gate: apply a bitwise OR   operations on all of its inputs
/// - AND  gate: apply a bitwise AND  operations on all of its inputs
/// - NOR  gate: apply a bitwise NOR  operations on all of its inputs
/// - NAND gate: apply a bitwise NAND operations on all of its inputs
/// - ADD  gate: sum all of its inputs
/// - MUL  gate: multiply all of its inputs
/// - MIN  gate: output the minimum value of all of its inputs
/// - MAX  gate: output the maximum value of all of its inputs
/// - multiplexer: to combine multiple inputs into a single output
/// - demultiplexer: to split a single input into multiple outputs
/// - IO bus: provide an interface to connect the circuit to the outside world
/// - event handler: process events to ouput values
use bevy::prelude::*;
use std::{
    marker::PhantomData,
    ops::{Index, IndexMut},
    slice::Iter,
};

/// Represent a value transmitted over a wire
pub trait Value: Default + Copy {
    /// used to fuse values together
    fn fuse_with(&mut self, other: Self);
}

/// References to the inputs of a given component.
#[derive(Component)]
pub struct GateInputs {
    wire_references: Vec<Entity>,
}

/// References to the outputs of a given components.
#[derive(Component)]
pub struct GateOuputs {
    wire_references: Vec<Entity>,
}

/// implements function to expose the inner vector of GateInputs and GateOutputs
macro_rules! impl_gate_connectors {
    ($typ: ty) => {
        impl $typ {
            // expose iterator
            pub fn iter(&self) -> Iter<'_, Entity> {
                self.wire_references.iter()
            }

            // expose length
            pub fn len(&self) -> usize {
                self.wire_references.len()
            }
        }
    };
}

impl_gate_connectors!(GateInputs);
impl_gate_connectors!(GateOuputs);

/// used by multiplexer and demultiplexer to pack or depack values
pub struct MuxPacker<V, C, const N: usize> {
    values: Vec<V>,
    phantom: PhantomData<C>,
}

impl<V: Default + Clone, C, const N: usize> Default for MuxPacker<V, C, N> {
    fn default() -> Self {
        let mut values = Vec::<V>::with_capacity(N);
        values.resize(N, V::default());
        Self {
            values,
            phantom: Default::default(),
        }
    }
}

impl<V, C, const N: usize> Index<C> for MuxPacker<V, C, N>
where
    usize: From<C>,
{
    type Output = V;

    fn index(&self, index: C) -> &Self::Output {
        &self.values[usize::from(index)]
    }
}

impl<V, C, const N: usize> IndexMut<C> for MuxPacker<V, C, N>
where
    usize: From<C>,
{
    fn index_mut(&mut self, index: C) -> &mut Self::Output {
        &mut self.values[usize::from(index)]
    }
}
