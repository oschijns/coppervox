use crate::{
    color::{ColorGradient, GradientIndex, HexColor},
    component::CompType,
};
use coppervox_common::Voxel;
use serde::Deserialize;
use std::collections::BTreeMap;

/*
 * Mapping files ared defined using various encoding schemes:
 * Voxels can be indentified either by index or by color. Color gradients of
 * components can be set directly or indexed. Thus we have a voxel identifier
 * method. An optional color gradient list. And each component has either a
 * gradient index of the gradient definition itself.
 */

/// Configuration loaded from a text file such as TOML
#[derive(Debug, Deserialize)]
pub struct Config {
    /// Associate a voxel value with a type of component
    pub mapping: Mapping,

    /// Associate a color index with a color gradient. If each component has
    /// its own gradient definition, this list is not necessary.
    #[serde(default)]
    pub color_gradients: Vec<ColorGradient>,
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
