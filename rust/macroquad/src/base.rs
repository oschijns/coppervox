/// Hardcoded shader config
pub mod shader;

/// Identify inputs
pub mod input;

/// Identify colors
pub mod color;

use macroquad::{color::Color, math::Vec3};
use serde::Deserialize;

/// Value type processed in the components
pub type Value = u32;

/// Initial value
pub const INIT: Value = 0;

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum CompType {
    /// constant value
    Const(Value) = 0,

    /// bitwise OR operation
    Or = 1,

    /// bitwise AND operation
    And = 2,

    /// bitwise NOR operation
    Nor = 3,

    /// bitwise NAND operation
    Nand = 4,

    /// arithmetic addition
    Add = 5,

    /// arithmetic multiplication
    Mul = 6,

    /// Minimum
    Min = 7,

    /// Maximum
    Max = 8,

    /// Input value
    Input { input: input::Input, value: Value },
}

/// Define how a component should be colorized
#[derive(Debug, Default)]
pub struct Colorize {
    /// OFF color (0x0)
    off: Color,

    /// ON low power color (0x1)
    on_low: Color,

    /// ON high power color (0xFFFF)
    on_high: Color,

    /// Describe how to draw a gradient
    gradient: [Vec3; 2],
}

/// Default to a component that returns zero
impl Default for CompType {
    #[inline]
    fn default() -> Self {
        Self::Const(INIT)
    }
}

impl Colorize {
    /// Build a config from values
    #[inline]
    pub fn new(off: Color, on_low: Color, on_high: Color) -> Self {
        Self {
            off,
            on_low,
            on_high,
            gradient: [color_to_vec3(on_low), color_to_vec3(on_high)],
        }
    }

    /// Return a color corresponding to the given value
    pub fn get_color(&self, value: Value) -> Color {
        if value == 0 {
            self.off
        } else if value == 1 {
            self.on_low
        } else if value > u16::MAX as u32 {
            self.on_high
        } else {
            let t = (value & 0xFFFF) as f32 / u16::MAX as f32;
            let inter = self.gradient[0].lerp(self.gradient[1], t);
            vec3_to_color(inter)
        }
    }
}

#[inline]
fn color_to_vec3(color: Color) -> Vec3 {
    Vec3::new(color.r, color.g, color.b)
}

#[inline]
fn vec3_to_color(vec3: Vec3) -> Color {
    Color::new(vec3.x, vec3.y, vec3.z, 1.0)
}
