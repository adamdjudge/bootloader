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
use serial::{BaudRate, ComPort, SerialPort};

global_asm!(include_str!("start.s"), options(att_syntax));

#[unsafe(no_mangle)]
fn main() -> ! {
    let mut writer = Writer::get();
    writer.clear_screen();
    let _ = write!(&mut writer, "Loading kernel over COM1 at 9600 baud...\n");

    let serial = SerialPort::get(ComPort::Com1);
    serial.set_baud_rate(BaudRate::try_from(9600).unwrap());
    serial.reset();

    let start_addr = serial::load_kernel(&serial);
    unsafe {
        asm!(
            "jmp eax",
            in("eax") start_addr,
            options(noreturn)
        );
    }
}

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let mut writer = Writer::get();
    writer.set_bg_color(Color::Black);
    writer.set_text_color(Color::LightRed);

    if let Some(location) = info.location() {
        let _ = write!(
            &mut writer,
            "\npanicked at {}:{} - {}",
            location.file(),
            location.line(),
            info.message()
        );
    } else {
        let _ = write!(&mut writer, "\npanicked - {}", info.message());
    }

    loop {
        unsafe {
            asm!("hlt");
        }
    }
}
