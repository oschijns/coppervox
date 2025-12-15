use macroquad::input::KeyCode;
use serde::{Deserialize, de::Visitor};

/// Define an input that can be deserialized from a config file
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Input(pub KeyCode);

impl<'de> Deserialize<'de> for Input {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        /// Simple visitor to match key code identifiers with name
        struct __Visitor;

        impl<'de> Visitor<'de> for __Visitor {
            type Value = KeyCode;

            /// Specify which value is expected
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("Expecting a name identifying the KeyCode")
            }

            /// Read a string
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(name_to_keycode(v))
            }
        }

        // Read key code
        let keycode = deserializer.deserialize_str(__Visitor)?;
        Ok(Input(keycode))
    }
}

/// Match an input name with a corresponding key code
#[rustfmt::skip]
pub fn name_to_keycode(name: &str) -> KeyCode {
    let name = name.to_ascii_lowercase();
    match name.as_str() {
        "space"         => KeyCode::Space,
        "apostrophe"    => KeyCode::Apostrophe,
        "comma"         => KeyCode::Comma,
        "minus"         => KeyCode::Minus,
        "period"        => KeyCode::Period,
        "slash"         => KeyCode::Slash,
        "key0"          => KeyCode::Key0,
        "key1"          => KeyCode::Key1,
        "key2"          => KeyCode::Key2,
        "key3"          => KeyCode::Key3,
        "key4"          => KeyCode::Key4,
        "key5"          => KeyCode::Key5,
        "key6"          => KeyCode::Key6,
        "key7"          => KeyCode::Key7,
        "key8"          => KeyCode::Key8,
        "key9"          => KeyCode::Key9,
        "semicolon"     => KeyCode::Semicolon,
        "equal"         => KeyCode::Equal,
        "a"             => KeyCode::A,
        "b"             => KeyCode::B,
        "c"             => KeyCode::C,
        "d"             => KeyCode::D,
        "e"             => KeyCode::E,
        "f"             => KeyCode::F,
        "g"             => KeyCode::G,
        "h"             => KeyCode::H,
        "i"             => KeyCode::I,
        "j"             => KeyCode::J,
        "k"             => KeyCode::K,
        "l"             => KeyCode::L,
        "m"             => KeyCode::M,
        "n"             => KeyCode::N,
        "o"             => KeyCode::O,
        "p"             => KeyCode::P,
        "q"             => KeyCode::Q,
        "r"             => KeyCode::R,
        "s"             => KeyCode::S,
        "t"             => KeyCode::T,
        "u"             => KeyCode::U,
        "v"             => KeyCode::V,
        "w"             => KeyCode::W,
        "x"             => KeyCode::X,
        "y"             => KeyCode::Y,
        "z"             => KeyCode::Z,
        "left_bracket"  => KeyCode::LeftBracket,
        "backslash"     => KeyCode::Backslash,
        "right_bracket" => KeyCode::RightBracket,
        "grave_accent"  => KeyCode::GraveAccent,
        "world1"        => KeyCode::World1,
        "world2"        => KeyCode::World2,
        "escape"        => KeyCode::Escape,
        "enter"         => KeyCode::Enter,
        "tab"           => KeyCode::Tab,
        "backspace"     => KeyCode::Backspace,
        "insert"        => KeyCode::Insert,
        "delete"        => KeyCode::Delete,
        "right"         => KeyCode::Right,
        "left"          => KeyCode::Left,
        "down"          => KeyCode::Down,
        "up"            => KeyCode::Up,
        "page_up"       => KeyCode::PageUp,
        "page_down"     => KeyCode::PageDown,
        "home"          => KeyCode::Home,
        "end"           => KeyCode::End,
        "caps_lock"     => KeyCode::CapsLock,
        "scroll_lock"   => KeyCode::ScrollLock,
        "num_lock"      => KeyCode::NumLock,
        "print_screen"  => KeyCode::PrintScreen,
        "pause"         => KeyCode::Pause,
        "f1"            => KeyCode::F1,
        "f2"            => KeyCode::F2,
        "f3"            => KeyCode::F3,
        "f4"            => KeyCode::F4,
        "f5"            => KeyCode::F5,
        "f6"            => KeyCode::F6,
        "f7"            => KeyCode::F7,
        "f8"            => KeyCode::F8,
        "f9"            => KeyCode::F9,
        "f10"           => KeyCode::F10,
        "f11"           => KeyCode::F11,
        "f12"           => KeyCode::F12,
        "f13"           => KeyCode::F13,
        "f14"           => KeyCode::F14,
        "f15"           => KeyCode::F15,
        "f16"           => KeyCode::F16,
        "f17"           => KeyCode::F17,
        "f18"           => KeyCode::F18,
        "f19"           => KeyCode::F19,
        "f20"           => KeyCode::F20,
        "f21"           => KeyCode::F21,
        "f22"           => KeyCode::F22,
        "f23"           => KeyCode::F23,
        "f24"           => KeyCode::F24,
        "f25"           => KeyCode::F25,
        "kp_0"          => KeyCode::Kp0,
        "kp_1"          => KeyCode::Kp1,
        "kp_2"          => KeyCode::Kp2,
        "kp_3"          => KeyCode::Kp3,
        "kp_4"          => KeyCode::Kp4,
        "kp_5"          => KeyCode::Kp5,
        "kp_6"          => KeyCode::Kp6,
        "kp_7"          => KeyCode::Kp7,
        "kp_8"          => KeyCode::Kp8,
        "kp_9"          => KeyCode::Kp9,
        "kp_decimal"    => KeyCode::KpDecimal,
        "kp_divide"     => KeyCode::KpDivide,
        "kp_multiply"   => KeyCode::KpMultiply,
        "kp_subtract"   => KeyCode::KpSubtract,
        "kp_add"        => KeyCode::KpAdd,
        "kp_enter"      => KeyCode::KpEnter,
        "kp_equal"      => KeyCode::KpEqual,
        "left_shift"    => KeyCode::LeftShift,
        "left_control"  => KeyCode::LeftControl,
        "left_alt"      => KeyCode::LeftAlt,
        "left_super"    => KeyCode::LeftSuper,
        "right_shift"   => KeyCode::RightShift,
        "right_control" => KeyCode::RightControl,
        "right_alt"     => KeyCode::RightAlt,
        "right_super"   => KeyCode::RightSuper,
        "menu"          => KeyCode::Menu,
        _               => KeyCode::Unknown,
    }
}
