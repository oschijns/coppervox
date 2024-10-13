use clap::Parser;
use clio::Input;
use coppervox_voxels::{
    format_conversion::{magica_voxel, OriginConfig},
    model::VoxModel,
    Cell,
};
use serde::de::DeserializeOwned;
use std::io::{self, Read};

use crate::config::MainConfig;

#[derive(Parser)]
#[command(
    version = "0.0.1",
    about = "Make logic circuits with voxels",
    long_about = "Process a model made of voxels and generate a logic circuit that can be run in realtime."
)]
pub struct Cli {
    /// Voxel file to load
    #[clap(short, long, value_parser)]
    input: Input,

    /// Configuration file to load
    #[clap(short, long, value_parser)]
    config: Input,
}

impl Cli {
    /// Parse the command line arguments and load the file to obtain a config
    pub fn load_config<V: DeserializeOwned>(&self) -> io::Result<MainConfig<V>> {
        serde_yaml::from_reader::<Input, MainConfig<V>>(self.config.clone())
            .map_err(io::Error::other)
    }

    /// Parse the command line arguments and load the file to obtain voxel models
    pub fn load_model<V: Cell + Default + Copy + From<u8>, const D: usize>(
        &self,
        origin: &[OriginConfig; 3],
    ) -> io::Result<Vec<VoxModel<V, D>>> {
        // read the input file to load a Magica Voxel model
        let mut data = Vec::<u8>::new();
        let _ = self.input.clone().read_to_end(&mut data)?;
        let magica = dot_vox::load_bytes(&data).map_err(io::Error::other)?;

        // load models from the file
        let res: Vec<VoxModel<V, D>> = magica
            .models
            .into_iter()
            .map(|model| magica_voxel::convert(&model, origin))
            .collect();

        Ok(res)
    }
}
