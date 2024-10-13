use crate::config::CircuitConfig;
use crate::Channel;
use bevy::asset::{Asset, Handle};
use bevy::ecs::system::EntityCommands;
use bevy::pbr::Material;
use bevy::prelude::Entity;
use bevy::reflect::TypePath;
use coppervox_logic::logic::bundled_wire::BundledWire;
use coppervox_logic::logic::constant_gate::ConstantGate;
use coppervox_logic::logic::io_bus::{IOBuffer, IOBus};
use coppervox_logic::logic::muxde::{Demultiplexer, Multiplexer};
use coppervox_logic::logic::wire::WireState;
use coppervox_logic::logic::{operation_gate::*, GateInputs, GateOuputs, Value};
use coppervox_logic::Element;
use coppervox_voxels::generate::{Builder, ElementType};
use num::traits::{WrappingAdd, WrappingMul};
use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::{BitAndAssign, BitOrAssign, Not};
//use std::ops::Neg;
//use num::traits::Inv;

/// Optimized mapper to generate bevy entities from circuit data
pub struct CircuitConverter<V> {
    map: HashMap<V, ElementType<CompType, Channel>>,
}

impl<V: Eq + Hash + Copy> CircuitConverter<V> {
    /// Generate a mapper from a config
    pub fn build(config: &CircuitConfig<V>) -> Self {
        const COMP_TYPES: usize = 14;

        let mut map = HashMap::<V, ElementType<CompType, Channel>>::with_capacity(
            config.wires.len() + COMP_TYPES,
        );

        for (channel, &wire) in (0u32..).zip(config.wires.iter()) {
            map.insert(wire, ElementType::Wire(channel));
        }

        // check if the value match with the one from the attribute
        // if so return the corresponding component type
        macro_rules! match_with {
            ($attr:ident => $comp:expr) => {
                if let Some(voxel) = &config.$attr {
                    map.insert(*voxel, ElementType::Component($comp));
                }
            };
        }
        match_with![ constant       => CompType::Constant             ];
        match_with![ or_gate        => CompType::Gate(GateType::Or)   ];
        match_with![ and_gate       => CompType::Gate(GateType::And)  ];
        match_with![ nor_gate       => CompType::Gate(GateType::Nor)  ];
        match_with![ nand_gate      => CompType::Gate(GateType::Nand) ];
        match_with![ addition       => CompType::Gate(GateType::Add)  ];
        match_with![ multiplication => CompType::Gate(GateType::Mul)  ];
        match_with![ negation       => CompType::Gate(GateType::Neg)  ];
        match_with![ invertion      => CompType::Gate(GateType::Inv)  ];
        match_with![ minimum        => CompType::Gate(GateType::Min)  ];
        match_with![ maximum        => CompType::Gate(GateType::Max)  ];
        match_with![ multiplexer    => CompType::Mux                  ];
        match_with![ demultiplexer  => CompType::Demux                ];
        match_with![ io_bus         => CompType::IOBus                ];
        match_with![ bundled_wire   => CompType::BundledWire          ];

        Self { map }
    }
}

/// Generate circuit elements
pub struct CircuitBuilder<M: Asset, V, B: Element + TypePath> {
    /// Materials for wires
    materials_wires: Vec<Handle<M>>,

    /// Materials for components
    materials_components: HashMap<CompType, Handle<M>>,

    /// Material for invalid elements
    material_neither: Handle<M>,

    /// IO buffer to connect the simulation to the outside world
    io_buffer: Handle<IOBuffer<B>>,

    /// Constraint the value type transmitted over the wires
    phantom: PhantomData<V>,
}

impl<
        M: Material,
        V: 'static
            + Element
            + Default
            + Copy
            + Value
            + BitOrAssign
            + BitAndAssign
            + Not<Output = V>
            + WrappingAdd
            + WrappingMul
            //+ Neg<Output = V>
            //+ Inv<Output = V>
            + Ord
            + From<usize>,
        B: Element + TypePath,
    > Builder<CompType, Channel, M> for CircuitBuilder<M, V, B>
{
    /// Get the material to apply to the entity
    fn get_material(&self, element_type: &ElementType<CompType, Channel>) -> Handle<M> {
        match element_type {
            ElementType::Wire(channel) => {
                let channel = *channel as usize;
                if channel < self.materials_wires.len() {
                    self.materials_wires[channel].clone()
                } else {
                    self.material_neither.clone()
                }
            }
            ElementType::Component(comp_type) => self
                .materials_components
                .get(comp_type)
                .cloned()
                .unwrap_or(self.material_neither.clone()),
            _ => self.material_neither.clone(),
        }
    }

    /// Set the entity to behave as a wire
    fn set_wire(&self, cmd: &mut EntityCommands, channel: &Channel) {
        cmd.insert(WireState::<V, Channel>::new(*channel));
    }

    /// Set the entity to behave as a component
    fn set_component(
        &self,
        cmd: &mut EntityCommands,
        comp_type: &CompType,
        voxel_count: usize,
        inputs: &[Entity],
        outputs: &[Entity],
    ) {
        macro_rules! insert_io {
            (I) => {
                cmd.insert(GateInputs::new(inputs))
            };
            (O) => {
                cmd.insert(GateOuputs::new(outputs))
            };
            (IO) => {
                cmd.insert((GateInputs::new(inputs), GateOuputs::new(outputs)))
            };
        }

        match comp_type {
            CompType::Constant => {
                insert_io![O];
                cmd.insert(ConstantGate::new(V::from(voxel_count)));
            }
            CompType::Gate(gate_type) => {
                insert_io![IO];
                cmd.insert(OperationGate::new(gate_type.get_func::<V>()));
            }
            CompType::Mux => {
                insert_io![IO];
                cmd.insert(Multiplexer::new(|v: V| v));
            }
            CompType::Demux => {
                insert_io![IO];
                cmd.insert(Demultiplexer::new(|v: V| v));
            }
            CompType::IOBus => {
                insert_io![IO];
                cmd.insert(IOBus::new(self.io_buffer.clone()));
            }
            CompType::BundledWire => {
                insert_io![IO];
                cmd.insert(BundledWire);
            }
        }
    }
}

/// The various types of components that can be created
#[derive(Hash, PartialEq, Eq)]
enum CompType {
    /// A constant
    Constant,

    /// A logic or arithmetic gate
    Gate(GateType),

    /// A multiplexer
    Mux,

    /// A demultiplexer
    Demux,

    /// An IO bus
    IOBus,

    /// A bundled wire
    BundledWire,
}

/// The various types of logic or arithmetic gates that can be created
#[derive(Hash, PartialEq, Eq)]
enum GateType {
    /// OR gate
    Or,

    /// AND gate
    And,

    /// NOR gate
    Nor,

    /// NAND gate
    Nand,

    /// + operation
    Add,

    /// * operation
    Mul,

    /// - unary operation
    Neg,

    /// 1 / x operation
    Inv,

    /// MIN function
    Min,

    /// MAX function
    Max,
}

impl GateType {
    fn get_func<
        V: 'static
            + Default
            + Copy
            + Value
            + BitOrAssign
            + BitAndAssign
            + Not<Output = V>
            + WrappingAdd
            + WrappingMul
            //+ Neg<Output = V>
            //+ Inv<Output = V>
            + Ord,
    >(
        &self,
    ) -> for<'v> fn(&'v [V]) -> V {
        match *self {
            Self::Or => bitwise_or::<V>,
            Self::And => bitwise_and::<V>,
            Self::Nor => bitwise_nor::<V>,
            Self::Nand => bitwise_nand::<V>,
            Self::Add => arithmetic_add::<V>,
            Self::Mul => arithmetic_multiply::<V>,
            Self::Min => find_min::<V>,
            Self::Max => find_max::<V>,
            _ => does_nothing::<V>,
        }
    }
}

/// A function that does nothing for gates that are not yet supported
fn does_nothing<V: Default>(_: &[V]) -> V {
    V::default()
}
