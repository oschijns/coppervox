use coppervox_bind::{Bind, decode_bind};
use coppervox_config::manifest::WasmModule as WasmModuleConfig;
use std::path::Path;
use wasmtime::{AsContextMut, Engine, Instance, Memory, Module, Store, TypedFunc};

/// Size of a cell in the buffer in bytes
const CELL_SIZE: usize = size_of::<u32>();

/// An instantiated WASM module bound to the simulator.
pub struct WasmModule {
    /// WASM engine
    #[allow(unused)]
    engine: Engine,

    /// WASM data
    store: Store<()>,

    /// WASM module `init` function
    init: TypedFunc<(), u32>,

    /// WASM module `tick` function
    tick: TypedFunc<(), u32>,

    /// WASM module dedicated memory
    memory: Memory,

    /// Input buffer binding
    input: WasmIoBuffer,

    /// Ouput buffer binding
    output: WasmIoBuffer,
}

/// Specify the start address and a buffer of 32-bits
/// cells to write or read from the WASM module.
pub struct WasmIoBuffer {
    /// Address in the WASM module
    address: usize,

    /// Data to read or write
    buffer: Vec<u32>,
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
        "WASM {name} buffer at address {address} with size {size} exceeds memory size {memory_size}"
    )]
    BufferOutOfBounds {
        name: String,
        address: usize,
        size: usize,
        memory_size: usize,
    },
}

/// An error encountered while running the module
#[derive(Debug, thiserror::Error)]
pub enum WasmRunError {
    /// Wasmtime failed to access a linear-memory buffer.
    #[error("WASM memory access error: {0}")]
    MemoryAccess(#[from] wasmtime::MemoryAccessError),

    /// Wasmtime failed to call init or tick functions
    #[error("WASM memory access error: {0}")]
    Call(#[from] wasmtime::Error),

    /// A configured component index does not fit in the corresponding buffer.
    #[error("WASM {name} buffer has no 32-bit value at slot {slot} (size {size} bytes)")]
    BufferSlotOutOfBounds {
        name: &'static str,
        slot: usize,
        size: usize,
    },

    /// The WASM function returned a nonzero application-level status code.
    #[error("WASM {function} returned error code {code}")]
    StatusCode { function: &'static str, code: u32 },
}

impl WasmRunError {
    #[inline]
    pub fn out_of_bounds(name: &'static str, slot: usize, size: usize) -> Self {
        Self::BufferSlotOutOfBounds { name, slot, size }
    }
}

impl WasmModule {
    /// Load and instantiate the configured module.
    ///
    /// Relative module paths are resolved from the process working directory.
    pub fn load(start_path: &Path, config: &WasmModuleConfig) -> Result<Self, WasmModuleError> {
        let engine = Engine::default();
        let module = Module::from_file(&engine, start_path.join(&config.path))?;
        let mut store = Store::new(&engine, ());
        let instance = Instance::new(&mut store, &module, &[])?;

        // Access the functions to run the WASM module
        let init = instance.get_typed_func::<(), u32>(&mut store, &config.init)?;
        let tick = instance.get_typed_func::<(), u32>(&mut store, &config.tick)?;

        // Prepare memory
        let Some(memory) = instance.get_memory(&mut store, "memory") else {
            return Err(WasmModuleError::MissingMemory);
        };
        let memory_size = memory.data_size(&store);
        let buffer_i = Self::prepare_buffer(&instance, &mut store, memory_size, &config.input)?;
        let buffer_o = Self::prepare_buffer(&instance, &mut store, memory_size, &config.output)?;

        Ok(Self {
            engine,
            store,
            init,
            tick,
            memory,
            input: buffer_i,
            output: buffer_o,
        })
    }

    /// Check the buffer targeted is within the boundaries of the WASM module
    fn prepare_buffer(
        instance: &Instance,
        mut store: impl AsContextMut,
        memory_size: usize,
        func_name: &str,
    ) -> Result<WasmIoBuffer, WasmModuleError> {
        let func = instance.get_typed_func::<(), Bind>(&mut store, func_name)?;
        let bind = func.call(&mut store, ())?;
        let [address, size] = decode_bind(bind);

        let end = address.checked_add(size);
        if end.is_none_or(|end| end > memory_size) {
            return Err(WasmModuleError::BufferOutOfBounds {
                name: func_name.to_string(),
                address,
                size,
                memory_size,
            });
        }
        Ok(WasmIoBuffer {
            address,
            buffer: vec![0; size / CELL_SIZE],
        })
    }

    /// Call the module's `init` function once and check its status code.
    #[inline]
    pub fn init(&mut self) -> Result<(), WasmRunError> {
        let code = self.init.call(&mut self.store, ())?;
        Self::check_status("init", code)
    }

    /// Call the module's `tick` function once and check its status code.
    #[inline]
    pub fn tick(&mut self) -> Result<(), WasmRunError> {
        let code = self.tick.call(&mut self.store, ())?;
        Self::check_status("tick", code)
    }

    /// Write the input buffer into the WASM module
    #[inline]
    pub fn write_input(&mut self) -> Result<(), WasmRunError> {
        self.memory.write(
            &mut self.store,
            self.input.address,
            bytemuck::cast_slice(&self.input.buffer),
        )?;
        Ok(())
    }

    /// Read the output buffer from the WASM module
    #[inline]
    pub fn read_output(&mut self) -> Result<(), WasmRunError> {
        self.memory.read(
            &self.store,
            self.output.address,
            bytemuck::cast_slice_mut(&mut self.output.buffer),
        )?;
        Ok(())
    }

    /// Write one 32-bit component value to the configured input slot.
    #[inline]
    pub fn write_input_value(&mut self, slot: usize, value: u32) -> Result<(), WasmRunError> {
        if let Some(cell) = self.input.buffer.get_mut(slot) {
            *cell = value;
            Ok(())
        } else {
            let size = self.input.buffer.len() * CELL_SIZE;
            Err(WasmRunError::out_of_bounds("input", slot, size))
        }
    }

    /// Read one 32-bit component value from the configured output slot.
    #[inline]
    pub fn read_output_value(&self, slot: usize) -> Result<u32, WasmRunError> {
        if let Some(cell) = self.output.buffer.get(slot) {
            Ok(*cell)
        } else {
            let size = self.output.buffer.len() * CELL_SIZE;
            Err(WasmRunError::out_of_bounds("output", slot, size))
        }
    }

    /// Validate that a component slot is available in the input buffer.
    pub fn validate_input_slot(&self, slot: usize) -> Result<(), WasmRunError> {
        let size = self.input.buffer.len();
        if slot < size {
            Ok(())
        } else {
            Err(WasmRunError::out_of_bounds("input", slot, size * CELL_SIZE))
        }
    }

    /// Validate that a component slot is available in the output buffer.
    pub fn validate_output_slot(&self, slot: usize) -> Result<(), WasmRunError> {
        let size = self.output.buffer.len();
        if slot < size {
            Ok(())
        } else {
            Err(WasmRunError::out_of_bounds(
                "output",
                slot,
                size * CELL_SIZE,
            ))
        }
    }

    /// Check the return code status
    #[inline]
    fn check_status(function: &'static str, code: u32) -> Result<(), WasmRunError> {
        if code == 0 {
            Ok(())
        } else {
            Err(WasmRunError::StatusCode { function, code })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{WasmModule, WasmRunError};

    #[test]
    fn zero_is_a_success_status() {
        assert!(WasmModule::check_status("tick", 0).is_ok());
    }

    #[test]
    fn nonzero_status_is_reported_with_function_and_code() {
        assert!(matches!(
            WasmModule::check_status("init", 7),
            Err(WasmRunError::StatusCode {
                function: "init",
                code: 7,
            })
        ));
    }
}
