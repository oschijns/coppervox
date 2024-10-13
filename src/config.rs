use coppervox_voxels::format_conversion::OriginConfig;
use serde::Deserialize;

/// Specify the format of the main configuration file
#[derive(Deserialize)]
pub struct MainConfig<V> {
    /// User inputs configuration file
    input: InputConfig,

    /// circuit interpretation file
    circuit: CircuitConfig<V>,
}

/// Specify the format of the input configuration file
#[derive(Default, Deserialize)]
#[serde(default)]
pub struct InputConfig {}

/// Specify the format of the circuit configuration file
#[derive(Deserialize)]
pub struct CircuitConfig<V> {
    /// Specify how many bits of data are transmitted over a wire
    pub wire_type: WireType,

    /// Voxel values used for the various wires based on channel
    pub wires: Vec<V>,

    /// Voxel value for constant component
    pub constant: Option<V>,

    /// Voxel value for the OR gate
    pub or_gate: Option<V>,

    /// Voxel value for the NOR gate
    pub nor_gate: Option<V>,

    /// Voxel value for the AND gate
    pub and_gate: Option<V>,

    /// Voxel value for the NAND gate
    pub nand_gate: Option<V>,

    /// Voxel value for the ADDITION
    pub addition: Option<V>,

    /// Voxel value for the MULTIPLICATION
    pub multiplication: Option<V>,

    /// Voxel value for the NEGATION
    pub negation: Option<V>,

    /// Voxel value for the INVERTION
    pub invertion: Option<V>,

    /// Voxel value for the MINIMUM gate
    pub minimum: Option<V>,

    /// Voxel value for the MAXIMUM gate
    pub maximum: Option<V>,

    /// Voxel value for the MULTIPLEXER
    pub multiplexer: Option<V>,

    /// Voxel value for the DEMULTIPLEXER
    pub demultiplexer: Option<V>,

    /// Voxel value for the IO BUS
    pub io_bus: Option<V>,

    /// Voxel value for the BUNDLED WIRE component
    pub bundled_wire: Option<V>,

    /// Specify how mesh are generated
    #[serde(default)]
    pub mesh_origin: [OriginConfig; 3],
}

/// Specify the available types of wires
#[derive(Default, Deserialize)]
pub enum WireType {
    /// Wires can only carry zero or one
    #[default]
    #[serde(rename = "boolean", alias = "bool")]
    Boolean,

    /// Wires carry 8 bits of data
    #[serde(rename = "8bits", alias = "8b")]
    Bits8,

    /// Wires carry 16 bits of data
    #[serde(rename = "16bits", alias = "16b")]
    Bits16,

    /// Wires carry 32 bits of data
    #[serde(rename = "32bits", alias = "32b")]
    Bits32,

    /// Wires carry 64 bits of data
    #[serde(rename = "64bits", alias = "64b")]
    Bits64,
}
