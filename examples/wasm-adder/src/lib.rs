#![no_std]

use coppervox_bind::{Bind, Buffer, bind};
use core::panic::PanicInfo;
use core::ptr::addr_of;

static mut BUFFER_I: Buffer<2> = [0; 2];
static mut BUFFER_O: Buffer<1> = [0; 1];

#[unsafe(no_mangle)]
pub extern "C" fn init() -> u32 {
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn tick() -> u32 {
    unsafe { BUFFER_O[0] = BUFFER_I[0] | BUFFER_I[1] };
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn bind_input() -> Bind {
    bind(addr_of!(BUFFER_I))
}

#[unsafe(no_mangle)]
pub extern "C" fn bind_output() -> Bind {
    bind(addr_of!(BUFFER_O))
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
