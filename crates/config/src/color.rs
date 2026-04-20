use kiss3d::color::Rgba;
use regex::Regex;
use serde::{Deserialize, Deserializer, de};
use std::{num::ParseIntError, sync::LazyLock};

/// Identifier for a color set
pub type ColorIndex = usize;

/// Define how a component should be colorized
#[derive(Debug, Clone, Copy, PartialEq, Default, Deserialize)]
pub struct ColorSet {
    /// OFF color (0x0)
    #[serde(deserialize_with = "deserialize_color")]
    pub off: Rgba<u8>,

    /// ON low power color (0x1)
    #[serde(deserialize_with = "deserialize_color", rename = "low")]
    pub on_low: Rgba<u8>,

    /// ON high power color (0xFFFF)
    #[serde(deserialize_with = "deserialize_color", rename = "high")]
    pub on_high: Rgba<u8>,
}

/// Error encountered when reading color data from the configuration
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum ColorError {
    /// Pattern is not an hexadecimal string
    #[error("Could not recognize color from \"{0}\" pattern")]
    UnexpectedPattern(String),

    /// Failed to parse the value as an integer
    #[error("Failed to parse the hexadecimal value")]
    ParseInt(#[from] ParseIntError),
}

/// Thin wrapper around `color_from_hex` function
fn deserialize_color<'de, D>(deserializer: D) -> Result<Rgba<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    color_from_hex(&s).map_err(de::Error::custom)
}

/// Regex for identifying colors encoded as hexadecimal strings
static COLOR_HEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^#([0-9A-Fa-f]{2}){3,4}$").unwrap());

/// Try to identify a color from an hexadecimal string
fn color_from_hex(hex: &str) -> Result<Rgba<u8>, ColorError> {
    if COLOR_HEX.is_match(hex) {
        // trim the first '#'
        let mut hex = hex[1..].to_string();

        // Check if the alpha channel is defined in the string.
        // If not add two extra digits so that the size is always 8.
        if hex.len() == 6 {
            hex += "FF";
        }

        // Parse the whole value
        let [r, g, b, a] = u32::from_str_radix(&hex, 16)?.to_be_bytes();

        // Convert the bytes into floats
        Ok(Rgba::new(r, g, b, a))
    } else {
        Err(ColorError::UnexpectedPattern(hex.to_string()))
    }
}
