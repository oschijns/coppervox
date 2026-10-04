use coppervox_common::rgb::{RGB8, RGBA8};
use regex::Regex;
use serde::{Deserialize, Deserializer, de};
use std::{num::ParseIntError, sync::LazyLock};

/// Identifier for a color gradient
pub type GradientIndex = usize;

/// Define how a component should be colorized
#[derive(Debug, Clone, Copy, PartialEq, Default, Deserialize)]
pub struct ColorGradient {
    /// OFF color (0x0)
    #[serde(deserialize_with = "deserialize_color")]
    pub off: RGB8,

    /// ON low power color (0x1)
    #[serde(deserialize_with = "deserialize_color", rename = "low")]
    pub on_low: RGB8,

    /// ON high power color (0xFFFF)
    #[serde(deserialize_with = "deserialize_color", rename = "high")]
    pub on_high: RGB8,
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

/// Thin wrapper to parse an hexadecimal string into a color
#[inline]
fn deserialize_color<'de, D>(deserializer: D) -> Result<RGB8, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    rgb_from_hex(&s).map_err(de::Error::custom)
}

/// Regex for identifying colors encoded as hexadecimal strings
static RGB_HEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^#([0-9A-Fa-f]{2}){3}$").unwrap());

/// Try to identify a color from an hexadecimal string
pub fn rgb_from_hex(hex: &str) -> Result<RGB8, ColorError> {
    if RGB_HEX.is_match(hex) {
        // trim the first '#'
        let hex = hex[1..].to_string();

        // Parse the whole value
        let [_, r, g, b] = u32::from_str_radix(&hex, 16)?.to_be_bytes();

        // Convert the bytes into floats
        Ok(RGB8::new(r, g, b))
    } else {
        Err(ColorError::UnexpectedPattern(hex.to_string()))
    }
}

/// Regex for identifying colors encoded as hexadecimal strings
static RGBA_HEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^#([0-9A-Fa-f]{2}){3,4}$").unwrap());

/// Try to identify a color from an hexadecimal string
pub fn rgba_from_hex(hex: &str) -> Result<RGBA8, ColorError> {
    if RGBA_HEX.is_match(hex) {
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
        Ok(RGBA8::new(r, g, b, a))
    } else {
        Err(ColorError::UnexpectedPattern(hex.to_string()))
    }
}
