use coppervox_common::rgb::{RGB8, RGBA8};
use regex::Regex;
use serde::{Deserialize, Deserializer, de};
use std::{num::ParseIntError, sync::LazyLock};

/// Identifier for a color gradient
pub type GradientIndex = usize;

/// Wrapper for deserializing a color from a hexadecimal string
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct HexColor(pub RGB8);

/// Define how a component should be colorized
#[derive(Debug, Clone, Copy, PartialEq, Default, Deserialize)]
pub struct ColorGradient {
    /// OFF color (0x0)
    pub off: HexColor,

    /// ON low power color (0x1)
    #[serde(rename = "low")]
    pub on_low: HexColor,

    /// ON high power color (0xFFFF)
    #[serde(rename = "high")]
    pub on_high: HexColor,
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

impl<'de> Deserialize<'de> for HexColor {
    /// Thin wrapper to parse an hexadecimal string into a color
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match rgb_from_hex(&s) {
            Ok(rgb) => Ok(Self(rgb)),
            Err(err) => Err(de::Error::custom(err)),
        }
    }
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
