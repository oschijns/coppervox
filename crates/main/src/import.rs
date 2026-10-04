use coppervox_common::Model;
use coppervox_importer::{dot_vox, magica};
use coppervox_runner::config::mapping::Config;
use std::{fs, io, path::Path};

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

/// Error encountered when loading the config files
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// No file extension on the file, could not deduce file type
    #[error("Missing extension in the config file")]
    NoFileExt,

    /// Unsupported file type
    #[error("Unsupported file type: \"{0}\"")]
    UnsupportedFile(String),

    /// Error loading the file
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    /// Error encountered when loading a TOML config
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),

    /// Error encountered when loading the RON config
    #[error("RON error: {0}")]
    Ron(#[from] ron::error::SpannedError),
}

/// Read the config file
pub fn load_config(path: &Path) -> Result<Config, ConfigError> {
    if let Some(ext) = path.extension() {
        // Read the text file
        let content = fs::read_to_string(path)?;

        // Based on the extension use the appropriate deserializer
        let ext = ext.to_string_lossy();
        match ext.as_ref() {
            "toml" => Ok(toml::from_str(&content)?),
            "ron" => Ok(ron::from_str(&content)?),
            _ => Err(ConfigError::UnsupportedFile(ext.to_string())),
        }
    } else {
        Err(ConfigError::NoFileExt)
    }
}
