use super::{Error, mesher::QuadDataReader};
use crate::shader_const::MAX_CNX;
use coppervox_analyzer::{
    Analysis, Connectivity, IslandInfo, Label, Mesher, ShapeSet,
    block_mesh::{QuadCoordinateConfig, RIGHT_HANDED_Y_UP_CONFIG},
    petgraph::Direction,
};
use coppervox_common::Model;
use coppervox_config::{
    Config,
    color::ColorSet,
    component::{CompDef, Interface},
};
use kiss3d::procedural::RenderMesh;
use std::{collections::HashMap, rc::Rc};

/*
 * When processing the analysis, we want to define two ordering of the components.
 * On one hand, we want to order the components by inputs, outputs and internals.
 * On the other hand, we want to order the components by their meshes.
 *
 * To obtain the buffers, we need first to define a list of unique meshes.
 * For each mesh, we want the list of components that use it.
 * On each component we'll store an index to the process buffers.
 *
 * Then we generate lists for the process logic.
 * For each component on the logical part,
 * we want to find the corresponding component on the render side.
 *
 */

/// Reference-counted mesh
type RcMesh = Rc<RenderMesh>;

/// Build an analysis of the model, using the given mesher and reader
fn analyze(model: &Model) -> Analysis<RcMesh> {
    // Config for generating meshes from quad data
    const QUAD_CONFIG: QuadCoordinateConfig = RIGHT_HANDED_Y_UP_CONFIG;
    let mut mesher = Mesher::new(QUAD_CONFIG, 0);
    let mut reader = QuadDataReader::new(true, false);

    // Analyse the model
    Analysis::build(model, &mut mesher, &mut reader)
}

/// Ordering of components by the mesh they share
struct MeshOrdering {
    /// List of unique meshes extracted from the shape set
    meshes: Vec<RcMesh>,

    /// List of components and their associated mesh indices
    components: Vec<(usize, Label)>,
}

impl MeshOrdering {
    /// Order components by the mesh they share
    fn make(shape_set: &ShapeSet<RcMesh>) -> Self {
        // How many objects and how many shapes/meshes are defined.
        let count_obj = shape_set.count_objects();
        let count_shp = shape_set.count_shapes();

        // storage for the meshes extracted and the components using them
        let mut meshes = Vec::with_capacity(count_shp);
        let mut components = Vec::with_capacity(count_obj);

        // temporary storage for the labels associated with each mesh
        let mut labels = Vec::with_capacity(count_obj);

        // Find all labels associated with each mesh
        for (index, (&hash, mesh)) in shape_set.iter_hashes_meshes().enumerate() {
            meshes.push(mesh.clone());

            // Find all labels associated with this mesh
            shape_set.find_labels(hash, &mut labels);
            for &label in labels.iter() {
                components.push((index, label));
            }
        }

        // return the ordering
        Self { meshes, components }
    }
}

/// Ordering of the components based on whenever they are inputs or outputs
struct IoOrdering {
    /// List of color sets
    colors: Vec<ColorSet>,

    /// List of components ordered by their IO type
    components: Vec<(Label, CompDef)>,

    /// Given a label, find the index of the component in the ordered list
    label_index: HashMap<Label, usize>,

    /// Range of inputs
    range_inputs: [usize; 2],

    /// Range of outputs
    range_outputs: [usize; 2],
}

impl IoOrdering {
    /// Order components by their IO connectivity
    fn make(config: &Config, islands: &HashMap<Label, IslandInfo>) -> Self {
        let count = islands.len();

        // we define multiple arrays to store the components by their IO type
        let mut lists = [
            Vec::with_capacity(count), // none
            Vec::with_capacity(count), // input
            Vec::with_capacity(count), // output
            Vec::with_capacity(count), // both
        ];

        // Iterate over each island data
        for (&label, info) in islands.iter() {
            let voxel = info.voxel_value();
            if let Some(def) = config.map.get(&voxel) {
                let interface = def.comp_type.interface();

                // Get the list for this interface type
                let list = &mut lists[interface as usize];
                list.push((label, *def));
            }
        }

        // Regroup the generated list in the following order: [ inputs, both, outputs, none ]
        // This allows faster buffers exchanges between the CPU and the GPU
        macro_rules! get {
            ($type:ident) => {
                lists[Interface::$type as usize].as_slice()
            };
        }
        let inputs = get![Input];
        let boths = get![Both];
        let outputs = get![Output];
        let nones = get![None];

        // Get the size of each array
        let li = inputs.len();
        let lb = boths.len();
        let lo = outputs.len();
        let ln = nones.len();

        // Compute the ranges of inputs and outputs
        let range_inputs = [0, li + lb];
        let range_outputs = [li, li + lb + lo];

        // Create a single list of components where they are ordered by their IO type
        let mut components = Vec::with_capacity(li + lb + lo + ln);
        components.extend_from_slice(inputs);
        components.extend_from_slice(boths);
        components.extend_from_slice(outputs);
        components.extend_from_slice(nones);

        // Find the index of each component in the ordered list
        let mut label_index = HashMap::with_capacity(components.len());
        for (i, (label, _)) in components.iter().enumerate() {
            label_index.insert(*label, i);
        }

        // Store the data found in a struct
        Self {
            colors: config.colors.clone(),
            components,
            label_index,
            range_inputs,
            range_outputs,
        }
    }
}

/// The maximum number of connections a component can have
const CNX_COUNT: usize = MAX_CNX as usize;

/// Define how components are connected to each other
struct Connect {
    /// For each component, we store at most `MAX_CNX` connections
    /// Components are ordered the same way as `IoOrdering`
    buffer: Vec<(usize, [usize; CNX_COUNT])>,
}

impl Connect {
    /// Construct a `Connect` buffer from the given `IoOrdering` and `Connectivity`
    fn make(io_order: &IoOrdering, connectivity: &Connectivity) -> Result<Self, Error> {
        // Buffer to compose
        let mut buffer = Vec::with_capacity(io_order.components.len());

        // Resolve the ordered position of the nodes from the incidence matrix.
        let incidence = connectivity.incidence_matrix();

        // Iterate over the components
        for (label1, _) in io_order.components.iter() {
            let node1 = connectivity.get_node_index(*label1)?;

            // Fixed size buffer to store the indexes of the components connected to this node
            let mut connections = [0usize; CNX_COUNT];

            // Find all neighbors going toward the node
            let mut count = 0usize;
            for (i, node2) in incidence
                .neighbors_directed(node1, Direction::Incoming)
                .enumerate()
            {
                // If a node has more than `MAX_CNX` connections, return an error
                if i >= CNX_COUNT {
                    return Err(Error::TooManyConnections);
                }

                // Get the label of the neighbor node
                let label2 = incidence.node_weight(node2);
                let Some(&index2) = io_order.label_index.get(label2) else {
                    return Err(Error::NodeIndexUnmapped(node2.index()));
                };

                // Store the index of the connected component
                connections[i] = index2;
                count += 1;
            }

            // Store the connections for this node
            buffer.push((count, connections));
        }

        Ok(Self { buffer })
    }
}
