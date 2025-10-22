#![no_std]
#![no_main]

use core::arch::global_asm;
use core::panic::PanicInfo;

global_asm!(include_str!("start.s"), options(att_syntax));

#[unsafe(no_mangle)]
fn main() -> ! {
    let screen = 0xb8000 as *mut u8;
    for i in 0..80*25 {
        unsafe { *screen.offset(i*2) = 0; }
    }
    for (i, c) in "Hello, world!".bytes().enumerate() {
        unsafe { *screen.offset(i as isize * 2) = c; }
    }
    loop {}
}

#[inline(never)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
