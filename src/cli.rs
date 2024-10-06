use clap::Parser;
use clio::Input;
use coppervox_voxels::{
    format_conversion::{magica_voxel, OriginConfig},
    generate::labeling::label_model,
    model::VoxModel,
};
use serde::Deserialize;
use std::io::{self, Read};

#[derive(Parser)]
#[command(
    version = "0.0.1",
    about = "Make logic circuits with voxels",
    long_about = "Process a model made of voxels and generate a logic circuit that can be run in realtime."
)]
struct Cli {
    /// Voxel file to load
    #[clap(short, long, value_parser)]
    input: Input,

    /// Configuration file to load
    #[clap(short, long, value_parser)]
    config: Input,
}

/// Read command line interface to generate a simulation from two files
pub fn read_cli() -> io::Result<()> {
    // read the command line parameters
    let cli = Cli::parse();

    // read the config file
    let config = match serde_yaml::from_reader::<Input, Config>(cli.config) {
        Ok(config) => config,
        Err(err) => return Err(io::Error::other(err)),
    };

    // read the input file to load a Magica Voxel model
    let mut data = Vec::<u8>::new();
    let _ = cli.input.clone().read_to_end(&mut data)?;
    let magica = match dot_vox::load_bytes(&data) {
        Ok(model) => model,
        Err(err) => return Err(io::Error::other(err)),
    };

    // load models from the file
    // use a chunk size of 8x8x8 voxels
    let mut models = Vec::<VoxModel<VoxValue, 8>>::new();

    // Magica Voxel files contain multiple models
    if let Some(indexes) = config.models {
        for index in indexes {
            if index < magica.models.len() {
                models.push(magica_voxel::convert(
                    &magica.models[index],
                    &config.mesh_origin,
                ));
            }
        }
    } else {
        for model in magica.models {
            models.push(magica_voxel::convert(&model, &config.mesh_origin));
        }
    }

    // TODO: for now we are just going to pick the first model
    let labeling = label_model::<VoxValue, 8, LabelId, VoxValue>(&models[0]);

    // TODO:
    // read the config file
    // read the voxel file
    // generate labeling, meshes
    // generate bevy entities
    // run the simulation
    Ok(())
}

/// Specify the label type used to identify islands
type LabelId = usize;

/// How are voxel values encoded ?
type VoxValue = u32;

/// Specify the format of the configuration file
#[derive(Default, Deserialize)]
#[serde(default)]
struct Config {
    /// Specify how many bits of data are transmitted over a wire
    wire_type: WireType,

    /// Voxel values used for the various wires based on channel
    wires: Vec<VoxValue>,

    /// Voxel value for constant component
    constant: Option<VoxValue>,

    /// Voxel value for the OR gate
    or_gate: Option<VoxValue>,

    /// Voxel value for the NOR gate
    nor_gate: Option<VoxValue>,

    /// Voxel value for the AND gate
    and_gate: Option<VoxValue>,

    /// Voxel value for the NAND gate
    nand_gate: Option<VoxValue>,

    /// Voxel value for the ADDER
    adder: Option<VoxValue>,

    /// Voxel value for the MULTIPLIER
    multiplier: Option<VoxValue>,

    /// Voxel value for the INVERTER
    inverter: Option<VoxValue>,

    /// Voxel value for the MINIMUM gate
    minimum: Option<VoxValue>,

    /// Voxel value for the MAXIMUM gate
    maximum: Option<VoxValue>,

    /// Voxel value for the MULTIPLEXER
    multiplexer: Option<VoxValue>,

    /// Voxel value for the DEMULTIPLEXER
    demultiplexer: Option<VoxValue>,

    /// Voxel value for the IO BUS
    io_bus: Option<VoxValue>,

    /// Voxel value for the BUNDLED WIRE component
    bundled_wire: Option<VoxValue>,

    /// Specify how mesh are generated
    #[serde(default)]
    mesh_origin: [OriginConfig; 3],

    /// Specify which models to load in the magica voxel file
    models: Option<Vec<usize>>,
}

/// Specify the available types of wires
#[derive(Default, Deserialize)]
enum WireType {
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
