#![no_std]

/// Buffer with a fixed size
pub type Buffer<const SIZE: usize> = [u32; SIZE];

/// Define a binding for a buffer by storing the buffer
/// address and the buffer size in a single 64-bits word.
pub type Bind = u64;

/// Mask to ensure address and size don't overlap
const MASK: u64 = u32::MAX as u64;

/// Generate a binding for the given static buffer
#[inline]
pub fn bind<B>(buffer: *const B) -> Bind {
    let addr = (buffer as u64) & MASK;
    let size = core::mem::size_of::<B>() as u64;
    (size << 32) | addr
}

/// Generate a binding for the given static buffer
#[inline]
pub fn decode_bind(bind: Bind) -> [usize; 2] {
    let addr = (bind & MASK) as usize;
    let size = (bind >> 32) as usize;
    [addr, size]
}
