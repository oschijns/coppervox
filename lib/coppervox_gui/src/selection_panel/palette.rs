use super::SelectableItem;
use egui::{Button, Color32, Response, Ui};

/// A color in a palette
pub struct PaletteItem {
    /// The name of the color
    name: String,

    /// Color
    color: Color32,
}

impl SelectableItem for PaletteItem {
    /// Show the item as selectable or selected
    fn show(&self, selected: bool, ui: &mut Ui) -> Response {
        ui.add_enabled(
            true,
            Button::new(self.name.as_str())
                .fill(self.color)
                .selected(selected),
        )
    }
}
