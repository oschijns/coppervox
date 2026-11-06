# CopperVox

Build logic circuits from voxel models and run them on the GPU.

CopperVox imports a voxel model, maps voxel colors or indices to component
types, detects neighboring components, and displays the running circuit. The
simulation updates at the configured tick rate. `INPUT_KEY` components respond
to keyboard input while the window is focused.

## Requirements

- Rust and Cargo, using the toolchain required by the workspace.
- A graphics environment that supports the project's `wgpu` renderer.

## Run

Pass a model and a **RON** or **TOML** configuration file:

```sh
coppervox --model <VOXEL_FILE> --config <CONFIG.ron>
```

To preview the generated mesh without the live simulation shader, add
`--preview-mesh`:

```sh
coppervox --model <VOXEL_FILE> --config <CONFIG.ron> --preview-mesh
```

## Components' connectivity and data flow

Components are continuous blob of voxels of the same color. The type of a 
component is specified by the companion **RON** file. Connections between 
components is defined by a simple contact rule.

Given component **A** and component **B**, if three voxels of **A** surround one 
voxel of **B**, then the output value computed by **A** will be feed as an input 
value for **B** on the next simulation tick.

```text
      A
      A
    B A B   <--- A is surrounded by three B
    B B B        thus B output its value into A
      B
      B
```


### Components' connectivity analysis

CopperVox first groups face-connected voxels of the same mapped voxel value
into an island. It then examines the six face-neighbors of each voxel in each
island (left, right, up, down, front, and back). Diagonal neighbors do not
count.

If a voxel has **at least three** face-neighbors belonging to a different
island, CopperVox creates a directed connection from that neighboring island
to the voxel's island. In other words, the voxel's component receives data
from the neighboring component. For example, if three or more of a component
voxel's six face-neighbors belong to an input component, that input feeds the
component. The rule is evaluated around individual voxels; one qualifying
voxel is enough to create the connection between the two islands.

Each component can receive at most 16 connections. If it has more, circuit
processing reports an error. Logic components combine their incoming values
using their configured operation; key inputs and constants provide values to
the circuit.


## Supported file formats

Model formats are selected by the file extension:

| Extension(s)        | Format                       |
|---------------------|------------------------------|
| `.gox`              | Goxel                        |
| `.vox`              | MagicaVoxel                  |
| `.qb`               | Qubicle Binary               |
| `.qbt`              | Qubicle Binary Tree          |
| `.qbcl`             | Qubicle Construction Library |
| `.vmax`             | Voxel Max                    |
| `.voxj`, `.voxjz`   | Voxel JSON                   |
| `.xcf`              | GIMP                         |
| `.ase`, `.aseprite` | Aseprite                     |

Voxel model files need colors for color-based mapping. Imported RGB colors are
assigned to the closest configured color; alpha values below 128 are treated
as transparent. Voxel scene transforms must align with the integer voxel grid:
fractional translations or scales and non-axis-aligned rotations are rejected.

Configuration files can be `.ron` or `.toml`. The example below uses RON and
color-based mapping.

## Configure color-to-component mapping

The top-level RON value is `Config`, with:

- `tick_rate`: simulation updates per second; must be greater than zero.
- `mapping`: how imported voxels are identified and which component definition
  to use.
- `color_gradients`: optional shared palette of component display gradients.

For color mapping, set `voxel_ident: COLOR` and list entries in `components`.
Each key is an RGB hex color (`"#RRGGBB"`); each value has a `type` and a
`gradient`. The color is a palette match target, so nearby colors in the model
map to the closest listed color.

```ron
Config(
    tick_rate: 60,
    mapping: (
        voxel_ident: COLOR,
        components: {
            // Logic components
            "#fff1e8": (
                type: OR,
                gradient: (off: "#111111", low: "#222222", high: "#ffffff"),
            ),
            "#ff004d": (
                type: NAND,
                gradient: 0,
            ),

            // A key-controlled input. Press Q to send this value.
            "#FFEC27": (
                type: INPUT_KEY(key: Q, value: 0xFFFFFFFF),
                gradient: 0,
            ),
        },
    ),
    color_gradients: [
        (off: "#110000", low: "#220000", high: "#ff004d"),
    ],
)
```

The example's first component has an inline gradient. The `NAND` and
`INPUT_KEY` entries use gradient index `0`, which refers to the first element
of `color_gradients`. Each gradient has three RGB hex colors:

- `off`: display color when the component value is zero.
- `low`: display color for a low, nonzero value.
- `high`: display color for the maximum value.

Gradients may be defined inline for each component or shared by index. If all
gradients are inline, `color_gradients` can be omitted.

### ~~Mapping by voxel index~~

**This feature is not yet supported!**

If a model format provides stable voxel indices rather than useful colors, use
`voxel_ident: INDEX` and use numeric voxel indices as keys:

```ron
Config(
    tick_rate: 60,
    mapping: (
        voxel_ident: INDEX,
        components: {
            1: (type: OR, gradient: 0),
            2: (type: NAND, gradient: 0),
        },
    ),
    color_gradients: [
        (off: "#111111", low: "#555555", high: "#ffffff"),
    ],
)
```

Every non-empty voxel in the model must have a matching component mapping.

## Component types

Since all components can have from zero up to sixteen inputs, available 
operations are necessarely associative and commutative.

Component type names in RON are uppercase:

| Type           | Meaning                            |
|----------------|------------------------------------|
| `CONST(value)` | Start with the specified 32-bit unsigned constant value. |
| `OR`           | Bitwise OR of incoming values.     |
| `AND`          | Bitwise AND of incoming values.    |
| `NOR`          | Bitwise NOR of incoming values.    |
| `NAND`         | Bitwise NAND of incoming values.   |
| `ADD`          | Add incoming unsigned values.      |
| `MUL`          | Multiply incoming unsigned values. |
| `MIN`          | Return the minimum incoming value. |
| `MAX`          | Return the maximum incoming value. |
| `INPUT_KEY(key: ..., value: ...)` | Send `value` while the specified keyboard key is held. |
| `INPUT_MASK(...)` | A 32-bit key-mask input variant in the configuration type. Keyboard binding is currently wired for `INPUT_KEY`; `INPUT_MASK` is not yet connected to the app's key handling. |
| `OUTPUT(index)` | Mark a component as an output interface. |

Values are unsigned 32-bit integers. For example, `0xFFFFFFFF` is the maximum
value. `INPUT_KEY` uses a key name from Kiss3D's key enum, such as `Q`, `W`, or
`E`. `CONST` and `OUTPUT` take a positional argument, for example
`CONST(0x00000001)` and `OUTPUT(0)`.

## WASM modules

The simulator can call a WASM module between simulation ticks. Modules export
linear memory as `memory` and functions `init() -> u32` and `tick() -> u32`;
zero indicates success. `WASM_OUTPUT(slot)` components send circuit values to
the module's configured input buffer, while `WASM_INPUT(slot)` components
receive values from its output buffer.

A standalone adder example, including build instructions and its buffer
configuration, is in [`examples/wasm-adder/README.md`](./examples/wasm-adder/README.md).

