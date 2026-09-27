#![no_std]
#![no_main]

mod console;
mod crc32;
mod memory;
mod mmu;
mod port;
mod serial;

use core::arch::{asm, global_asm};
use core::panic::PanicInfo;

use console::Color;
use serial::{ComPort, SerialPort};

global_asm!(include_str!("start.s"), options(att_syntax));

#[unsafe(no_mangle)]
fn main() -> ! {
    mmu::init();
    console::clear();

    println!("Memory regions from BIOS:");
    for region in memory::get_regions() {
        println!(
            "  0x{:08x} - 0x{:08x} {:?}",
            region.addr,
            region.addr + region.size - 1,
            region.rtype
        );
    }

    println!("Loading kernel over COM1 at 19200 baud...");
    let serial = SerialPort::get(ComPort::Com1, 19200);
    let start_addr = serial::load_kernel(&serial);

    // Reset the stack pointer and push a null return address, then transfer execution to kernel.
    unsafe {
        asm!(
            "mov esp, __loram_top
            push 0
            jmp eax",
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

    console::set_bg_color(Color::Black);
    console::set_text_color(Color::LightRed);

    if let Some(location) = info.location() {
        print!(
            "\npanicked at {}:{} - {}",
            location.file(),
            location.line(),
            info.message()
        );
    } else {
        print!("\npanicked - {}", info.message());
    }

    loop {
        unsafe {
            asm!("hlt");
        }
    }
}
