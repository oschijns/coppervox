pub mod bundled_wire;
pub mod constant_gate;
//pub mod event_handler_bus;
pub mod io_bus;
pub mod muxde;
pub mod operation_gate;
pub mod speaker;
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
use std::slice::Iter;

/// Represent a value transmitted over a wire
pub trait Value {
    /// used to fuse values together
    fn fuse_with(&mut self, other: &Self);
}

/// Implement the value trait for basic integer types
macro_rules! impl_value {
    ($type:ty as bitmask) => {
        impl Value for $type {
            fn fuse_with(&mut self, other: &Self) {
                *self |= other;
            }
        }
    };
    ($type:ty as factor) => {
        impl Value for $type {
            fn fuse_with(&mut self, other: &Self) {
                *self *= other;
            }
        }
    };
}

impl_value![u8 as bitmask];
impl_value![u16 as bitmask];
impl_value![u32 as bitmask];
impl_value![u64 as bitmask];
impl_value![usize as bitmask];
impl_value![i8 as bitmask];
impl_value![i16 as bitmask];
impl_value![i32 as bitmask];
impl_value![i64 as bitmask];
impl_value![isize as bitmask];
impl_value![f32 as factor];
impl_value![f64 as factor];

/// Represent a group of values splitted over multiple channels
pub trait ValueBuffer<V, C>: Default {
    /// get the value stored for this channel
    fn get(&self, channel: &C) -> V;

    /// fuse the value stored for this channel with the one provided
    fn fuse_with(&mut self, channel: &C, value: &V);
}

/// Implement the value buffer trait for basic integer types
macro_rules! impl_value_buffer {
    ($data_type:ty [ $channel_type:ty ]) => {
        impl ValueBuffer<$data_type, $channel_type> for $data_type {
            fn get(&self, channel: &$channel_type) -> $data_type {
                let channel = *channel as u32;
                if self.wrapping_shr(channel) & 0b1 != 0 {
                    <$data_type>::MAX
                } else {
                    0
                }
            }

            fn fuse_with(&mut self, channel: &$channel_type, value: &$data_type) {
                let channel = *channel as u32;
                if *value != 0 {
                    *self |= 0b1u32.wrapping_shl(channel) as $data_type;
                } else {
                    *self &= !0b1u32.wrapping_shl(channel) as $data_type;
                }
            }
        }
    };
}
impl_value_buffer![u8[u8]];
impl_value_buffer![u8[u16]];
impl_value_buffer![u8[u32]];
impl_value_buffer![u8[u64]];
impl_value_buffer![u8[usize]];
impl_value_buffer![u16[u8]];
impl_value_buffer![u16[u16]];
impl_value_buffer![u16[u32]];
impl_value_buffer![u16[u64]];
impl_value_buffer![u16[usize]];
impl_value_buffer![u32[u8]];
impl_value_buffer![u32[u16]];
impl_value_buffer![u32[u32]];
impl_value_buffer![u32[u64]];
impl_value_buffer![u32[usize]];
impl_value_buffer![u64[u8]];
impl_value_buffer![u64[u16]];
impl_value_buffer![u64[u32]];
impl_value_buffer![u64[u64]];
impl_value_buffer![u64[usize]];
impl_value_buffer![usize[u8]];
impl_value_buffer![usize[u16]];
impl_value_buffer![usize[u32]];
impl_value_buffer![usize[u64]];
impl_value_buffer![usize[usize]];

/// References to the inputs of a given component.
#[derive(Component)]
pub struct GateInputs {
    wire_references: Vec<Entity>,
}

impl GateInputs {
    pub fn new(wires: &[Entity]) -> Self {
        Self {
            wire_references: wires.to_vec(),
        }
    }
}

impl From<Vec<Entity>> for GateInputs {
    fn from(wires: Vec<Entity>) -> Self {
        Self {
            wire_references: wires,
        }
    }
}

/// References to the outputs of a given components.
#[derive(Component)]
pub struct GateOuputs {
    wire_references: Vec<Entity>,
}

impl GateOuputs {
    pub fn new(wires: &[Entity]) -> Self {
        Self {
            wire_references: wires.to_vec(),
        }
    }
}

impl From<Vec<Entity>> for GateOuputs {
    fn from(wires: Vec<Entity>) -> Self {
        Self {
            wire_references: wires,
        }
    }
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

            // expose is empty
            pub fn is_empty(&self) -> bool {
                self.wire_references.is_empty()
            }
        }
    };
}

impl_gate_connectors!(GateInputs);
impl_gate_connectors!(GateOuputs);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_buffer() {
        let mut buffer = 0u8;
        <u8 as ValueBuffer<u8, usize>>::fuse_with(&mut buffer, &12usize, &20u8);
        assert_eq![0u8, buffer];

        <u8 as ValueBuffer<u8, u16>>::fuse_with(&mut buffer, &2u16, &20u8);
        assert_eq![0b100u8, buffer];
    }
}
