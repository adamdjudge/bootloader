#![no_std]
#![no_main]

mod port;
mod console;

use core::arch::global_asm;
use core::fmt::Write;
use core::panic::PanicInfo;

global_asm!(include_str!("start.s"), options(att_syntax));

#[unsafe(no_mangle)]
fn main() -> ! {
    console::init();
    let mut writer = console::ConsoleWriter::new();
    writer.write_str("Hello from Rust!").unwrap();
    loop {}
}

#[inline(never)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
