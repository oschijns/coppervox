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
`bind_input` and `bind_output` functions. CopperVox calls these functions to
discover the actual locations and sizes chosen by the linker, so no memory
addresses need to be hard-coded.

Configure the module in the manifest:

```ron
wasm_module: Some((path: "../examples/wasm-adder/target/wasm32-unknown-unknown/release/coppervox_wasm_adder.wasm")),
```

Map two circuit components whose values should be added to `WASM_INPUT(0)` and `WASM_INPUT(1)`.
Map the component that should receive the sum to `WASM_OUTPUT(0)`.
`WASM_INPUT` sends component values to the module's input buffer, 
while `WASM_OUTPUT` supplies the module's output values to components.

WASM functions can return multiple values when the WebAssembly multi-value feature is enabled, 
but a tuple is not a portable `extern "C"` return type for this interface.
The example instead exports separate `() -> u32` getters for each address and size.
