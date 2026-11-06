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

The crate's linker configuration reserves bytes `1024..1036` for the
simulator's I/O buffers; Rust's static data starts at byte `2048`. Configure
the module in a CopperVox RON file with:

```ron
wasm_module: (
    path: "examples/wasm-adder/target/wasm32-unknown-unknown/release/coppervox_wasm_adder.wasm",
    input: (address: 1024, size: 8),
    output: (address: 1032, size: 4),
),
```

Map two circuit components whose values should be added to `WASM_OUTPUT(0)`
and `WASM_OUTPUT(1)`. Map the component that should receive the sum to
`WASM_INPUT(0)`. The RON `input` buffer receives values from
`WASM_OUTPUT` components; the `output` buffer supplies values to
`WASM_INPUT` components.
