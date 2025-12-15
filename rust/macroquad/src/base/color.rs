use std::{num::ParseIntError, sync::LazyLock};

use macroquad::color::{BLACK, Color};
use regex::Regex;
use serde::{Deserialize, de::Visitor};

/// Define a color that can be deserialized from a config file
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorCode(pub Color);

/// Error when trying to read a color code
#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("Could not identify a color tag")]
    NoTag,

    #[error("Could not parse hexadecimal value {0}")]
    InvalidHex(#[from] ParseIntError),
}

/// Default color
impl Default for ColorCode {
    #[inline]
    fn default() -> Self {
        Self(BLACK)
    }
}

impl<'de> Deserialize<'de> for ColorCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        /// Simple visitor to match key code identifiers with name
        struct __Visitor;

        impl<'de> Visitor<'de> for __Visitor {
            type Value = Color;

            /// Specify which value is expected
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("Expecting a color code")
            }

            /// Read a string
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                identify_color(v).map_err(|e| E::custom(e.to_string()))
            }
        }

        // Read key code
        let keycode = deserializer.deserialize_str(__Visitor)?;
        Ok(ColorCode(keycode))
    }
}

/// Regex for reading hexadecimal values
static RE_HEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"#[[:xdigit:]]{6}"#).unwrap());

/// Regex for reading a RGB value
static RE_RGB: LazyLock<Regex> = LazyLock::new(|| {
    let float = r#"\s*([+-]?([0-9]*[.])?[0-9]+)\s*"#;
    let expr = format!(r#"rgb\({},{},{}\)"#, float, float, float);
    Regex::new(expr.as_str()).unwrap()
});

/// Read the color code to deduce the color
pub fn identify_color(code: &str) -> Result<Color, Error> {
    if let Some(cap) = RE_HEX.captures(code) {
        let code = cap.get(0).unwrap().as_str().parse::<u32>()?;
        Ok(Color::from_hex(code))
    } else {
        Err(Error::NoTag)
    }
}
