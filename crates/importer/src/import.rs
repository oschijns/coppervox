use coppervox_common::Model;
use coppervox_config::identify::VoxelIdentifier;
use std::path::Path;
use voxconv::{ReadFormat, ext::load_with_ext};

/// Read the file at the given path and load voxel data from it
pub fn import_model(
    path: &Path,
    vox_ident: &mut VoxelIdentifier,
    chunk_dim: [usize; 3],
) -> Result<Model, ImportError> {
    if let Some(ext) = path.extension() {
        let ext = ext.to_string_lossy();

        let format = ident_file_format(&ext)?;

        //let vox = load_with_ext(&(), format, path)?;
        todo!()
    } else {
        Err(ImportError::NoFileExt)
    }
}

#[rustfmt::skip]
fn ident_file_format(ext: &str) -> Result<ReadFormat, ImportError> {
    match ext {
        "gox"            => Ok(ReadFormat::Goxl),
        "vox"            => Ok(ReadFormat::MVox),
        "qb"             => Ok(ReadFormat::Qb  ),
        "qbt"            => Ok(ReadFormat::Qbt ),
        "qbcl"           => Ok(ReadFormat::Qbcl),
        "vmax"           => Ok(ReadFormat::VMax),
        "voxj" | "voxjz" => Ok(ReadFormat::Voxj),
        _ => Err(ImportError::UnsupportedFile(ext.to_string())),
    }
}

/// Error encountered when importing voxel files
#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    /// No file extension on the file, could not deduce file type
    #[error("Missing extension in the model file")]
    NoFileExt,

    /// Unsupported file type
    #[error("Unsupported file type: \"{0}\"")]
    UnsupportedFile(String),

    #[error("Voxconv: {0}")]
    VoxConv(#[from] voxconv::Error),
}
