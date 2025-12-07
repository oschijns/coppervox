use crate::builder::gd_mesher::GdMesher;
use coppervox_analyzer::{
    Analysis, Index, Mesher, Model, Voxel, block_mesh::RIGHT_HANDED_Y_UP_CONFIG,
};
use godot::{
    classes::{ArrayMesh, GridMap},
    prelude::*,
};

/// Load a voxel model from some arbitrary source and convert it
/// into processed data that can be used to build a logic circuit.
#[derive(GodotClass)]
#[class(init, base=Resource, rename=CVModelConverter)]
pub struct ModelConverter {
    base: Base<Resource>,

    /// The voxel canvas to write to before converting it
    model: Model,

    /// Mesher
    #[init(val = Mesher::new(RIGHT_HANDED_Y_UP_CONFIG, 0))]
    mesher: Mesher,

    /// Mesh reader
    mesh_reader: GdMesher,

    /// Resulting analysis of the model
    analysis: Option<Analysis<Gd<ArrayMesh>>>,
}

#[godot_api]
impl ModelConverter {
    /// Read data from the GridMap to populate this converter
    #[func]
    fn read_gridmap(&mut self, gridmap: Gd<GridMap>) {
        // For each used cell in the gridmap
        for pos in gridmap.get_used_cells().iter_shared() {
            let item = gridmap.get_cell_item(pos);

            // Convert the index
            let index = [pos.x as Index, pos.y as Index, pos.z as Index];
            self.model[index] = item as Voxel;
        }
    }

    /// Process the internal model into data for building the logic circuit
    #[func]
    fn process_model(&mut self) {
        self.analysis = Some(Analysis::build(
            &self.model,
            &mut self.mesher,
            &mut self.mesh_reader,
        ));
    }
}

impl ModelConverter {
    /// Get the analysis of the model if one was produced
    #[inline]
    pub fn analysis(&self) -> Option<&Analysis<Gd<ArrayMesh>>> {
        self.analysis.as_ref()
    }
}
