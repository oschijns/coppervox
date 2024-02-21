use egui::{Grid, Response, Ui};

pub mod palette;
pub mod toolbox;

/// An item to select
pub trait SelectableItem {
    /// Show the item as selectable or selected
    fn show(&self, selected: bool, ui: &mut Ui) -> Response;
}

/// A panel of items to select
pub struct SelectionPanel<T> {
    /// Name of the panel
    name: String,

    /// The list of items available
    items: Vec<T>,

    /// The selected tool
    selected_index: usize,

    /// Maximum number of columns to display
    columns: usize,
}

impl<T> SelectionPanel<T> {
    /// Return the selected tool in the panel
    pub fn get_selected(&self) -> &'_ T {
        &self.items[self.selected_index]
    }
}

impl<T: SelectableItem> SelectionPanel<T> {
    /// Draw the tool panel in the UI
    pub fn show(&mut self, ui: &mut Ui) {
        // create a grid of buttons to select a tool
        Grid::new(self.name.as_str()).show(ui, |ui| {
            // iterate over the list of tools
            for (index, tool) in self.items.iter().enumerate() {
                // check if the hovered tool is already selected or not
                if tool.show(index == self.selected_index, ui).clicked() {
                    self.selected_index = index;
                }

                // at each end of a row, move on to next row
                if (index + 1) % self.columns == 0 {
                    ui.end_row();
                }
            }
        });
    }
}
