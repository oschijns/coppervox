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
pub struct Manifest {
    /// Absolute path to this manifest file
    #[serde(default)]
    pub path: PathBuf,

    /// Tick rate to run the simulation
    pub tick_rate: u32,

    /// Voxel model to load. This is a path relative to the manifest file.
    pub model: PathBuf,

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

    /// Function to initialize the module. It must have signature `() -> u32`,
    /// where zero means success and any nonzero value is an error code.
    #[serde(default = "default_init")]
    pub init: String,

    /// Function to call every tick. It must have signature `() -> u32`,
    /// where zero means success and any nonzero value is an error code.
    #[serde(default = "default_tick")]
    pub tick: String,

    /// Function providing the address and size of the input buffer
    #[serde(default = "default_input")]
    pub input: String,

    /// Function providing the address and size if the output buffer
    #[serde(default = "default_output")]
    pub output: String,
}

#[inline]
fn default_init() -> String {
    "init".to_string()
}

#[inline]
fn default_tick() -> String {
    "tick".to_string()
}

#[inline]
fn default_input() -> String {
    "bind_input".to_string()
}

#[inline]
fn default_output() -> String {
    "bind_output".to_string()
}

impl Manifest {
    /// Evaluate the path to the model
    #[inline]
    pub fn model_path(&self) -> PathBuf {
        self.path.join(&self.model)
    }

    /// Evaluate the path to the WASM module (if any)
    #[inline]
    pub fn wasm_path(&self) -> Option<PathBuf> {
        self.wasm_module
            .as_ref()
            .map(|wasm| self.path.join(&wasm.path))
    }

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
