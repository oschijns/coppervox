/// Find the number of contact points between blobs of voxels
pub mod connectivity;

/// Represent an island of similar voxels
pub mod island;

/// Identify blob of similar voxel and assign a label to them
pub mod labeling;

/// Generate a mesh from a blob of voxels
pub mod mesh;

/// Identify a pattern from a blob of voxels
pub mod pattern;

use super::{Cell, array3, model::VoxModel};
use bevy::{
    asset::{Assets, Handle},
    ecs::system::EntityCommands,
    pbr::{Material, MeshMaterial3d},
    prelude::{Commands, Entity, Mesh, Mesh3d, ResMut, Transform},
};
use connectivity::connectivity;
use labeling::{Label, LabeledModel};
use mesh::{BlockMeshMatrix, BoolVoxel};
use pattern::PatternId;
use petgraph::{
    Undirected,
    csr::{Csr, NodeIndex},
    visit::{EdgeRef, IntoEdgeReferences, IntoNodeReferences},
};
use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

/// Define the elements of the circuit
pub struct CircuitData<T, W> {
    /// The elements of the circuit
    circuit: Csr<CircuitElement<T, W>, bool, Undirected>,
}

/// Define an element of the circuit
pub struct CircuitElement<T, W> {
    /// The type of the element
    element_type: ElementType<T, W>,

    /// Number of voxels composing this element
    voxel_count: usize,

    /// Transform of the element
    transform: Transform,

    /// The mesh used to render the element
    mesh_handle: Handle<Mesh>,
}

impl<T: Copy + Hash + Eq, W: Clone + From<usize>> CircuitData<T, W> {
    /// Analyze the voxel model to prepare a circuit
    /// that can be used to generate bevy entities.
    pub fn generate<
        C: Converter<V, T, W>,
        V: Cell + Default + Copy + Into<T> + Hash + Eq,
        const D: usize,
    >(
        mut meshes: ResMut<Assets<Mesh>>,
        model: &VoxModel<V>,
        converter: &C,
        threshold: usize,
    ) -> Self {
        let labeled = LabeledModel::from_model(model);

        // Generate the meshes that will be used by bevy
        let mut mesh_map =
            HashMap::<PatternId, Handle<Mesh>>::with_capacity(labeled.patterns.len());
        for (id, pattern) in labeled.patterns {
            let block_mesh = BlockMeshMatrix::<BoolVoxel>::from_matrix(&pattern);
            mesh_map.insert(
                id,
                meshes.add(block_mesh.build_mesh(
                    [0, 0, 0],
                    array3![for N in {
                        id.dimensions[N]
                    } -> u32],
                )),
            );
        }

        // convert the labeled model into a 3D matrix
        let matrix = labeled.labeled_model.to_matrix();

        // generate the connectivity graph from the matrix
        let labels = HashSet::from_iter(labeled.islands.keys().copied());
        let graph = connectivity(&matrix, &labels);

        // generate the circuit data
        let mut circuit = Csr::<CircuitElement<T, W>, bool, Undirected>::new();
        let mut node_map = HashMap::<Label, NodeIndex>::with_capacity(labeled.islands.len());
        for (&label, island) in &labeled.islands {
            let pos = island.bounding_box.start;
            let index = circuit.add_node(CircuitElement {
                element_type: converter.convert(&island.value),
                voxel_count: island.count,
                transform: Transform::from_xyz(pos[0] as f32, pos[1] as f32, pos[2] as f32),
                mesh_handle: mesh_map[&island.pattern_id].clone(),
            });
            node_map.insert(label, index);
        }

        // iterate over every contacts in the graph
        for (index1, index2, &edge) in graph.edge_references() {
            // connection can only be established between two elements
            // if one is a wire and the other is a component
            let label1 = graph.node_weight(index1);
            let label2 = graph.node_weight(index2);
            if label1 != label2 {
                // access the circuit element from the label
                macro_rules! label_to_element {
                    ($label:ident) => {
                        if let Some(&node_index) = node_map.get($label) {
                            let element = &circuit[node_index];
                            match element.element_type {
                                ElementType::Neither => {
                                    continue;
                                }
                                _ => (node_index, element),
                            }
                        } else {
                            continue;
                        }
                    };
                }
                let (index1, element1) = label_to_element![label1];
                let (index2, element2) = label_to_element![label2];

                // identify the elements: wire and component
                let (index_wire, index_comp) = match element1.element_type {
                    ElementType::Component(_) => {
                        if let ElementType::Wire(_) = element2.element_type {
                            (index2, index1)
                        } else {
                            continue;
                        }
                    }
                    ElementType::Wire(_) => {
                        if let ElementType::Component(_) = element2.element_type {
                            (index1, index2)
                        } else {
                            continue;
                        }
                    }
                    _ => {
                        continue;
                    }
                };

                // check the number of contact points between the wire and
                // the component to know if the wire is an input or an output
                circuit.add_edge(index_comp, index_wire, edge >= threshold);
            }
        }
        Self { circuit }
    }
}

/// Convert a voxel value into either a component type or a wire channel
pub trait Converter<V, T, W> {
    /// Take a voxel value as input and return the corresponding element
    fn convert(&self, voxel: &V) -> ElementType<T, W>;
}

// TODO : drop distinction between components and wires
// they are the same thing now
// simply use usize to identify elements

/// The types of elements that are part of a circuit
pub enum ElementType<T, W> {
    /// An actual component (logic gate, bus, etc..)
    Component(T),

    /// A wire with a given channel
    Wire(W),

    /// Not an element
    Neither,
}

impl<T, W> CircuitData<T, W> {
    /// Generate the bevy entities
    pub fn build<B: Builder<T, W, M>, M: Material>(&self, mut cmd: Commands, builder: &B) {
        // FIRST: create the entities for the wires
        let mut wire_map = HashMap::<NodeIndex, Entity>::with_capacity(self.circuit.node_count());
        for (index, element) in self.circuit.node_references() {
            // only create entities for wires at first
            if let ElementType::Wire(channel) = &element.element_type {
                // spawn an entity for the wire
                let mut cmd = cmd.spawn((
                    Mesh3d(element.mesh_handle.clone()),
                    MeshMaterial3d(builder.get_material(&element.element_type)),
                    element.transform,
                ));

                // call the builder to add the wire behavior
                builder.set_wire(&mut cmd, channel);

                // keep track of the wire generated for the other components
                wire_map.insert(index, cmd.id());
            }
        }

        // SECOND: create entities for the components
        for (index, element) in self.circuit.node_references() {
            // only create entities for components then
            if let ElementType::Component(comp_type) = &element.element_type {
                // spawn an entity for the component
                let mut cmd = cmd.spawn((
                    Mesh3d(element.mesh_handle.clone()),
                    MeshMaterial3d(builder.get_material(&element.element_type)),
                    element.transform,
                ));

                // look for the input and the output wires of the component
                let edges = self.circuit.edges(index);
                let size = edges.size_hint().0;
                let mut inputs = Vec::with_capacity(size);
                let mut outputs = Vec::with_capacity(size);

                for edge in edges {
                    if let Some(&wire_entity) = wire_map.get(&edge.target()) {
                        if *edge.weight() {
                            outputs.push(wire_entity);
                        } else {
                            inputs.push(wire_entity);
                        }
                    }
                }

                // call the builder to add the component behavior
                builder.set_component(&mut cmd, comp_type, element.voxel_count, &inputs, &outputs);
            }
        }
    }
}

/// Add the necessary components to bevy entities
pub trait Builder<T, W, M: Material> {
    /// Get the material to apply to the entity
    fn get_material(&self, entity_type: &ElementType<T, W>) -> Handle<M>;

    /// Set the entity to behave as a wire
    fn set_wire(&self, cmd: &mut EntityCommands, channel: &W);

    /// Set the entity to behave as a component
    fn set_component(
        &self,
        cmd: &mut EntityCommands,
        comp_type: &T,
        voxel_count: usize,
        inputs: &[Entity],
        outputs: &[Entity],
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_label() {}
}
