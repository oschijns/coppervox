# Roadmap

Config file is now converted into `VoxelIdentifier` structure.
That structure is to be used to identify voxels by index or by color.
It also enforce a coupling component type - color gradient.
It means that a component label should give a Voxel index which would give the
component type and the color gradient at once.

Rework to do in `main/src/construct.rs`.
We want to introduce `List<T>` which is an alias for `Box<[T]>` this will help distinguish between mutable vectors and completed lists.

We want to rename and properly document a lot of structs.
`VoxelIdentifier` is not yet to process the model, it should be used in the `importer` crate.
