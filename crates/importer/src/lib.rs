// Method for loading a common voxel file format and convert it into a model
pub mod voxel;

/// Load Aseprite as voxel files
pub mod aseprite;

/// Load GIMP files
pub mod gimp;

// re-export crates
pub use coppervox_common as common;
use voxconv::{DependenciesImpl, ext::load_with_ext};
use xcf::Xcf;

use crate::{
    aseprite::{AsepriteImportError, convert_aseprite},
    gimp::convert_gimp,
    voxel::convert_voxel,
};
use aseprite_loader::loader::AsepriteFile;
use coppervox_common::Model;
use coppervox_config::identify::VoxelIdentifier;
use std::{fs, path::Path, str::FromStr};

/// Given the path to a file, identify the type of the file read its data and
/// construct a model to analyze.
pub fn import_model(
    path: &Path,
    vox_ident: &VoxelIdentifier,
    chunk_dim: [usize; 3],
) -> Result<Model, ImportError> {
    // Use the file extension to deduce how to read the file
    if let Some(ext) = path.extension() {
        let ext = ext.to_string_lossy();
        let Ok(format) = FileFormat::from_str(&ext) else {
            return Err(ImportError::UnsupportedFile(ext.to_string()));
        };

        // For now, we only support identifying voxels by color
        let Some(color_space) = &vox_ident.color_space else {
            return Err(ImportError::MissingColorSpace);
        };

        // Allocate a model to fill with voxels
        let mut model = Model::new(chunk_dim);

        // The file is either a common voxel file format to read with voxconv
        // or it is a picture file format with multiple layers.
        if let Some(format) = format.to_voxel_format() {
            let file = load_with_ext(&DependenciesImpl, format, path)?;
            convert_voxel(file, color_space, &mut model)?;
        } else {
            match format {
                FileFormat::Xcf => {
                    let file = Xcf::open(path)?;
                    convert_gimp(&file, color_space, &mut model)?;
                }
                FileFormat::Aseprite => {
                    // load the aseprite file
                    let data = fs::read(path)?;
                    let file = AsepriteFile::load(&data).map_err(AsepriteImportError::Load)?;

                    // Read data from the aseprite file
                    convert_aseprite(&file, color_space, &mut model)?;
                }
                _ => {
                    panic!("Should have been handled by the `to_voxel_format` call.");
                }
            }
        }
        Ok(model)
    } else {
        Err(ImportError::NoFileExt)
    }
}

/// File formats supported for loading a voxel model
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumString)]
#[strum(ascii_case_insensitive)]
pub enum FileFormat {
    /// Goxel
    Gox,

    /// Magica Voxel
    Vox,

    /// Qubicle Binary
    Qb,

    /// Qubicle Binary Tree
    Qbt,

    /// Qubicle Construction Library
    Qbcl,

    /// Voxel Max
    VMax,

    /// Voxel JSON
    #[strum(serialize = "voxj", serialize = "voxjz")]
    Voxj,

    /// Gimp
    Xcf,

    /// Aseprite
    #[strum(serialize = "aseprite", serialize = "ase")]
    Aseprite,
}

impl FileFormat {
    /// If the format is a voxel file format get the corresponding type from voxconv crate
    #[rustfmt::skip]
    pub fn to_voxel_format(self) -> Option<voxconv::ReadFormat> {
        match self {
            Self::Gox  => Some(voxconv::ReadFormat::Goxl),
            Self::Vox  => Some(voxconv::ReadFormat::MVox),
            Self::Qb   => Some(voxconv::ReadFormat::Qb  ),
            Self::Qbt  => Some(voxconv::ReadFormat::Qbt ),
            Self::Qbcl => Some(voxconv::ReadFormat::Qbcl),
            Self::VMax => Some(voxconv::ReadFormat::VMax),
            Self::Voxj => Some(voxconv::ReadFormat::Voxj),
            _ => None,
        }
    }
}

/// Error encountered when importing voxel files
#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    /// The file could not be read from disk.
    #[error("Could not read file: {0}")]
    Io(#[from] std::io::Error),

    /// No file extension on the file, could not deduce file type
    #[error("Missing extension in the model file")]
    NoFileExt,

    /// Unsupported file type
    #[error("Unsupported file type: \"{0}\"")]
    UnsupportedFile(String),

    /// Invalid or inconsistent scene graph in the voxel file
    #[error("Invalid voxel scene graph")]
    InvalidScene,

    /// The configuration identifies voxels by color, but the file has no base colors
    #[error("The voxel file does not provide colors required by the configuration")]
    MissingVoxelColors,

    /// No color space defined, thus voxels cannot be identified by color
    #[error("Missing color space for identifying voxels by color")]
    MissingColorSpace,

    /// A scene transform produced a non-finite or out-of-range voxel position
    #[error("Invalid voxel position after applying scene transforms")]
    InvalidPosition,

    /// A scene transform cannot be represented on the integer voxel grid
    #[error("Voxel scene transform is not integer-grid aligned")]
    InvalidTransform,

    #[error("Voxconv: {0}")]
    VoxConv(#[from] voxconv::Error),

    #[error("Voxcore: {0}")]
    VoxCore(#[from] voxcore::Error),

    #[error("Color space: {0}")]
    ColorSpace(#[from] coppervox_common::color_space::LockError),

    #[error("GIMP: {0}")]
    Xcf(#[from] xcf::Error),

    #[error("Aseprite: {0}")]
    Aseprite(#[from] crate::aseprite::AsepriteImportError),
}
