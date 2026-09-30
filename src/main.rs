#![no_std]
#![no_main]

mod console;
mod crc32;
mod loader;
mod memory;
mod mmu;
mod port;
mod serial;

use core::arch::{asm, global_asm};
use core::panic::PanicInfo;

use console::Color;
use loader::Loader;
use serial::{ComPort, SerialPort};

global_asm!(include_str!("start.s"), options(att_syntax));

#[unsafe(no_mangle)]
fn main() -> ! {
    mmu::init();
    console::clear();

    println!("Memory regions from BIOS:");
    let regions = memory::get_regions();
    for region in regions {
        println!(
            "  0x{:08x} - 0x{:08x} {:?}",
            region.addr,
            region.addr + region.size as u32 - 1,
            region.rtype
        );
    }

    let loader = loader::TestLoader;
    let (_, base) = loader.load_kernel(regions).unwrap();
    println!("KernelAddress: virt_base=0x{:08x} phys_base=0x{:08x}", base.virt_base, base.phys_base);

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
