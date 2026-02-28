use super::Loader;
use crate::{
    loader::mesher::QuadDataReader,
    shader_const::{MAX_CNX, Op},
};
use coppervox_analyzer::{
    Analysis, IslandInfo, Label, Mesher,
    block_mesh::{QuadCoordinateConfig, RIGHT_HANDED_Y_UP_CONFIG},
    petgraph::{Direction, matrix_graph::NodeIndex, visit::IntoNodeIdentifiers},
};
use coppervox_common::Voxel;
use coppervox_config::component::{CompDef, Interface};
use kiss3d::procedural::RenderMesh;
use std::sync::Arc;

/// Error reporting when accessing information
#[derive(thiserror::Error, Debug, Clone, Copy)]
pub enum Error {
    #[error("{0}")]
    Analysis(#[from] coppervox_analyzer::Error),

    #[error("No configuration defined for the voxel type {0}")]
    NoConfig(Voxel),

    #[error("Node 0x{0:04x} in the incidence matrix is not mapped")]
    NodeIndexUnmapped(usize),

    #[error("Too many connections defined on a component")]
    TooManyConnections,
}

/// Define an ordering for the components
struct Ordering {
    /// Define the order of the components
    #[allow(clippy::vec_box)]
    order: Vec<Box<Component>>,

    /// For a given node index, get its position in the ordered list
    indexes: Vec<usize>,

    /// Range of the components used as inputs
    range_inputs: [usize; 2],

    /// Range of the components used as outputs
    range_outputs: [usize; 2],
}

/// Group all the relevant data of a component
#[derive(Clone)]
struct Component {
    /// Identifier of the component
    label: Label,

    /// Corresponding config
    definition: CompDef,

    /// Voxel information
    info: IslandInfo,

    /// Get the index of the component in the incidence matrix
    node_index: NodeIndex,

    /// The mesh to render
    mesh: Arc<RenderMesh>,
}

/// Generated buffers at the end of the processing
pub struct Buffers {
    /// Store connections between components
    connections: Vec<u32>,

    /// Store operations and the number of connections
    operations: Vec<u32>,

    /// Store the initial values for the inputs
    inits: Vec<u32>,
}

impl Loader {
    /// Build a logic circuit from the loaded model data
    pub fn build(&self) -> Result<Buffers, Error> {
        const COUNT: usize = MAX_CNX as usize;

        // Config for generating meshes from quad data
        const QUAD_CONFIG: QuadCoordinateConfig = RIGHT_HANDED_Y_UP_CONFIG;
        let mut mesher = Mesher::new(QUAD_CONFIG, 0);
        let mut reader = QuadDataReader::new(true, false);

        // Analyse the model
        let analysis = Analysis::build(&self.model, &mut mesher, &mut reader);
        let order = self.get_order(&analysis)?;

        // Allocate buffers to store the data
        let count = analysis.count();
        let mut connections = vec![0; count * COUNT];
        let mut operations = vec![0; count];
        let mut inits = vec![0; count];

        // Resolve the ordered position of the nodes from the incidence matrix.
        let incidence = analysis.connectivity().incidence_matrix();

        // For each node in the matrix
        for node1 in incidence.node_identifiers() {
            let index1 = order.indexes[node1.index()];
            let start = index1 * COUNT;

            // Find all neighbors going toward the node
            let mut counter = 0usize;
            for (i, node2) in incidence
                .neighbors_directed(node1, Direction::Incoming)
                .enumerate()
            {
                let index2 = order.indexes[node2.index()];
                connections[start + i] = index2 as u32;
                counter += 1;
            }

            // check that we did not overshoot the maximum number of connections allowed
            if counter > COUNT {
                return Err(Error::TooManyConnections);
            }

            // Get the rest of the informations for this component
            let comp = &order.order[index1].definition.comp_type;
            operations[index1] = Op::from_type(comp) as u32 | counter as u32;
            inits[index1] = comp.start_value();
        }

        Ok(Buffers {
            connections,
            operations,
            inits,
        })
    }

    /// Define an ordering for the components.
    /// This will allow to optimize buffer copies between the CPU and the GPU.
    fn get_order(&self, analysis: &Analysis<Arc<RenderMesh>>) -> Result<Ordering, Error> {
        // How many component do we have in the simulation?
        let count = analysis.count();
        let shape_set = analysis.shape_set();
        let connectivity = analysis.connectivity();

        // We want to organize components into three groups
        let mut group_nones = Vec::with_capacity(count);
        let mut group_inputs = Vec::with_capacity(count);
        let mut group_outputs = Vec::with_capacity(count);

        // For a given node in the incidence matrix,
        // find its position in the ordered list.
        let mut indexes = Vec::with_capacity(count);
        indexes.resize(count, usize::MAX);

        // Iterate over the components
        for (i, (&label, &info)) in analysis.labeled_model().iter().enumerate() {
            // Identify the type of the component
            let voxel = info.voxel_value();
            let Some(&def) = self.config.0.get(&voxel) else {
                return Err(Error::NoConfig(voxel));
            };

            let mesh = shape_set.get_mesh(label)?;
            let node_index = connectivity.get_node_index(label)?;

            // Store the position of the node in the ordered list
            indexes[node_index.index()] = i;

            // Deduce to which group the component should be added
            let group = match def.comp_type.interface() {
                Interface::None => &mut group_nones,
                Interface::Input => &mut group_inputs,
                Interface::Output => &mut group_outputs,
            };

            // Get most data related to the component
            group.push(Box::new(Component {
                label,
                definition: def,
                info,
                node_index,
                mesh: mesh.clone(),
            }));
        }

        // Check that the indexes list is completely field
        if let Some(index) = indexes.iter().position(|&i| i == usize::MAX) {
            return Err(Error::NodeIndexUnmapped(index));
        }

        // We are putting the groups in the following order:
        // - Inputs
        // - Outputs
        // - Internals

        // Compute the ranges for the interfaces
        let count_in = group_inputs.len();
        let count_out = group_outputs.len();
        let range_inputs = [0usize, count_in];
        let range_outputs = [count_in, count_in + count_out];

        // Compute the final list
        let mut order = group_inputs;
        order.append(&mut group_outputs);
        order.append(&mut group_nones);

        // Return the result
        Ok(Ordering {
            order,
            indexes,
            range_inputs,
            range_outputs,
        })
    }
}
