use std::ops::IndexMut;

use super::{wire::WireState, GateOuputs, Value};
use crate::Element;
use bevy::{
    input::{keyboard, ButtonState},
    prelude::*,
};

/// Used to transform events into values.
/// Given an event, its will be converted into values
/// to be sent over its various outputs.
#[derive(Component)]
pub struct EventHandlerBus;

/// Used to construct a operation gate
#[derive(Bundle)]
pub struct EventHandlerBusBundle {
    io_bus: EventHandlerBus,
    outputs: GateOuputs,
}

/// Store the result of the processed events
pub trait Storage: Resource + IndexMut<usize, Output = bool> {}

/// Read keyboard events
pub fn read_key_events<R: Storage>(
    mut events: EventReader<keyboard::KeyboardInput>,
    mut storage: ResMut<R>,
) {
    for event in events.read() {
        storage[event.scan_code as usize] = event.state == ButtonState::Pressed;
    }
}

/// For each multiplexer in the simulation.
/// Read the inputs and combine them into a single value.
/// Set each of its outputs to the single value.
pub fn process<V: Element + Value, C: Element + Copy + Sized>(
    bus_query: Query<&GateOuputs, With<EventHandlerBus>>,
    mut wire_query: Query<&mut WireState<V, C>>,
) {
    // for each IO bus
    for ref_outputs in bus_query.iter() {
        // set all the ouputs to the computed result
        for &id in ref_outputs.iter() {
            if let Ok(mut _wire) = wire_query.get_mut(id) {
                todo!();
            }
        }
    }
}
