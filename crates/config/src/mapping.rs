use crate::{
    color::{ColorGradient, GradientIndex, HexColor},
    component::CompType,
};
use coppervox_common::Voxel;
use serde::Deserialize;
use std::{collections::BTreeMap, path::PathBuf};

/*
 * Mapping files ared defined using various encoding schemes:
 * Voxels can be indentified either by index or by color. Color gradients of
 * components can be set directly or indexed. Thus we have a voxel identifier
 * method. An optional color gradient list. And each component has either a
 * gradient index of the gradient definition itself.
 */

// TODO: specify path to voxel model in manifest directly

/// Configuration loaded from a text file such as TOML
#[derive(Debug, Deserialize)]
pub struct Config {
    /// Tick rate to run the simulation
    pub tick_rate: u32,

    /// Associate a voxel value with a type of component
    pub mapping: Mapping,

    /// Associate a color index with a color gradient. If each component has
    /// its own gradient definition, this list is not necessary.
    #[serde(default)]
    pub color_gradients: Vec<ColorGradient>,

    /// WASM module (if any)
    pub wasm_module: Option<WasmModule>,
}

/// Configuration loaded from a text file such as TOML
#[derive(Debug, Clone, Deserialize)]
pub struct Component {
    /// Component definition
    #[serde(rename = "type")]
    pub define: CompType,

    /// Component color gradient
    pub gradient: CompGradient,
}

/// Define mapping between voxel color and component type and color activation gradiants
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(untagged)]
pub enum CompGradient {
    /// Color gradient is identified by an index
    Index(GradientIndex),

    /// Color gradient is defined in place
    Gradient(ColorGradient),
}

/// Define mapping between voxel color and component type and color activation gradiants
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "voxel_ident", content = "components", rename_all = "UPPERCASE")]
pub enum Mapping {
    /// Identify voxels by index
    Index(BTreeMap<Voxel, Component>),

    /// Identify voxels by color
    Color(BTreeMap<HexColor, Component>),
}

/// Define binding for WASM module
#[derive(Debug, Clone, Deserialize)]
pub struct WasmModule {
    /// Relative path to the WASM module
    pub path: PathBuf,

    /// Input buffer for the WASM module
    pub input: WasmBuffer,

    /// Output buffer for the WASM module
    pub output: WasmBuffer,

    /// Function to initialize the module. It must have signature `() -> u32`,
    /// where zero means success and any nonzero value is an error code.
    #[serde(default = "default_init")]
    pub init: String,

    /// Function to call every tick. It must have signature `() -> u32`,
    /// where zero means success and any nonzero value is an error code.
    #[serde(default = "default_tick")]
    pub tick: String,
}

#[inline]
fn default_init() -> String {
    "init".to_string()
}

#[inline]
fn default_tick() -> String {
    "tick".to_string()
}

/// Define the start address and byte size of a WASM IO buffer.
#[derive(Debug, Clone, Deserialize)]
pub struct WasmBuffer {
    /// Start address of the buffer in the WASM module
    pub address: usize,

    /// Size of the buffer in bytes. WASM component slots each occupy one u32.
    pub size: usize,
}

impl Config {
    /// Return true if the configuration is valid.
    /// Each color index should map to a valid color set.
    pub fn validate(&self) -> bool {
        // How many color gradients are defined?
        let max = self.color_gradients.len();

        // Check if the color gradient is defined by index,
        // and if so check it is not out-of-bound.
        macro_rules! check {
            ($comp:ident) => {
                if let CompGradient::Index(index) = $comp.gradient
                    && index >= max
                {
                    return false;
                }
            };
        }

        match &self.mapping {
            Mapping::Index(map) => {
                for comp in map.values() {
                    check!(comp);
                }
            }
            Mapping::Color(map) => {
                for comp in map.values() {
                    check!(comp);
                }
            }
        }
        true
    }

    /// Get the color gradient for the component.
    /// As long as the config was validated before hand, this function will not fail.
    pub fn get_gradient(&self, comp: &Component) -> ColorGradient {
        match comp.gradient {
            CompGradient::Index(i) => self.color_gradients[i],
            CompGradient::Gradient(gradient) => gradient,
        }
    }
}
