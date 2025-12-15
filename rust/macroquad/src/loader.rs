/// Build the runner
pub mod build;

/// Build mesh from quad data
pub mod mesher;

use crate::base::{Colorize, CompType, color::ColorCode};
use coppervox_analyzer::{Model, Voxel};
use serde::Deserialize;
use std::{collections::HashMap, sync::Arc};

/// Configuration loaded from the input files provided
pub struct Loader {
    /// Model loaded from the file
    model: Model,

    /// Configuration loaded from the text file
    config: Config,
}

/// Configuration loaded from a text file such as TOML
#[derive(Debug, Deserialize)]
pub struct Config(HashMap<Voxel, ComponentConfig>);

#[derive(Debug, Deserialize)]
pub struct ComponentConfig {
    /// Define the type of component
    #[serde(rename = "type")]
    comp_type: CompType,

    /// Colors of the component when off
    color_off: ColorCode,

    /// Color of the component when on low power
    color_on_low: ColorCode,

    /// Color of the component when on high power
    color_on_high: ColorCode,

    /// Processed color codes into a config
    #[serde(skip)]
    colorize: Arc<Colorize>,
}

impl Loader {
    /// Create a new config
    #[inline]
    pub fn new(model: Model, config: Config) -> Self {
        Self { model, config }
    }
}

impl ComponentConfig {
    /// Setup colorize component from deserialized values
    #[inline]
    pub fn init(&mut self) {
        self.colorize = Arc::new(Colorize::new(
            self.color_off.0,
            self.color_on_low.0,
            self.color_on_high.0,
        ))
    }
}
