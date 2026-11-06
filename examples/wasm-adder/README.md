# CopperVox WASM adder

This standalone crate exports the functions and memory layout expected by
CopperVox's WASM runner. Each `tick` adds two unsigned 32-bit values supplied
in input slots 0 and 1 and writes the result to output slot 0. Addition wraps
on overflow.

## Build

Install the WebAssembly target once, then build this crate independently of
the CopperVox workspace:

```sh
rustup target add wasm32-unknown-unknown
cd examples/wasm-adder
cargo build --target wasm32-unknown-unknown --release
```

The module is written to:

```text
examples/wasm-adder/target/wasm32-unknown-unknown/release/coppervox_wasm_adder.wasm
```

The input and output buffers are Rust static arrays, so the linker allocates
their storage alongside the module's other static data. The module exports
`input_buffer` and `output_buffer` functions, each returning two `u32`.
These expose the actual locations and sizes chosen by the linker.

The current CopperVox runner still expects literal integer addresses and sizes
in the RON manifest; it does not call these getter functions. To use this
module with the current runner, configure the manifest with the values returned
by the getters for the exact built module. For example:

```ron
wasm_module: (path: "examples/wasm-adder/target/wasm32-unknown-unknown/release/coppervox_wasm_adder.wasm"),
```

Map two circuit components whose values should be added to `WASM_OUTPUT(0)` and `WASM_OUTPUT(1)`. 
Map the component that should receive the sum to `WASM_INPUT(0)`.
The RON `input` buffer receives values from `WASM_OUTPUT` components; 
the `output` buffer supplies values to `WASM_INPUT` components.

WASM functions can return multiple values when the WebAssembly multi-value
feature is enabled, but a tuple is not a portable `extern "C"` return type for
this interface. The example instead exports separate `() -> u32` getters for
each address and size.
