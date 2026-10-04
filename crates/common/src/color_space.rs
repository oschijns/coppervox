use crate::{EMPTY_VOXEL, Voxel};
use rgb::{RGB8, Rgb};
use std::{
    collections::HashMap,
    hash::{BuildHasherDefault, Hash, Hasher},
    sync::{Arc, RwLock},
};

/// Point in 3D color space
pub type Point = [i32; 3];

/// Define a partitioning of the color space given a finite set of colors.
#[derive(Debug, Clone)]
pub struct ColorSpace {
    /// Read-only part of the palette
    read: Arc<InnerRead>,

    /// Writable part of the palette
    write: Arc<RwLock<InnerWrite>>,
}

/// Storage for the processed palette.
#[derive(Debug)]
struct InnerRead {
    /// Set of unique colors extracted from the palette
    colorset: Vec<(ColorHash, Voxel, Point)>,
}

/// Storage for the processed palette.
#[derive(Debug)]
struct InnerWrite {
    /// Store already identified mapping between RGB colors and corresponding index.
    /// This map is initially filled with colors from the palette but may be completed
    /// overtime if we encounter colors that are close but not perfect match.
    lookup: HashMap<ColorHash, Voxel, BuildHasherDefault<ColorHasher>>,
}

/// Default threshold for discarding colors based on their alpha channel.
/// Any value below this threshold is treated as transparent.
pub const ALPHA_THRESHOLD: u8 = 128;

impl ColorSpace {
    /// Create a color space from the given list of colors
    pub fn load(color_list: &[(RGB8, Voxel)]) -> Self {
        let (read, write) = load_indexed_colors(color_list);
        Self {
            read: Arc::new(read),
            write: Arc::new(RwLock::new(write)),
        }
    }
}

/// Given a list of colors with their corresponding index,
/// return structures for indexing the color space.
fn load_indexed_colors(color_list: &[(RGB8, Voxel)]) -> (InnerRead, InnerWrite) {
    // Assign a unique index to each color encountered by converting the color set into a list
    let mut colorset = Vec::with_capacity(color_list.len());
    for (color, index) in color_list.iter() {
        colorset.push((ColorHash::from(*color), *index, to_point(*color)));
    }

    // Initialize the lookup table and fill it with color indexes
    let mut lookup =
        HashMap::with_capacity_and_hasher(colorset.len(), BuildHasherDefault::<ColorHasher>::new());
    for (hash, index, _) in colorset.iter() {
        lookup.insert(*hash, *index);
    }

    (InnerRead { colorset }, InnerWrite { lookup })
}

/// Error encountered when trying to lock a shared resource
#[derive(thiserror::Error, Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockError {
    #[error("Failed to lock the shared map for reading")]
    ReadLock,

    #[error("Failed to lock the shared map for writing")]
    WriteLock,
}

impl ColorSpace {
    /// Given an color, identify the indexes of the color of each pixel.
    /// If the color is not a perfect match return the index of the closest color
    /// in the set and cache the new mapping for faster lookup on the next iteration.
    pub fn identify_color_index(&self, color: RGB8) -> Result<Voxel, LockError> {
        // Check if the color has already been identified before
        let hash = ColorHash::from(color);

        // Lock the lookup map for reading
        let Ok(write) = self.write.read() else {
            return Err(LockError::ReadLock);
        };

        // Check if an index has already be identified for this color
        if let Some(index) = write.lookup.get(&hash) {
            Ok(*index)
        } else {
            // Drop the lock on the lookup map while we search
            // for the index of the closest color.
            drop(write);

            // Otherwise look for an index based on the proximity
            // to the colors in the initial colorset.
            let p1 = to_point(color);

            // Find the index of the closest color in the color set.
            let mut selected = (u32::MAX, EMPTY_VOXEL);
            for &(_, index, p0) in self.read.colorset.iter() {
                let sqr_dist = squared_distance(p0, p1);

                // If the squared distance is smaller than
                // the current minimum, update the selection.
                if sqr_dist < selected.0 {
                    selected = (sqr_dist, index);
                }
            }

            // Lock the lookup map in writing to insert the new entry
            let Ok(mut write) = self.write.write() else {
                return Err(LockError::WriteLock);
            };
            write.lookup.insert(hash, selected.1);

            // Return the index found
            Ok(selected.1)
        }
    }

    /// Get the index of the provided color.
    pub fn get_color_index(&self, color: RGB8) -> Option<Voxel> {
        // Check if the color has already been identified before
        let hash = ColorHash::from(color);

        // Lock the lookup map for reading
        let Ok(write) = self.write.read() else {
            return None;
        };

        // Check if an index has already be identified for this color
        if let Some(index) = write.lookup.get(&hash) {
            Some(*index)
        } else {
            None
        }
    }
}

/// Compute the squared distance between two points in 3D space
#[inline]
const fn squared_distance(p0: Point, p1: Point) -> u32 {
    let [x0, y0, z0] = p0;
    let [x1, y1, z1] = p1;

    #[inline]
    const fn pow2(x: i32) -> u32 {
        (x * x) as u32
    }

    // 255^2 * 3 is below the maximum value of 2^32 so overflow is not possible
    pow2(x0 - x1) + pow2(y0 - y1) + pow2(z0 - z1)
}

/// Convert a RGB value into a array of signed integers
#[inline]
const fn to_point(color: RGB8) -> Point {
    let Rgb { r, g, b } = color;
    [r as i32, g as i32, b as i32]
}

/// Wrapper to quickly hash the RGB color
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ColorHash(u32);

/// Hasher for colors
#[derive(Debug, Default, Clone, Copy)]
struct ColorHasher(u32);

impl From<RGB8> for ColorHash {
    fn from(color: RGB8) -> Self {
        let Rgb { r, g, b } = color;
        Self(u32::from_ne_bytes([r, g, b, 0x00]))
    }
}

impl Hash for ColorHash {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u32(self.0);
    }
}

impl Hasher for ColorHasher {
    #[inline]
    fn finish(&self) -> u64 {
        self.0 as u64
    }

    #[inline]
    fn write_u32(&mut self, i: u32) {
        self.0 = i;
    }

    /// Should not be used
    fn write(&mut self, _: &[u8]) {
        panic!("Invalid use")
    }
}
