use super::SelectableItem;
use egui::{Button, Response, Ui};

/// A tool to select
pub enum ToolItem {
    /// draw in a canvas
    Draw {
        brush_size: usize,
        brush_shape: BrushShape,
    },

    /// fill an area in a canvas
    Fill,

    /// pick a color from a canvas
    Picker,

    /// select a box area in a canvas
    SelectBox,

    /// select an island of elements based on color
    SelectIsland,

    /// draw a path given two points
    Path { path_type: PathType },
}

/// brush shape to use with the draw tool
pub enum BrushShape {
    Square,
    Round,
}

/// path type to generate
pub enum PathType {
    Manhattan,
    Straight,
    Curve,
}

impl ToolItem {
    /// Get a label for the tool
    fn label(&self) -> &str {
        match self {
            Self::Draw {
                brush_size: _,
                brush_shape: _,
            } => "draw",
            Self::Fill => "fill",
            Self::Picker => "picker",
            Self::SelectBox => "select box",
            Self::SelectIsland => "select continuous",
            Self::Path { path_type: _ } => "path",
        }
    }
}

impl SelectableItem for ToolItem {
    /// Show the item as selectable or selected
    fn show(&self, selected: bool, ui: &mut Ui) -> Response {
        ui.add_enabled(true, Button::new(self.label()).selected(selected))
    }
}
