use super::{Error, uni_mesher::UniQuadDataReader};
use crate::{loader::uni_mesher::MeshIndices, shader_const::MAX_CNX};
use coppervox_analyzer::{
    Analysis, Connectivity, IslandInfo, Label, Mesher, ShapeSet,
    block_mesh::{QuadCoordinateConfig, RIGHT_HANDED_Y_UP_CONFIG},
    petgraph::Direction,
};
use coppervox_common::{
    List, Model, Voxel,
    gridmap::{gridmap::bounding_box::BoundingBox, hashbrown::HashMap},
};
use coppervox_config::{
    color::GradientIndex,
    component::{CompType, Interface},
    identify::VoxelIdentifier,
};
use std::{rc::Rc, slice::Iter};

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

/// Circuit data to be used by the process shader
pub struct ProcessingData {
    /// Components identified by their voxel value
    ident: ComponentIdentifier,

    /// Meshes with corresponding components
    mesh_order: MeshOrdering,

    /// Components ordered by their IO type
    io_order: IoOrdering,

    /// Connections between components
    connect: Connect,

    /// Quad mesher storing the vertices data
    quad_data: UniQuadDataReader,

    /// Bounding box aroudn the whole circuit
    bounding_box: BoundingBox<3>,
}

impl ProcessingData {
    /// Get identified components
    #[inline]
    pub fn ident(&self) -> &ComponentIdentifier {
        &self.ident
    }

    /// Get the meshes with corresponding components
    #[inline]
    pub fn meshes(&self) -> &MeshOrdering {
        &self.mesh_order
    }

    /// Get the components ordered by their IO type
    #[inline]
    pub fn io_order(&self) -> &IoOrdering {
        &self.io_order
    }

    /// Get the connections between components
    #[inline]
    pub fn connect(&self) -> &Connect {
        &self.connect
    }

    /// Access the quad reader which contains the raw vertex data that was collected
    #[inline]
    pub fn quad_data(&self) -> &UniQuadDataReader {
        &self.quad_data
    }

    /// Access the bounding box around the whole circuit
    #[inline]
    pub fn bounding_box(&self) -> &BoundingBox<3> {
        &self.bounding_box
    }

    /// Center of the circuit
    pub fn center(&self) -> [isize; 3] {
        let [ax, ay, az] = self.bounding_box.start;
        let [bx, by, bz] = self.bounding_box.end;
        [bx - ax, by - ay, bz - az]
    }

    /// Process the data loaded to obtain meshes and buffers
    pub fn make(model: &Model, ident: &VoxelIdentifier) -> Result<Self, Error> {
        // First analyze the model to identify shapes and connectivity
        let mut quad_reader = UniQuadDataReader::new();
        let analysis = analyze(model, &mut quad_reader);

        // Collect component types
        let types = ident
            .definitions
            .iter()
            .map(|(c, _)| c.clone())
            .collect::<List<_>>();

        // Then convert the analysis into data usable by the GPU
        let ident = ComponentIdentifier::make(types, analysis.labeled_model().islands())?;
        let mesh_order = MeshOrdering::make(&ident, analysis.shape_set())?;
        let io_order = IoOrdering::make(&ident);
        let connect = Connect::make(&io_order, analysis.connectivity())?;

        Ok(Self {
            ident,
            mesh_order,
            io_order,
            connect,
            quad_data: quad_reader,
            bounding_box: *analysis.bounding_box(),
        })
    }
}

/// Build an analysis of the model, using the given mesher and reader
fn analyze(model: &Model, reader: &mut UniQuadDataReader) -> Analysis<Rc<MeshIndices>> {
    // Config for generating meshes from quad data
    const QUAD_CONFIG: QuadCoordinateConfig = RIGHT_HANDED_Y_UP_CONFIG;
    let mut mesher = Mesher::new(QUAD_CONFIG, 0);

    // Analyse the model
    Analysis::build(model, &mut mesher, reader)
}

/// Identifies the type and color of a component based on its label instead of its voxel value
pub struct ComponentIdentifier {
    /// For each label, provides the component definition (type and color)
    components: HashMap<Label, (Voxel, BoundingBox<3>)>,

    /// List of component definition identified by their voxel index
    types: List<CompType>,
}

impl ComponentIdentifier {
    /// Returns the component definition for the given label, if one exists
    #[inline]
    pub fn get(&self, label: Label) -> Option<(Voxel, BoundingBox<3>)> {
        self.components.get(&label).cloned()
    }

    /// For each island, find the corresponding component definition in the config
    fn make(types: List<CompType>, islands: &HashMap<Label, IslandInfo>) -> Result<Self, Error> {
        let mut components = HashMap::with_capacity(islands.len());

        // Iterate over each island data
        for (&label, info) in islands.iter() {
            let voxel = info.voxel_value();
            components.insert(label, (voxel, *info.bounding_box()));
        }

        Ok(Self { components, types })
    }
}

/// Ordering of components by the mesh they share
pub struct MeshOrdering {
    // TODO: with the unified approach, we have only one Mesh to deal with
    meshes: Vec<Rc<MeshIndices>>,

    /// List of components and their associated mesh indices
    /// Each component is identified by its label.
    /// The mesh index is the index into the `meshes` vector.
    /// The color index is the index into the `colors` vector.
    components: Vec<(Label, MeshIndex, GradientIndex, BoundingBox<3>)>,
}

/// Index of the mesh to use for a given component
pub type MeshIndex = usize;

impl MeshOrdering {
    /// Returns a slice of the unique meshes extracted from the shape set.
    #[inline]
    pub fn meshes(&self) -> &[Rc<MeshIndices>] {
        &self.meshes
    }

    /// Returns a slice of the components and their associated mesh indices.
    #[inline]
    pub fn components(&self) -> &[(Label, MeshIndex, GradientIndex, BoundingBox<3>)] {
        &self.components
    }

    /// Create an iterator over the components and their meshes
    #[inline]
    pub fn iter_components<'m>(&'m self) -> IterComp<'m> {
        IterComp {
            components: self.components.iter(),
            meshes: &self.meshes,
        }
    }

    /// Order components by the mesh they share
    fn make(
        ident: &ComponentIdentifier,
        shape_set: &ShapeSet<Rc<MeshIndices>>,
    ) -> Result<Self, Error> {
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
            // TODO: rework this part
            meshes.push(mesh.clone());

            // Find all labels associated with this mesh
            shape_set.find_labels(hash, &mut labels);
            for &label in labels.iter() {
                let Some((voxel, bbox)) = ident.components.get(&label) else {
                    return Err(Error::NoConfigLabel(label));
                };

                components.push((label, index, *voxel as usize, *bbox));
            }
        }

        // return the ordering
        Ok(Self { meshes, components })
    }
}

/// Iterator over the components and their meshes
pub struct IterComp<'m> {
    /// Iterator over components data
    components: Iter<'m, (Label, MeshIndex, GradientIndex, BoundingBox<3>)>,

    /// List of meshes
    meshes: &'m [Rc<MeshIndices>],
}

impl<'m> Iterator for IterComp<'m> {
    type Item = (Label, Rc<MeshIndices>, GradientIndex, BoundingBox<3>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((label, mesh_index, color_index, bbox)) = self.components.next() {
            Some((
                *label,
                self.meshes[*mesh_index].clone(),
                *color_index,
                *bbox,
            ))
        } else {
            None
        }
    }
}

/// Ordering of the components based on whenever they are inputs or outputs
pub struct IoOrdering {
    /// List of components ordered by their IO type
    /// Each component is identified by its label.
    /// And we added extra definition such as the component operator and the color index.
    components: Vec<(Label, CompType)>,

    /// Given a label, find the index of the component in the ordered list
    /// This is used to generate the `Connect` struct.
    label_index: HashMap<Label, usize>,

    /// Range of inputs
    /// Should be in the range `[0, len(in)[`.
    range_inputs: [usize; 2],

    /// Range of outputs
    /// Should be in the range `[len(in), len(in) + len(out)[`.
    range_outputs: [usize; 2],
}

impl IoOrdering {
    /// Number of components in the ordering
    pub fn count(&self) -> usize {
        self.components.len()
    }

    /// Returns a slice of the components ordered by their IO type.
    #[inline]
    pub fn components(&self) -> &[(Label, CompType)] {
        &self.components
    }

    /// Returns the range of inputs.
    #[inline]
    pub fn range_inputs(&self) -> [usize; 2] {
        self.range_inputs
    }

    /// Returns the range of outputs.
    #[inline]
    pub fn range_outputs(&self) -> [usize; 2] {
        self.range_outputs
    }

    /// Order components by their IO connectivity
    fn make(ident: &ComponentIdentifier) -> Self {
        let count = ident.components.len();

        // we define multiple arrays to store the components by their IO type
        let mut lists = [
            Vec::with_capacity(count), // none
            Vec::with_capacity(count), // input
            Vec::with_capacity(count), // output
            Vec::with_capacity(count), // both
        ];

        // Iterate over each island data
        for (&label, (voxel, _)) in ident.components.iter() {
            let comp_type = ident.types[*voxel as usize].clone();
            let interface = comp_type.interface();

            // Get the list for this interface type
            let list = &mut lists[interface as usize];
            list.push((label, comp_type));
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
            components,
            label_index,
            range_inputs,
            range_outputs,
        }
    }
}

/// The maximum number of connections a component can have
pub const CNX_COUNT: usize = MAX_CNX as usize;

/// Define how components are connected to each other
pub struct Connect {
    /// For each component, we store at most `MAX_CNX` connections
    /// Components are ordered the same way as `IoOrdering`
    /// First value is the number of connections defined
    /// Second value is an array of connection indices, padded with `usize::MAX`
    buffer: Vec<(usize, [usize; CNX_COUNT])>,
}

impl Connect {
    /// Returns a slice of the buffer.
    #[inline]
    pub fn buffer(&self) -> &[(usize, [usize; CNX_COUNT])] {
        &self.buffer
    }

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
            let mut connections = [usize::MAX; CNX_COUNT];

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
