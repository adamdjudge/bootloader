use core::fmt;
use core::mem;

use crate::port;

/// Console width in characters.
pub const WIDTH: usize = 80;
/// Console height in characters.
pub const HEIGHT: usize = 25;

/// Text color for characters and background shown on the console.
#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Color {
    Black,
    Blue,
    Green,
    Cyan,
    Red,
    Magenta,
    Brown,
    LightGray,
    Gray,
    LightBlue,
    LightGreen,
    LightCyan,
    LightRed,
    Pink,
    Yellow,
    White,
}

impl TryFrom<u8> for Color {
    type Error = u8;
    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0..16 => Ok(unsafe { mem::transmute(val) }),
            _ => Err(val),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(C)]
struct VgaChar {
    char: u8,
    color: u8,
}

impl VgaChar {
    fn from(c: u8) -> Self {
        Self {
            char: c,
            color: unsafe { COLOR },
        }
    }
}

#[repr(transparent)]
struct VgaBuffer {
    chars: [VgaChar; WIDTH * HEIGHT],
}

impl VgaBuffer {
    fn get() -> &'static mut Self {
        unsafe { &mut *(0xb8000 as *mut Self) }
    }
}

static mut POSITION: usize = 0;
static mut COLOR: u8 = Color::LightGray as u8;

fn update_cursor() {
    unsafe {
        port::out8(0x3d4, 0xf);
        port::out8(0x3d5, (POSITION & 0xff) as u8);
        port::out8(0x3d4, 0xe);
        port::out8(0x3d5, ((POSITION >> 8) & 0xff) as u8);
    }
}

/// Sets the current position of the console cursor. Returns `Ok` if the given cursor position is
/// within bounds (`pos < WIDTH * HEIGHT`), otherwise returns `Err(pos)` and has no effect.
pub fn set_position(pos: usize) -> Result<(), usize> {
    if pos < WIDTH * HEIGHT {
        unsafe {
            POSITION = pos;
        }
        update_cursor();
        Ok(())
    } else {
        Err(pos)
    }
}

/// Returns the current position of the console cursor.
pub fn get_position() -> usize {
    unsafe { POSITION }
}

/// Sets the text color for subsequent character writes to the console.
pub fn set_text_color(color: Color) {
    unsafe {
        COLOR = COLOR & 0xf0 | color as u8;
    }
}

/// Sets the background color for subsequent character writes to the console.
pub fn set_bg_color(color: Color) {
    unsafe {
        COLOR = COLOR & 0x0f | (color as u8) << 4;
    }
}

fn advance(count: usize) {
    let pos = get_position() + count;
    if pos < WIDTH * HEIGHT {
        let _ = set_position(pos);
    } else {
        let _ = set_position(WIDTH * (HEIGHT - 1));

        // Scroll text lines up, and then clear the bottom line
        let buffer = VgaBuffer::get();
        for line in 1..HEIGHT {
            let (prev, curr) = buffer.chars.split_at_mut(line * WIDTH);
            prev[(line - 1) * WIDTH..].clone_from_slice(&curr[..WIDTH]);
        }
        buffer.chars[(HEIGHT - 1) * WIDTH..].fill(VgaChar::from(0));
    }
}

fn put_byte(b: u8) {
    let pos = get_position();
    VgaBuffer::get().chars[pos] = VgaChar::from(b);
    advance(1);
}

/// Clears the console by removing all text.
pub fn clear() {
    let _ = set_position(0);
    set_text_color(Color::LightGray);
    set_bg_color(Color::Black);
    VgaBuffer::get().chars.fill(VgaChar::from(0));
}

/// Writes one character to the console.
pub fn put_char(c: char) {
    match c {
        '\0' => put_byte(0),
        '\n' => advance(WIDTH - get_position() % WIDTH),
        ' '..='~' => put_byte(c as u8),
        '\u{80}'.. => put_byte(0xfe),
        _ => {}
    }
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
        for c in s.chars() {
            put_char(c);
        }
        Ok(())
    }
}
