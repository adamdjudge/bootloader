#![no_std]
#![no_main]

mod console;
mod port;

use core::arch::{asm, global_asm};
use core::fmt::Write;
use core::panic::PanicInfo;

global_asm!(include_str!("start.s"), options(att_syntax));

#[unsafe(no_mangle)]
fn main() -> ! {
    console::clear();
    let mut writer = console::ConsoleWriter::new();
    for i in 0..=10000 {
        let _ = write!(writer, "Hello from Rust! This is line {}\n", i);
    }

    loop {
        unsafe {
            asm!("hlt");
        }
    }
}

#[inline(never)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
