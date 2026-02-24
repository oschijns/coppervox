use coppervox_analyzer::Voxel;
use kiss3d::{color::Color, event::Key};
use regex::Regex;
use serde::{Deserialize, Deserializer, de};
use std::{collections::HashMap, num::ParseIntError, sync::LazyLock};

/// Configuration loaded from a text file such as TOML
#[derive(Debug, Deserialize)]
pub struct Config(pub HashMap<Voxel, CompDef>);

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct CompDef {
    /// Define the type of component
    #[serde(rename = "type")]
    pub comp_type: CompType,

    /// Colors of the component
    pub color_set: ColorSet,
}

/// Values transmitted over components are 32-bits
pub type Value = u32;

/// Values transmitted over components are 32-bits
pub const VALUE_SIZE: usize = 32;

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum CompType {
    /// constant value
    Const(Value) = 0,

    /// bitwise OR operation
    Or,

    /// bitwise AND operation
    And,

    /// bitwise NOR operation
    Nor,

    /// bitwise NAND operation
    Nand,

    /// arithmetic addition
    Add,

    /// arithmetic multiplication
    Mul,

    /// Minimum
    Min,

    /// Maximum
    Max,

    /// Map a key press to a single output value
    InputKey {
        /// The key to track
        key: Key,

        /// The value to output
        value: Value,
    },

    /// Map a list of keys to a mask
    InputMask([Option<Key>; VALUE_SIZE]),

    /// The component output a value
    Output(u8),
}

/// Define how a component should be colorized
#[derive(Debug, Clone, Copy, PartialEq, Default, Deserialize)]
pub struct ColorSet {
    /// OFF color (0x0)
    #[serde(deserialize_with = "deserialize_color")]
    off: Color,

    /// ON low power color (0x1)
    #[serde(deserialize_with = "deserialize_color", rename = "low")]
    on_low: Color,

    /// ON high power color (0xFFFF)
    #[serde(deserialize_with = "deserialize_color", rename = "high")]
    on_high: Color,
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
fn deserialize_color<'de, D>(deserializer: D) -> Result<Color, D::Error>
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
fn color_from_hex(hex: &str) -> Result<Color, ColorError> {
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
        Ok(Color::new(
            convert_channel(r),
            convert_channel(g),
            convert_channel(b),
            convert_channel(a),
        ))
    } else {
        Err(ColorError::UnexpectedPattern(hex.to_string()))
    }
}

/// Convert a RGBA byte into a RGBA float
#[inline]
fn convert_channel(byte: u8) -> f32 {
    // remap values in the range [0, 255] to values in the range [0.0, 1.0]
    const F: f32 = 1.0 / u8::MAX as f32;

    // Convert the byte
    (byte as f32) * F
}
