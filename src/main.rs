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
    let mut writer = console::Writer::get();
    writer.clear_screen();
    let _ = write!(&mut writer, "Hello from Rust on x86!\n\n");

    for bg in 0..16 {
        writer.set_bg_color(bg.try_into().unwrap());
        for fg in 0..16 {
            writer.set_text_color(fg.try_into().unwrap());
            let _ = write!(&mut writer, " {:02x} ", bg << 4 | fg);
        }
        writer.put_char('\n');
    }

    loop {
        unsafe {
            asm!("hlt");
        }
    }
}

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let mut writer = console::Writer::get();
    writer.set_bg_color(console::Color::Black);
    writer.set_text_color(console::Color::LightRed);

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

    loop {}
}
