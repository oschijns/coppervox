use coppervox_config::mapping::{WasmBuffer, WasmModule as WasmModuleConfig};
use std::path::Path;
use wasmtime::{Engine, Error, Instance, Memory, MemoryAccessError, Module, Store, TypedFunc};

/// An instantiated WASM module bound to the simulator.
pub struct WasmModule {
    /// WASM engine
    _engine: Engine,

    /// WASM data
    store: Store<()>,

    /// WASM module `init` function
    init: TypedFunc<(), ()>,

    /// WASM module `tick` function
    tick: TypedFunc<(), ()>,

    /// WASM module dedicated memory
    memory: Memory,

    /// Input buffer binding
    input: WasmBuffer,

    /// Ouput buffer binding
    output: WasmBuffer,
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

        let Some(memory) = instance.get_memory(&mut store, "memory") else {
            return Err(WasmModuleError::MissingMemory);
        };
        Self::validate_buffer(&memory, &store, "input", &config.input)?;
        Self::validate_buffer(&memory, &store, "output", &config.output)?;

        Ok(Self {
            _engine: engine,
            store,
            init,
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

    /// Call the module's `init` function once.
    #[inline]
    pub fn init(&mut self) -> Result<(), Error> {
        self.init.call(&mut self.store, ())
    }

    /// Call the module's `tick` function once.
    #[inline]
    pub fn tick(&mut self) -> Result<(), Error> {
        self.tick.call(&mut self.store, ())
    }

    /// Copy bytes into the configured input buffer.
    #[inline]
    pub fn write_input(&mut self, data: &[u8]) -> Result<(), MemoryAccessError> {
        self.memory.write(&mut self.store, self.input.address, data)
    }

    /// Copy bytes from the configured output buffer into the provided slice.
    /// Said slice must have the required size.
    #[inline]
    pub fn read_output(&self, data: &mut [u8]) -> Result<(), MemoryAccessError> {
        self.memory.read(&self.store, self.output.address, data)
    }

    /// Get the size of the input buffer
    #[inline]
    pub fn input_buffer_size(&self) -> usize {
        self.input.size
    }

    /// Get the size of the output buffer
    #[inline]
    pub fn output_buffer_size(&self) -> usize {
        self.output.size
    }

    /// Check the buffer targeted is within the boundaries of the WASM module
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
