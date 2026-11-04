use coppervox_config::mapping::{WasmBuffer, WasmModule as WasmModuleConfig};
use std::path::Path;
use wasmtime::{Engine, Instance, Memory, Module, Store, TypedFunc};

/// An instantiated WASM module bound to the simulator.
///
/// The module must export `init` and `tick` functions with the signature
/// `() -> ()`. When either buffer is configured, it must also export linear
/// memory as `memory`; buffer addresses are byte offsets into that memory.
pub struct WasmModule {
    _engine: Engine,
    store: Store<()>,
    tick: TypedFunc<(), ()>,
    memory: Option<Memory>,
    input: Option<WasmBuffer>,
    output: Option<WasmBuffer>,
}

/// An error encountered while loading or using a WASM module.
#[derive(Debug, thiserror::Error)]
pub enum WasmModuleError {
    /// Wasmtime failed to load, instantiate, or execute the module.
    #[error("WASM runtime error: {0}")]
    Wasmtime(#[from] wasmtime::Error),

    /// Wasmtime failed to access a linear-memory buffer.
    #[error("WASM memory access error: {0}")]
    MemoryAccess(#[from] wasmtime::MemoryAccessError),

    /// The module did not export its linear memory.
    #[error("WASM module does not export memory as \"memory\"")]
    MissingMemory,

    /// A configured buffer does not fit in the module's linear memory.
    #[error(
        "WASM {buffer} buffer at address {address} with size {size} exceeds memory size {memory_size}"
    )]
    BufferOutOfBounds {
        buffer: &'static str,
        address: usize,
        size: usize,
        memory_size: usize,
    },

    /// An input or output buffer was requested but not configured.
    #[error("WASM {0} buffer is not configured")]
    BufferNotConfigured(&'static str),

    /// The provided input does not match the fixed configured buffer size.
    #[error("WASM input has size {actual}, expected {expected}")]
    InputSizeMismatch { expected: usize, actual: usize },
}

impl WasmModule {
    /// Load and instantiate the configured module, then call its init function.
    ///
    /// Relative module paths are resolved from the process working directory.
    pub fn load(config: &WasmModuleConfig) -> Result<Self, WasmModuleError> {
        let engine = Engine::default();
        let module = Module::from_file(&engine, &config.path)?;
        let mut store = Store::new(&engine, ());
        let instance = Instance::new(&mut store, &module, &[])?;

        let init = instance.get_typed_func::<(), ()>(&mut store, &config.init)?;
        let tick = instance.get_typed_func::<(), ()>(&mut store, &config.tick)?;

        let memory = instance.get_memory(&mut store, "memory");
        if (config.input.is_some() || config.output.is_some()) && memory.is_none() {
            return Err(WasmModuleError::MissingMemory);
        }

        if let Some(memory) = memory {
            if let Some(input) = &config.input {
                Self::validate_buffer(&memory, &store, "input", input)?;
            }
            if let Some(output) = &config.output {
                Self::validate_buffer(&memory, &store, "output", output)?;
            }
        }

        init.call(&mut store, ())?;

        Ok(Self {
            _engine: engine,
            store,
            tick,
            memory,
            input: config.input.clone(),
            output: config.output.clone(),
        })
    }

    /// Load and instantiate the configured module using an explicit module path.
    ///
    /// This is useful when resolving the config's relative module path against
    /// the directory containing the config file.
    pub fn load_from_path(
        config: &WasmModuleConfig,
        path: impl AsRef<Path>,
    ) -> Result<Self, WasmModuleError> {
        let mut resolved_config = config.clone();
        resolved_config.path = path.as_ref().to_path_buf();
        Self::load(&resolved_config)
    }

    /// Call the module's tick function once.
    pub fn tick(&mut self) -> Result<(), WasmModuleError> {
        self.tick.call(&mut self.store, ())?;
        Ok(())
    }

    /// Copy bytes into the configured input buffer.
    pub fn write_input(&mut self, data: &[u8]) -> Result<(), WasmModuleError> {
        let input = self
            .input
            .as_ref()
            .ok_or(WasmModuleError::BufferNotConfigured("input"))?;
        if data.len() != input.size {
            return Err(WasmModuleError::InputSizeMismatch {
                expected: input.size,
                actual: data.len(),
            });
        }
        let memory = self.memory.as_ref().ok_or(WasmModuleError::MissingMemory)?;
        memory.write(&mut self.store, input.address, data)?;
        Ok(())
    }

    /// Copy bytes from the configured output buffer.
    pub fn read_output(&self) -> Result<Vec<u8>, WasmModuleError> {
        let output = self
            .output
            .as_ref()
            .ok_or(WasmModuleError::BufferNotConfigured("output"))?;
        let memory = self.memory.as_ref().ok_or(WasmModuleError::MissingMemory)?;
        let mut data = vec![0; output.size];
        memory.read(&self.store, output.address, &mut data)?;
        Ok(data)
    }

    fn validate_buffer(
        memory: &Memory,
        store: &Store<()>,
        name: &'static str,
        buffer: &WasmBuffer,
    ) -> Result<(), WasmModuleError> {
        let memory_size = memory.data_size(store);
        let end = buffer.address.checked_add(buffer.size);
        if end.is_none_or(|end| end > memory_size) {
            return Err(WasmModuleError::BufferOutOfBounds {
                buffer: name,
                address: buffer.address,
                size: buffer.size,
                memory_size,
            });
        }
        Ok(())
    }
}
