use core::fmt;
use core::ptr;
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::port;

/// Console width in characters.
pub const WIDTH: usize = 80;
/// Console height in characters.
pub const HEIGHT: usize = 25;

const BUFFER: *mut u8 = 0xb8000 as *mut u8;
static POSITION: AtomicUsize = AtomicUsize::new(0);

fn update_cursor() {
    let pos = POSITION.load(Ordering::Relaxed);
    unsafe {
        // Set cursor position.
        port::out8(0x3d4, 0xf);
        port::out8(0x3d5, (pos & 0xff) as u8);
        port::out8(0x3d4, 0xe);
        port::out8(0x3d5, ((pos >> 8) & 0xff) as u8);
    }
}

/// Initialize the console.
pub fn init() {
    unsafe {
        // Set cursor shape to block.
        port::out8(0x3d4, 0xa);
        port::out8(0x3d5, port::in8(0x3d5) & 0xc0);
        port::out8(0x3d4, 0xb);
        port::out8(0x3d5, port::in8(0x3d5) & 0xe0 | 0xf);
    }
    clear();
}

/// Clears the console by removing all text.
pub fn clear() {
    for i in 0..WIDTH*HEIGHT {
        unsafe { ptr::write_volatile(BUFFER.add(i * 2), 0); }
    }
    POSITION.store(0, Ordering::Relaxed);
    update_cursor();
}

/// Used for writing strings to the console.
pub struct ConsoleWriter {}

impl ConsoleWriter {
    pub fn new() -> Self {
        Self {}
    }
}

impl fmt::Write for ConsoleWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.bytes() {
            let pos = POSITION.fetch_add(1, Ordering::Relaxed);
            unsafe { ptr::write_volatile(BUFFER.add(pos * 2), c); }
        }
        update_cursor();
        Ok(())
    }
}
