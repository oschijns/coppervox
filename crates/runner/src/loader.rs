/// Build a single mesh from quad data
pub mod uni_mesher;

/// Process the model and extract data in the order expected by the GPU
pub mod processor;

/// Convert the data into types supported by the GPU
pub mod converter;

/// Allow loading a WASM module to be run inbetween ticks
pub mod wasm_module;

use crate::shader_const::GpuOp;
use coppervox_analyzer::Label;
use coppervox_common::Voxel;
use coppervox_config::component::CompType;
use kiss3d::{prelude::GpuMesh3d, procedural::RenderMesh};
use std::{cell::RefCell, rc::Rc};

/// Error reporting when accessing information
#[derive(thiserror::Error, Debug, Clone, Copy)]
pub enum Error {
    #[error("Analysis error: {0}")]
    Analysis(#[from] coppervox_analyzer::Error),

    #[error("No configuration defined for the voxel type {0}")]
    NoConfigVoxel(Voxel),

    #[error("No configuration defined for the component label {0}")]
    NoConfigLabel(Label),

    #[error("Node 0x{0:04x} in the incidence matrix is not mapped")]
    NodeIndexUnmapped(usize),

    #[error("Too many connections defined on a component")]
    TooManyConnections,
}

/// Reference-counted mesh
pub type RcMesh = Rc<RenderMesh>;

/// Reference to a GPU mesh, held in an `Rc` and protected by a `RefCell`
pub type RcGpuMesh = Rc<RefCell<GpuMesh3d>>;

impl GpuOp {
    /// Get the operation value to submit to the GPU
    #[rustfmt::skip]
    pub fn from_type(comp_type: &CompType) -> Self {
        match comp_type {
            CompType::Or   => Self::Or,
            CompType::And  => Self::And,
            CompType::Nor  => Self::Nor,
            CompType::Nand => Self::Nand,
            CompType::Add  => Self::Add,
            CompType::Mul  => Self::Mul,
            CompType::Min  => Self::Min,
            CompType::Max  => Self::Max,
            CompType::WasmInput(_) => Self::Max,
            _ => Self::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wasm_input_uses_or_to_combine_connected_values() {
        assert_eq!(
            GpuOp::from_type(&CompType::WasmInput(0)) as u32,
            GpuOp::Max as u32
        );
    }
}
