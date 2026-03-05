#![no_std]
#![no_main]

mod console;
mod crc32;
mod port;
mod serial;

use core::arch::{asm, global_asm};
use core::fmt::Write;
use core::panic::PanicInfo;

use console::{Color, Writer};
use serial::{ComPort, SerialPort};

global_asm!(include_str!("start.s"), options(att_syntax));

#[unsafe(no_mangle)]
fn main() -> ! {
    let writer = Writer::get();
    writer.clear_screen();
    let _ = write!(writer, "Loading kernel over COM1 at 19200 baud...\n");

    let serial = SerialPort::get(ComPort::Com1, 19200);
    let start_addr = serial::load_kernel(&serial);

    // Jump to the start address of the kernel, ending bootloader execution.
    unsafe {
        asm!(
            "jmp eax",
            in("eax") start_addr,
            options(noreturn)
        );
    }
}

/// Global panic handler for the bootloader.
#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    unsafe {
        asm!("cli");
    }

    let writer = Writer::get();
    writer.set_bg_color(Color::Black);
    writer.set_text_color(Color::LightRed);

    if let Some(location) = info.location() {
        let _ = write!(
            writer,
            "\npanicked at {}:{} - {}",
            location.file(),
            location.line(),
            info.message()
        );
    } else {
        let _ = write!(writer, "\npanicked - {}", info.message());
    }

    loop {
        unsafe {
            asm!("hlt");
        }
    }
}
