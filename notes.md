# Roadmap

Config file is now converted into `VoxelIdentifier` structure.
That structure is to be used to identify voxels by index or by color.
It also enforce a coupling component type - color gradient.
It means that a component label should give a Voxel index which would give the
component type and the color gradient at once.
