#![no_std]

use core::panic::PanicInfo;

/// Input buffer is 8 bytes long, containing two u32 values.
const INPUT_ADDRESS: usize = 1024;

/// Output buffer is 4 bytes long, containing one u32 value.
const OUTPUT_ADDRESS: usize = 1032;

#[unsafe(no_mangle)]
pub extern "C" fn init() -> u32 {
    unsafe {
        core::ptr::write_volatile(OUTPUT_ADDRESS as *mut u32, 0);
    }
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn tick() -> u32 {
    let left = unsafe { core::ptr::read_volatile(INPUT_ADDRESS as *const u32) };
    let right = unsafe { core::ptr::read_volatile((INPUT_ADDRESS + 4) as *const u32) };
    let sum = left.wrapping_add(right);
    unsafe {
        core::ptr::write_volatile(OUTPUT_ADDRESS as *mut u32, sum);
    }
    0
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
