# coppervox voxels library

This library handles all voxels processings.
This includes:
- importing voxels models from various formats
- drawing new models made of voxels
- applying standard transformations to voxels models
- combining voxels models into a single one
- generating meshes from voxels models
- analyzing voxels models to construct a logic circuits made of bevy's entities and components


## Components

This library is centered around the `VoxModel` type.
This type allows to store a large quantity of voxels without any predifined dimensions.
The model is capable of growing or shrinking dynamically as voxels are added or removed.
It is based on a `HashMap` containing chunks of type `ndarray::Array3`.

The `VoxModel` type offers various methods to iterate over the voxels stored in it.
The iterators include a `FilterIter` attribute to select which voxel to iterate over and which to skip.

Multiple models can be combined together in a tree data structure.
This allows for designing components and sub-components which can be reused multiple times inside of the same projects.


## Acknowledgements

This library is adapted from the [bevy_vox_mesh crate](https://crates.io/crates/bevy_vox_mesh) by Lucas A.
