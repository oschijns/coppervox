use crate::{
    color::ColorGradient,
    component::CompType,
    mapping::{Config, Mapping},
};
use coppervox_common::{List, Voxel, color_space::ColorSpace};

/*
 * TODO:
 * There might be an issue here where the voxel index might be too large and
 * the definitions vector use too much memory. We define a constant to arbitraly
 * limit the size of this vector.
 */

/// Avoid having too many definitions
const DEF_LIMIT: usize = u16::MAX as usize;

/// Given a voxel identify it and recover its component definition
pub struct VoxelIdentifier {
    /// Optional color space partitioning if we are identifying voxel by color
    pub color_space: Option<ColorSpace>,

    /// List of component definition identified by their voxel index
    pub definitions: List<(CompType, ColorGradient)>,
}

impl VoxelIdentifier {
    /// Convert the config into an exploitable voxel identifier
    pub fn make(config: Config) -> Self {
        match &config.mapping {
            Mapping::Index(map) => {
                // Create a buffer to store components
                let mut defs = Vec::new();

                for (&index, comp) in map.iter() {
                    let index = index as usize;

                    // Voxel index is too high and will result in too much memory consumption.
                    if index > DEF_LIMIT {
                        panic!("Voxel index is too high")
                    }

                    // Ensure the buffer is large enough
                    if defs.len() <= index {
                        defs.resize(index, Default::default());
                    }

                    // Get the color gradient for the component
                    let gradient = config.get_gradient(comp);
                    defs.insert(index, (comp.define.clone(), gradient));
                }

                Self {
                    color_space: None,
                    definitions: defs.into_boxed_slice(),
                }
            }
            Mapping::Color(map) => {
                // We are building two lists:
                // one for setting the color space partitioning
                // one for recovering the component config from its index
                let count = map.len();
                let mut colors = Vec::with_capacity(count);
                let mut defs = Vec::with_capacity(count + 1);
                defs.push(Default::default());

                // Associate the color with a voxel index
                for (index, (&color, comp)) in map.iter().enumerate() {
                    // Zero is reserved for empty voxels in the model.
                    let index = index as u32 + 1;
                    colors.push((color.0, index));

                    let gradient = config.get_gradient(comp);
                    defs.push((comp.define.clone(), gradient));
                }

                Self {
                    color_space: Some(ColorSpace::load(&colors)),
                    definitions: defs.into_boxed_slice(),
                }
            }
        }
    }

    /// Get the component definition from a voxel index
    #[inline]
    pub fn get_definition(&self, voxel: Voxel) -> Option<(CompType, ColorGradient)> {
        let index = voxel as usize;
        if index < self.definitions.len() {
            Some(self.definitions[index].clone())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::VoxelIdentifier;
    use crate::{
        color::{ColorGradient, HexColor},
        component::CompType,
        mapping::{CompGradient, Component, Config, Mapping},
    };
    use coppervox_common::rgb::RGB8;
    use std::collections::BTreeMap;

    #[test]
    fn color_mappings_leave_zero_reserved_for_empty_voxels() {
        let color = RGB8::new(12, 34, 56);
        let component = Component {
            define: CompType::Or,
            gradient: CompGradient::Gradient(ColorGradient::default()),
        };
        let config = Config {
            tick_rate: 10,
            mapping: Mapping::Color(BTreeMap::from([(HexColor(color), component)])),
            color_gradients: Vec::new(),
        };

        let ident = VoxelIdentifier::make(config);

        assert_eq!(
            ident
                .color_space
                .as_ref()
                .and_then(|space| space.get_color_index(color)),
            Some(1)
        );
        assert_eq!(
            ident.get_definition(0).map(|(component, _)| component),
            Some(CompType::Const(0))
        );
        assert_eq!(
            ident.get_definition(1).map(|(component, _)| component),
            Some(CompType::Or)
        );
    }
}
