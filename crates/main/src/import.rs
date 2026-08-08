use coppervox_common::Model;
use coppervox_importer::{dot_vox, magica};
use std::path::Path;

/// Error encountered when importing voxel files
#[derive(Debug, thiserror::Error, Clone)]
pub enum ImportError {
    /// No file extension on the file, could not deduce file type
    #[error("Missing extension in the model file")]
    NoFileExt,

    /// Unsupported file type
    #[error("Unsupported file type: \"{0}\"")]
    UnsupportedFile(String),

    /// Error encountered when loading a Magica Voxel model
    #[error("MagicaVoxel error: {0}")]
    MagicaVoxel(String),
}

/// Read the file at the given path and load voxel data from it
pub fn import_model(path: &Path, chunk_dim: [usize; 3]) -> Result<Model, ImportError> {
    if let Some(ext) = path.extension() {
        let ext = ext.to_string_lossy();
        match ext.as_ref() {
            "vox" => {
                // Load the Magica Voxel file
                let dotvox = match dot_vox::load(&path.to_string_lossy()) {
                    Ok(dotvox) => dotvox,
                    Err(error) => {
                        return Err(ImportError::MagicaVoxel(error.to_string()));
                    }
                };

                // Convert it into a standard voxel model
                let remap = magica::Remapper::default();
                Ok(remap.convert(&dotvox, chunk_dim))
            }
            "gox" => {
                unimplemented!("Goxel file not yet supported");
            }
            "xcf" => {
                unimplemented!("GIMP file not yet supported");
            }
            _ => Err(ImportError::UnsupportedFile(ext.to_string())),
        }
    } else {
        Err(ImportError::NoFileExt)
    }
}
