use crate::import::import_model;
use coppervox_runner::{
    config::{component::CompType, identify::VoxelIdentifier, manifest::Manifest},
    loader::{
        converter::{Processing, RenderingData},
        processor::ProcessingData,
        wasm_module::WasmModule,
    },
    runner::{InputBinding, KeyboardState, Runner, RunnerData, WasmBinding},
    shader_const::{GpuColorGradient, GpuMat4x4},
};
use kiss3d::{
    glamx::Vec3,
    procedural::{IndexBuffer, RenderMesh},
    resource::Material3d,
};
use std::{cell::RefCell, collections::HashSet, num::NonZeroU32, rc::Rc};

/// Identify matrix to prepare the projection matrix for the camera
const IDENTITY_MATRIX: GpuMat4x4 = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// Circuit data to construct
pub struct Construct {
    /// Mesh depicting the circuit
    pub render_mesh: RenderMesh,

    /// Circuit mesh with component translations baked in for standard-material previews
    pub preview_mesh: RenderMesh,

    /// Material based on a Runner
    pub material: Rc<RefCell<Box<dyn Material3d + 'static>>>,

    /// Center of the model
    pub center: Vec3,

    /// Camera position relative to the centered model
    pub camera_offset: Vec3,

    /// Keys currently held down by the user
    pub pressed_keys: KeyboardState,
}

impl Construct {
    /// Process the data to construct the circuit
    pub fn new(config: &Manifest) -> Result<Self, anyhow::Error> {
        // Load the configuration provided
        let tick_rate = NonZeroU32::new(config.tick_rate)
            .ok_or_else(|| anyhow::anyhow!("Simulation tick_rate must be greater than zero"))?;

        // Check if we should load a WASM module too
        let wasm_module = if let Some(wasm) = &config.wasm_module {
            let module = WasmModule::load(&config.path, wasm)?;
            Some(module)
        } else {
            None
        };

        // Generate an identifier for voxels
        let vox_ident = VoxelIdentifier::make(config);

        // Load the voxel data
        const CHUNK_DIM: [usize; 3] = [12, 12, 12];
        let model = import_model(&config.model_path(), &vox_ident, CHUNK_DIM)?;

        // Process the data to build the logic circuit and the rendered mesh
        let ordered = ProcessingData::make(&model, &vox_ident)?;
        let process = Processing::make(ordered.io_order(), ordered.connect());

        // Find IO connectors in the circuit
        let count = ordered.count();
        let mut input_bindings = Vec::with_capacity(count);
        let mut wasm_inputs = Vec::with_capacity(count);
        let mut wasm_outputs = Vec::with_capacity(count);

        for (component, (i, comp_type)) in ordered.io_order().components().iter().enumerate() {
            match comp_type {
                CompType::InputKey { key, value } => {
                    input_bindings.push(InputBinding::new(component, *key, *value));
                }
                CompType::WasmInput(slot) => {
                    wasm_inputs.push(WasmBinding::new(component, *slot));
                }
                CompType::WasmOutput(slot) => {
                    wasm_outputs.push(WasmBinding::new(component, *slot));
                }
                _ => {}
            }
        }

        match &wasm_module {
            Some(module) => {
                println!("WASM MODULE:\n{}", module);
                for binding in &wasm_inputs {
                    println!("input slot: {}", binding.slot);
                }
                for binding in &wasm_outputs {
                    println!("output slot: {}", binding.slot);
                }

                for binding in &wasm_inputs {
                    module.validate_input_slot(binding.slot)?;
                }
                for binding in &wasm_outputs {
                    module.validate_output_slot(binding.slot)?;
                }
            }
            None if !wasm_inputs.is_empty() || !wasm_outputs.is_empty() => {
                anyhow::bail!("WASM components are configured but no WASM module is loaded");
            }
            None => {}
        }
        let pressed_keys: KeyboardState = Rc::new(RefCell::new(HashSet::new()));

        let color_presets = vox_ident
            .definitions
            .iter()
            .map(|(_, g)| GpuColorGradient::from(*g))
            .collect::<Box<[_]>>();
        let render = RenderingData::make(
            ordered.quad_data(),
            ordered.meshes(),
            ordered.io_order(),
            color_presets,
        );
        if render.instances_data.is_empty() {
            anyhow::bail!("The imported model contains no renderable components");
        }
        let translations = ordered
            .meshes()
            .iter_components()
            .map(|(_, _, _, bounds)| {
                let [x, y, z] = bounds.start;
                Vec3::new(x as f32, y as f32, z as f32)
            })
            .collect::<Vec<_>>();
        let preview_mesh = make_preview_mesh(&render.render_mesh, &translations)?;

        // Center on the actual bounds, which may not start at the origin.
        let bounds = ordered.bounding_box();
        let [min_x, min_y, min_z] = bounds.start;
        let [max_x, max_y, max_z] = bounds.end;
        let center = Vec3::new(
            ((min_x as f64 + max_x as f64) * 0.5) as f32,
            ((min_y as f64 + max_y as f64) * 0.5) as f32,
            ((min_z as f64 + max_z as f64) * 0.5) as f32,
        );

        let size_x = (max_x as f64 - min_x as f64).max(1.0);
        let size_y = (max_y as f64 - min_y as f64).max(1.0);
        let size_z = (max_z as f64 - min_z as f64).max(1.0);
        let camera_distance =
            ((size_x * size_x + size_y * size_y + size_z * size_z).sqrt() * 1.75).max(4.0) as f32;
        let camera_offset = Vec3::splat(camera_distance / 3.0_f32.sqrt());

        // Group runner data
        let data = RunnerData::new(
            IDENTITY_MATRIX,
            process.connections.into_boxed_slice(),
            process.operations.into_boxed_slice(),
            render.instances_data,
            render.color_presets,
            process.initial_values.into_boxed_slice(),
        );

        let runner = Runner::new(
            data,
            tick_rate,
            input_bindings.into_boxed_slice(),
            Rc::clone(&pressed_keys),
            wasm_module,
            wasm_inputs.into_boxed_slice(),
            wasm_outputs.into_boxed_slice(),
        );

        Ok(Self {
            render_mesh: render.render_mesh,
            preview_mesh,
            material: Rc::new(RefCell::new(
                Box::new(runner) as Box<dyn Material3d + 'static>
            )),
            center,
            camera_offset,
            pressed_keys,
        })
    }
}

fn make_preview_mesh(
    mesh: &RenderMesh,
    translations: &[Vec3],
) -> Result<RenderMesh, anyhow::Error> {
    let IndexBuffer::Split(faces) = &mesh.indices else {
        anyhow::bail!("Expected a split index buffer for the circuit mesh");
    };
    let uvs = mesh
        .uvs
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Circuit mesh is missing component index data"))?;

    let mut coords = Vec::with_capacity(faces.len() * 3);
    let mut indices = Vec::with_capacity(faces.len());
    for face in faces {
        let mut triangle = [0; 3];
        for (corner_index, [vertex_index, _, component_index]) in face.iter().enumerate() {
            let translation = translations
                .get(*component_index as usize)
                .ok_or_else(|| anyhow::anyhow!("Circuit mesh has an invalid component index"))?;
            if *vertex_index as usize >= mesh.coords.len() || *component_index as usize >= uvs.len()
            {
                anyhow::bail!("Circuit mesh has an invalid vertex or UV index");
            }

            triangle[corner_index] = coords.len() as u32;
            coords.push(mesh.coords[*vertex_index as usize] + *translation);
        }
        indices.push(triangle);
    }

    if indices.is_empty() {
        anyhow::bail!(
            "The generated circuit mesh contains no triangles ({} source vertices, {} source triangles, {} component translations)",
            mesh.coords.len(),
            mesh.num_triangles(),
            translations.len()
        );
    }

    let mut preview = RenderMesh::new(coords, None, None, Some(IndexBuffer::Unified(indices)));
    preview.recompute_normals();
    Ok(preview)
}

#[cfg(test)]
mod tests {
    use super::make_preview_mesh;
    use kiss3d::{
        glamx::{Vec2, Vec3},
        procedural::{IndexBuffer, RenderMesh},
    };

    #[test]
    fn preview_mesh_applies_instance_translations() {
        let mesh = RenderMesh::new(
            vec![Vec3::ZERO, Vec3::X, Vec3::Y],
            Some(vec![Vec3::Z]),
            Some(vec![Vec2::ZERO]),
            Some(IndexBuffer::Split(vec![[[0, 0, 0], [1, 0, 0], [2, 0, 0]]])),
        );

        let preview = make_preview_mesh(&mesh, &[Vec3::new(10.0, 20.0, 30.0)]).unwrap();

        assert_eq!(
            preview.coords,
            vec![
                Vec3::new(10.0, 20.0, 30.0),
                Vec3::new(11.0, 20.0, 30.0),
                Vec3::new(10.0, 21.0, 30.0),
            ]
        );
        assert_eq!(preview.num_triangles(), 1);
    }
}
