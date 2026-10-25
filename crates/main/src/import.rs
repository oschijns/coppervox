pub use coppervox_importer::import::import_model;
use coppervox_runner::config::mapping::Config;
use std::{fs, io, path::Path};

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
